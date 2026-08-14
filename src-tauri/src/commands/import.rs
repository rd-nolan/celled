use std::path::Path;

use tauri::State;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::commands::template::{require_template, template_embeddings};
use crate::domain::{
    ConfirmMappingRequest, ImportSession, ImportStatus, MappingSource, SourceColumn,
    UpdateMappingRequest,
};
use crate::error::AppError;
use crate::excel::{build_preview, headers_at, sample_values, ExcelReader, HeaderDetector};
use crate::mapping::{mapping_conflict_message, normalize_header, HeaderMatcher};

const PREVIEW_ROWS: usize = 16;

#[tauri::command]
pub async fn analyze_data_excel(
    path: String,
    template_id: String,
    sheet_name: Option<String>,
    state: State<'_, AppState>,
) -> Result<ImportSession, AppError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        analyze_data_excel_inner(&path, &template_id, sheet_name.as_deref(), &state)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
}

#[tauri::command]
pub async fn update_import_header_row(
    session_id: String,
    header_row: usize,
    state: State<'_, AppState>,
) -> Result<ImportSession, AppError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        update_import_header_row_inner(&session_id, header_row, None, &state)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
}

#[tauri::command]
pub async fn update_import_sheet(
    session_id: String,
    sheet_name: String,
    state: State<'_, AppState>,
) -> Result<ImportSession, AppError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        update_import_header_row_inner(&session_id, 0, Some(sheet_name), &state)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
}

#[tauri::command]
pub async fn update_mapping(
    request: UpdateMappingRequest,
    state: State<'_, AppState>,
) -> Result<ImportSession, AppError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || update_mapping_inner(request, &state))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
}

#[tauri::command]
pub async fn confirm_mapping(
    request: ConfirmMappingRequest,
    state: State<'_, AppState>,
) -> Result<ImportSession, AppError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || confirm_mapping_inner(request, &state))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
}

fn analyze_data_excel_inner(
    path: &str,
    template_id: &str,
    sheet_name: Option<&str>,
    state: &AppState,
) -> Result<ImportSession, AppError> {
    let template = require_template(state)?;
    if template.id != template_id {
        return Err(AppError::TemplateNotConfirmed);
    }
    let path = Path::new(path);
    let sheets = ExcelReader::list_sheets(path)?;
    let sheet_name = match sheet_name {
        Some(name) if sheets.iter().any(|s| s == name) => name.to_string(),
        _ => ExcelReader::first_non_empty_sheet(path)?,
    };
    let data = ExcelReader::read_sheet(path, &sheet_name, Some(PREVIEW_ROWS))?;
    let detection = HeaderDetector::detect(&data.rows);
    let session = build_session(
        Uuid::new_v4().to_string(),
        path,
        sheets,
        sheet_name,
        detection.row_index,
        state,
        &template,
        &data,
    )?;
    if let Ok(mut sessions) = state.sessions.lock() {
        sessions.insert(session.id.clone(), session.clone());
    }
    Ok(session)
}

fn update_import_header_row_inner(
    session_id: &str,
    header_row: usize,
    sheet_name: Option<String>,
    state: &AppState,
) -> Result<ImportSession, AppError> {
    let template = require_template(state)?;
    let existing = {
        let sessions = state
            .sessions
            .lock()
            .map_err(|_| AppError::Internal("sessions lock poisoned".into()))?;
        sessions.get(session_id).cloned().ok_or(AppError::SessionNotFound)?
    };
    let path = Path::new(&existing.file_path);
    let sheets = ExcelReader::list_sheets(path)?;
    let sheet_name = sheet_name.unwrap_or(existing.sheet_name);
    let data = ExcelReader::read_sheet(path, &sheet_name, Some(PREVIEW_ROWS))?;
    let header_row = if header_row == 0 {
        HeaderDetector::detect(&data.rows).row_index
    } else {
        header_row
    };
    let session = build_session(
        existing.id,
        path,
        sheets,
        sheet_name,
        header_row,
        state,
        &template,
        &data,
    )?;
    if let Ok(mut sessions) = state.sessions.lock() {
        sessions.insert(session.id.clone(), session.clone());
    }
    Ok(session)
}

fn build_session(
    id: String,
    path: &Path,
    sheets: Vec<String>,
    sheet_name: String,
    header_row: usize,
    state: &AppState,
    template: &crate::domain::TemplateSchema,
    data: &crate::excel::SheetData,
) -> Result<ImportSession, AppError> {
    let headers = headers_at(data, header_row)?;
    if headers.iter().all(|h| h.trim().is_empty()) {
        return Err(AppError::InvalidHeaderRow);
    }
    let data_start_row = header_row + 1;
    let source_columns: Vec<SourceColumn> = headers
        .iter()
        .enumerate()
        .filter(|(_, name)| !name.trim().is_empty())
        .map(|(index, name)| SourceColumn {
            index,
            header: name.clone(),
            normalized_header: normalize_header(name),
            sample_values: sample_values(data, data_start_row, index, 5),
        })
        .collect();

    let embeddings = template_embeddings(state, template)?;
    let matcher = HeaderMatcher {
        template,
        template_embeddings: &embeddings,
        history: state.database.as_ref(),
        alias: state.alias.as_ref(),
        embedding: state.embedding.as_ref(),
    };
    let mappings = matcher.match_headers(&source_columns)?;

    Ok(ImportSession {
        id,
        file_path: path.to_string_lossy().to_string(),
        file_name: ExcelReader::file_name(path),
        sheet_name,
        sheets,
        header_row,
        data_start_row,
        source_columns,
        mappings,
        preview: build_preview(data, header_row),
        confirmed: false,
        status: ImportStatus::Pending,
        error: None,
    })
}

fn update_mapping_inner(
    request: UpdateMappingRequest,
    state: &AppState,
) -> Result<ImportSession, AppError> {
    let template = require_template(state)?;
    let mut sessions = state
        .sessions
        .lock()
        .map_err(|_| AppError::Internal("sessions lock poisoned".into()))?;
    let session = sessions
        .get_mut(&request.session_id)
        .ok_or(AppError::SessionNotFound)?;

    if let Some(target_index) = request.target_column_index {
        let target = template
            .columns
            .iter()
            .find(|c| c.index == target_index)
            .ok_or_else(|| AppError::InvalidMapping("模板字段不存在".into()))?;
        if let Some(message) =
            mapping_conflict_message(&session.mappings, request.source_column_index, target_index, &target.name)
        {
            return Err(AppError::MappingConflict(message));
        }
        let mapping = session
            .mappings
            .iter_mut()
            .find(|m| m.source_column_index == request.source_column_index)
            .ok_or_else(|| AppError::InvalidMapping("源字段不存在".into()))?;
        mapping.target_column_index = Some(target_index);
        mapping.target_header = Some(target.name.clone());
        mapping.source = MappingSource::Manual;
        mapping.score = None;
    } else {
        let mapping = session
            .mappings
            .iter_mut()
            .find(|m| m.source_column_index == request.source_column_index)
            .ok_or_else(|| AppError::InvalidMapping("源字段不存在".into()))?;
        mapping.target_column_index = None;
        mapping.target_header = None;
        mapping.source = MappingSource::Unmatched;
        mapping.score = None;
    }

    session.confirmed = false;
    session.status = ImportStatus::Pending;
    Ok(session.clone())
}

fn confirm_mapping_inner(
    request: ConfirmMappingRequest,
    state: &AppState,
) -> Result<ImportSession, AppError> {
    let mut sessions = state
        .sessions
        .lock()
        .map_err(|_| AppError::Internal("sessions lock poisoned".into()))?;
    let session = sessions
        .get_mut(&request.session_id)
        .ok_or(AppError::SessionNotFound)?;

    let mut seen = Vec::new();
    for mapping in &session.mappings {
        if let Some(target) = mapping.target_column_index {
            if seen.contains(&target) {
                return Err(AppError::MappingConflict(format!(
                    "“{}”存在重复映射",
                    mapping.target_header.clone().unwrap_or_default()
                )));
            }
            seen.push(target);
        }
    }

    state.database.record_confirmed_mappings(&session.mappings)?;
    session.confirmed = true;
    session.status = ImportStatus::Confirmed;
    Ok(session.clone())
}

