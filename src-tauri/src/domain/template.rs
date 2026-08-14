use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateColumn {
    pub index: usize,
    pub name: String,
    pub normalized_name: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSchema {
    pub id: String,
    pub file_name: String,
    pub file_path: String,
    pub sheet_name: String,
    pub header_start_row: usize,
    pub header_end_row: usize,
    pub data_start_row: usize,
    pub columns: Vec<TemplateColumn>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderDetectionResult {
    pub row_index: usize,
    pub confidence: f32,
    pub headers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelPreview {
    pub sheet_name: String,
    pub header_row: usize,
    pub start_row: usize,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateAnalysis {
    pub file_path: String,
    pub file_name: String,
    pub sheets: Vec<String>,
    pub sheet_name: String,
    pub detection: HeaderDetectionResult,
    pub preview: ExcelPreview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmTemplateRequest {
    pub file_path: String,
    pub file_name: String,
    pub sheet_name: String,
    pub header_row: usize,
    pub data_start_row: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub embedding_backend: String,
    pub embedding_model_version: String,
}
