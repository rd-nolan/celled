use std::path::Path;

use tauri::State;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::domain::{
    ConfirmTemplateRequest, HeaderDetectionResult, TemplateAnalysis, TemplateColumn, TemplateSchema,
};
use crate::embedding::CachedTemplateEmbedding;
use crate::error::AppError;
use crate::excel::{build_preview, headers_at, ExcelReader, HeaderDetector};
use crate::mapping::normalize_header;

const PREVIEW_ROWS: usize = 16;

#[tauri::command]
pub async fn get_app_info(state: State<'_, AppState>) -> Result<crate::domain::AppInfo, AppError> {
    Ok(crate::domain::AppInfo {
        embedding_backend: state.embedding.backend_name().to_string(),
        embedding_model_version: state.embedding.model_version().to_string(),
    })
}

#[tauri::command]
pub async fn analyze_template(
    path: String,
    sheet_name: Option<String>,
    state: State<'_, AppState>,
) -> Result<TemplateAnalysis, AppError> {
    let _ = state;
    tauri::async_runtime::spawn_blocking(move || analyze_template_inner(&path, sheet_name.as_deref()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
}

#[tauri::command]
pub async fn update_template_header_row(
    path: String,
    sheet_name: String,
    header_row: usize,
    state: State<'_, AppState>,
) -> Result<TemplateAnalysis, AppError> {
    let _ = state;
    tauri::async_runtime::spawn_blocking(move || {
        update_header_row_inner(&path, &sheet_name, header_row)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
}

#[tauri::command]
pub async fn confirm_template(
    request: ConfirmTemplateRequest,
    state: State<'_, AppState>,
) -> Result<TemplateSchema, AppError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || confirm_template_inner(request, &state))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
}

fn analyze_template_inner(path: &str, sheet_name: Option<&str>) -> Result<TemplateAnalysis, AppError> {
    let path = Path::new(path);
    let sheets = ExcelReader::list_sheets(path)?;
    if sheets.is_empty() {
        return Err(AppError::EmptyWorksheet);
    }
    let sheet_name = match sheet_name {
        Some(name) if sheets.iter().any(|s| s == name) => name.to_string(),
        _ => ExcelReader::first_non_empty_sheet(path)?,
    };
    let data = ExcelReader::read_sheet(path, &sheet_name, Some(PREVIEW_ROWS))?;
    if data.rows.iter().all(|row| row.iter().all(|c| c.trim().is_empty())) {
        return Err(AppError::EmptyWorksheet);
    }
    let detection = HeaderDetector::detect(&data.rows);
    let preview = build_preview(&data, detection.row_index);
    Ok(TemplateAnalysis {
        file_path: path.to_string_lossy().to_string(),
        file_name: ExcelReader::file_name(path),
        sheets,
        sheet_name,
        detection,
        preview,
    })
}

fn update_header_row_inner(
    path: &str,
    sheet_name: &str,
    header_row: usize,
) -> Result<TemplateAnalysis, AppError> {
    let path = Path::new(path);
    let sheets = ExcelReader::list_sheets(path)?;
    let data = ExcelReader::read_sheet(path, sheet_name, Some(PREVIEW_ROWS))?;
    let headers = headers_at(&data, header_row)?;
    if headers.iter().all(|h| h.trim().is_empty()) {
        return Err(AppError::InvalidHeaderRow);
    }
    let detection = HeaderDetectionResult {
        row_index: header_row,
        confidence: 1.0,
        headers,
    };
    Ok(TemplateAnalysis {
        file_path: path.to_string_lossy().to_string(),
        file_name: ExcelReader::file_name(path),
        sheets,
        sheet_name: sheet_name.to_string(),
        detection,
        preview: build_preview(&data, header_row),
    })
}

fn confirm_template_inner(
    request: ConfirmTemplateRequest,
    state: &AppState,
) -> Result<TemplateSchema, AppError> {
    let path = Path::new(&request.file_path);
    let data = ExcelReader::read_sheet(path, &request.sheet_name, Some(PREVIEW_ROWS))?;
    let headers = headers_at(&data, request.header_row)?;
    if headers.iter().all(|h| h.trim().is_empty()) {
        return Err(AppError::InvalidHeaderRow);
    }

    let columns: Vec<TemplateColumn> = headers
        .into_iter()
        .enumerate()
        .filter(|(_, name)| !name.trim().is_empty())
        .map(|(index, name)| TemplateColumn {
            index,
            normalized_name: normalize_header(&name),
            name,
            path: None,
        })
        .collect();

    if columns.is_empty() {
        return Err(AppError::HeaderNotFound);
    }

    let data_start_row = request.data_start_row.unwrap_or(request.header_row + 1);
    let template = TemplateSchema {
        id: Uuid::new_v4().to_string(),
        file_name: request.file_name,
        file_path: request.file_path,
        sheet_name: request.sheet_name,
        header_start_row: request.header_row,
        header_end_row: request.header_row,
        data_start_row,
        columns,
    };

    let texts: Vec<String> = template
        .columns
        .iter()
        .map(|c| c.normalized_name.clone())
        .collect();
    let embeddings = state.embedding.embed(&texts)?;
    let version = state.embedding.model_version().to_string();
    state
        .database
        .save_template(&template, &embeddings, &version)?;
    if let Ok(mut cache) = state.template_cache.lock() {
        cache.insert(CachedTemplateEmbedding {
            template_id: template.id.clone(),
            model_version: version,
            vectors: embeddings,
        });
    }
    if let Ok(mut slot) = state.template.lock() {
        *slot = Some(template.clone());
    }
    if let Ok(mut sessions) = state.sessions.lock() {
        sessions.clear();
    }
    Ok(template)
}

pub fn require_template(state: &AppState) -> Result<TemplateSchema, AppError> {
    state
        .template
        .lock()
        .map_err(|_| AppError::Internal("template lock poisoned".into()))?
        .clone()
        .ok_or(AppError::TemplateNotConfirmed)
}

pub fn template_embeddings(state: &AppState, template: &TemplateSchema) -> Result<Vec<Vec<f32>>, AppError> {
    let version = state.embedding.model_version().to_string();
    if let Ok(cache) = state.template_cache.lock() {
        if let Some(vectors) = cache.get(&template.id, &version) {
            return Ok(vectors.to_vec());
        }
    }
    if let Some(stored) = state.database.load_template_embeddings(&template.id, &version)? {
        if let Ok(mut cache) = state.template_cache.lock() {
            cache.insert(CachedTemplateEmbedding {
                template_id: template.id.clone(),
                model_version: version.clone(),
                vectors: stored.clone(),
            });
        }
        return Ok(stored);
    }
    let texts: Vec<String> = template
        .columns
        .iter()
        .map(|c| c.normalized_name.clone())
        .collect();
    let embeddings = state.embedding.embed(&texts)?;
    if let Ok(mut cache) = state.template_cache.lock() {
        cache.insert(CachedTemplateEmbedding {
            template_id: template.id.clone(),
            model_version: version,
            vectors: embeddings.clone(),
        });
    }
    Ok(embeddings)
}
