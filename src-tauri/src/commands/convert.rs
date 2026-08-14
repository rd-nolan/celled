use std::collections::HashMap;
use std::path::{Path, PathBuf};

use tauri::State;

use crate::app_state::AppState;
use crate::commands::template::require_template;
use crate::domain::{ConvertRequest, ImportSession, OutputFile};
use crate::error::AppError;
use crate::excel::{ExcelReader, ExcelTransformer, ExcelWriter};

#[tauri::command]
pub async fn convert_files(
    request: ConvertRequest,
    state: State<'_, AppState>,
) -> Result<Vec<OutputFile>, AppError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || convert_files_inner(request, &state))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
}

fn convert_files_inner(
    request: ConvertRequest,
    state: &AppState,
) -> Result<Vec<OutputFile>, AppError> {
    let template = require_template(state)?;
    let sessions = state
        .sessions
        .lock()
        .map_err(|_| AppError::Internal("sessions lock poisoned".into()))?;

    if request.session_ids.is_empty() {
        return Err(AppError::NotAllConfirmed);
    }
    let mut name_counts: HashMap<String, usize> = HashMap::new();
    for id in &request.session_ids {
        let session = sessions.get(id).ok_or(AppError::SessionNotFound)?;
        if !session.confirmed {
            return Err(AppError::NotAllConfirmed);
        }
        *name_counts.entry(session.file_name.clone()).or_insert(0) += 1;
    }

    let output_template = template.with_source_column();
    let mut merged_rows = Vec::new();
    for id in &request.session_ids {
        let session = sessions.get(id).ok_or(AppError::SessionNotFound)?;
        let data =
            ExcelReader::read_sheet(Path::new(&session.file_path), &session.sheet_name, None)?;
        let start = session.data_start_row.saturating_sub(1);
        let source_rows = if start < data.rows.len() {
            data.rows[start..].to_vec()
        } else {
            Vec::new()
        };
        let label = source_label(session, &name_counts);
        let transformed =
            ExcelTransformer::transform(&source_rows, &session.mappings, &output_template, &label);
        merged_rows.extend(transformed);
    }

    let output_path = resolve_output_path(&request.output_path, &template.file_name);
    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    ExcelWriter::write_workbook(&output_path, &output_template, &merged_rows)?;

    let file_name = output_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Celled_合并.xlsx")
        .to_string();
    Ok(vec![OutputFile {
        session_id: String::new(),
        path: output_path.to_string_lossy().to_string(),
        file_name,
    }])
}

pub fn merged_output_file_name(template_file_name: &str) -> String {
    let stem = Path::new(template_file_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    match stem {
        Some(stem) => format!("{stem}_合并.xlsx"),
        None => "Celled_合并.xlsx".into(),
    }
}

fn source_label(session: &ImportSession, name_counts: &HashMap<String, usize>) -> String {
    let ambiguous = name_counts.get(&session.file_name).copied().unwrap_or(0) > 1;
    if ambiguous {
        session.file_path.clone()
    } else if !session.file_name.is_empty() {
        session.file_name.clone()
    } else {
        ExcelReader::file_name(Path::new(&session.file_path))
    }
}

fn resolve_output_path(output_path: &str, template_file_name: &str) -> PathBuf {
    let path = PathBuf::from(output_path);
    let looks_like_dir = output_path.ends_with(['/', '\\'])
        || path
            .extension()
            .and_then(|e| e.to_str())
            .map(|ext| !ext.eq_ignore_ascii_case("xlsx"))
            .unwrap_or(true);
    if looks_like_dir && (path.is_dir() || !path.exists()) {
        path.join(merged_output_file_name(template_file_name))
    } else {
        path
    }
}

#[cfg(test)]
mod tests {
    use super::{convert_files_inner, merged_output_file_name};
    use crate::app_state::AppState;
    use crate::database::Database;
    use crate::domain::{
        ConvertRequest, ExcelPreview, HeaderMapping, ImportSession, ImportStatus, MappingSource,
        TemplateColumn, TemplateSchema,
    };
    use crate::embedding::MockEmbeddingProvider;
    use crate::excel::{fixtures, ExcelReader};
    use crate::mapping::AliasDictionary;
    use std::sync::Arc;
    use tempfile::tempdir;

    fn mapping(
        source_index: usize,
        source_header: &str,
        target_index: usize,
        target_header: &str,
    ) -> HeaderMapping {
        HeaderMapping {
            source_column_index: source_index,
            source_header: source_header.into(),
            normalized_source_header: source_header.into(),
            target_column_index: Some(target_index),
            target_header: Some(target_header.into()),
            score: Some(1.0),
            source: MappingSource::Manual,
            candidates: vec![],
        }
    }

    fn preview(sheet_name: &str) -> ExcelPreview {
        ExcelPreview {
            sheet_name: sheet_name.into(),
            header_row: 1,
            start_row: 2,
            rows: vec![],
        }
    }

    #[test]
    fn default_merged_name_uses_template_stem() {
        assert_eq!(
            merged_output_file_name("人员信息.xlsx"),
            "人员信息_合并.xlsx"
        );
        assert_eq!(merged_output_file_name(""), "Celled_合并.xlsx");
    }

    #[test]
    fn merges_two_source_tables_into_one_template_shaped_workbook() {
        let dir = tempdir().unwrap();
        fixtures::write_simple(
            &dir.path().join("source-a.xlsx"),
            "Sheet1",
            &[&["姓名", "学生编号", "备注"], &["张三", "A001", "忽略"]],
        )
        .unwrap();
        fixtures::write_simple(
            &dir.path().join("source-b.xlsx"),
            "Sheet1",
            &[&["名字", "学号"], &["李四", "B002"], &["王五", "B003"]],
        )
        .unwrap();

        let template = TemplateSchema {
            id: "tmpl".into(),
            file_name: "报名表.xlsx".into(),
            file_path: "报名表.xlsx".into(),
            sheet_name: "汇总".into(),
            header_start_row: 1,
            header_end_row: 1,
            data_start_row: 2,
            columns: vec![
                TemplateColumn {
                    index: 0,
                    name: "姓名".into(),
                    normalized_name: "姓名".into(),
                    path: None,
                },
                TemplateColumn {
                    index: 1,
                    name: "学号".into(),
                    normalized_name: "学号".into(),
                    path: None,
                },
                TemplateColumn {
                    index: 2,
                    name: "班级".into(),
                    normalized_name: "班级".into(),
                    path: None,
                },
            ],
        };

        let session_a = ImportSession {
            id: "a".into(),
            file_path: dir
                .path()
                .join("source-a.xlsx")
                .to_string_lossy()
                .to_string(),
            file_name: "source-a.xlsx".into(),
            sheet_name: "Sheet1".into(),
            sheets: vec!["Sheet1".into()],
            header_row: 1,
            data_start_row: 2,
            source_columns: vec![],
            mappings: vec![
                mapping(0, "姓名", 0, "姓名"),
                mapping(1, "学生编号", 1, "学号"),
            ],
            preview: preview("Sheet1"),
            confirmed: true,
            status: ImportStatus::Confirmed,
            error: None,
        };
        let session_b = ImportSession {
            id: "b".into(),
            file_path: dir
                .path()
                .join("source-b.xlsx")
                .to_string_lossy()
                .to_string(),
            file_name: "source-b.xlsx".into(),
            sheet_name: "Sheet1".into(),
            sheets: vec!["Sheet1".into()],
            header_row: 1,
            data_start_row: 2,
            source_columns: vec![],
            mappings: vec![mapping(0, "名字", 0, "姓名"), mapping(1, "学号", 1, "学号")],
            preview: preview("Sheet1"),
            confirmed: true,
            status: ImportStatus::Confirmed,
            error: None,
        };

        let state = AppState::new(
            Arc::new(MockEmbeddingProvider),
            Database::memory().unwrap(),
            AliasDictionary::default(),
        );
        *state.template.lock().unwrap() = Some(template);
        {
            let mut sessions = state.sessions.lock().unwrap();
            sessions.insert(session_a.id.clone(), session_a);
            sessions.insert(session_b.id.clone(), session_b);
        }

        let output_path = dir.path().join("out.xlsx");
        let outputs = convert_files_inner(
            ConvertRequest {
                session_ids: vec!["a".into(), "b".into()],
                output_path: output_path.to_string_lossy().to_string(),
            },
            &state,
        )
        .unwrap();

        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].file_name, "out.xlsx");
        assert!(output_path.exists());

        let data = ExcelReader::read_sheet(&output_path, "汇总", None).unwrap();
        assert_eq!(data.rows[0], vec!["姓名", "学号", "班级", "来源"]);
        assert_eq!(data.rows[1], vec!["张三", "A001", "", "source-a.xlsx"]);
        assert_eq!(data.rows[2], vec!["李四", "B002", "", "source-b.xlsx"]);
        assert_eq!(data.rows[3], vec!["王五", "B003", "", "source-b.xlsx"]);
        assert_eq!(data.rows.len(), 4);
    }
}
