use std::path::{Path, PathBuf};

use tauri::State;

use crate::app_state::AppState;
use crate::commands::template::require_template;
use crate::domain::{ConvertRequest, OutputFile};
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

fn convert_files_inner(request: ConvertRequest, state: &AppState) -> Result<Vec<OutputFile>, AppError> {
    let template = require_template(state)?;
    let sessions = state
        .sessions
        .lock()
        .map_err(|_| AppError::Internal("sessions lock poisoned".into()))?;

    if request.session_ids.is_empty() {
        return Err(AppError::NotAllConfirmed);
    }
    for id in &request.session_ids {
        let session = sessions.get(id).ok_or(AppError::SessionNotFound)?;
        if !session.confirmed {
            return Err(AppError::NotAllConfirmed);
        }
    }

    let output_dir = PathBuf::from(&request.output_dir);
    std::fs::create_dir_all(&output_dir)?;

    let mut outputs = Vec::new();
    for id in &request.session_ids {
        let session = sessions.get(id).ok_or(AppError::SessionNotFound)?;
        let data = ExcelReader::read_sheet(Path::new(&session.file_path), &session.sheet_name, None)?;
        let start = session.data_start_row.saturating_sub(1);
        let source_rows = if start < data.rows.len() {
            data.rows[start..].to_vec()
        } else {
            Vec::new()
        };
        let transformed = ExcelTransformer::transform(&source_rows, &session.mappings, &template);
        let file_name = converted_file_name(&session.file_name);
        let output_path = output_dir.join(&file_name);
        ExcelWriter::write_workbook(&output_path, &template, &transformed)?;
        outputs.push(OutputFile {
            session_id: session.id.clone(),
            path: output_path.to_string_lossy().to_string(),
            file_name,
        });
    }
    Ok(outputs)
}

fn converted_file_name(file_name: &str) -> String {
    let path = Path::new(file_name);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    format!("{stem}_converted.xlsx")
}
