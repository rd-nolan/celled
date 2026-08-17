use serde::{Deserialize, Serialize};

use super::{ExcelPreview, HeaderMapping, SourceColumn};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImportStatus {
    Pending,
    Confirmed,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportSession {
    pub id: String,
    pub file_path: String,
    pub file_name: String,
    pub sheet_name: String,
    pub sheets: Vec<String>,
    pub header_row: usize,
    pub data_start_row: usize,
    pub source_columns: Vec<SourceColumn>,
    pub mappings: Vec<HeaderMapping>,
    pub preview: ExcelPreview,
    pub confirmed: bool,
    pub status: ImportStatus,
    pub error: Option<String>,
    pub read_filtered_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertRequest {
    pub session_ids: Vec<String>,
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputFile {
    pub session_id: String,
    pub path: String,
    pub file_name: String,
}
