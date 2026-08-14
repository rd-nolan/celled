use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MappingSource {
    Exact,
    NormalizedExact,
    History,
    Alias,
    Embedding,
    Manual,
    Unmatched,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchCandidate {
    pub template_column_index: usize,
    pub template_header: String,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderMapping {
    pub source_column_index: usize,
    pub source_header: String,
    pub normalized_source_header: String,
    pub target_column_index: Option<usize>,
    pub target_header: Option<String>,
    pub score: Option<f32>,
    pub source: MappingSource,
    pub candidates: Vec<MatchCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceColumn {
    pub index: usize,
    pub header: String,
    pub normalized_header: String,
    pub sample_values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMappingRequest {
    pub session_id: String,
    pub source_column_index: usize,
    pub target_column_index: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmMappingRequest {
    pub session_id: String,
}
