use serde::{Deserialize, Serialize};

/// Header used when the template has no 来源 / 来源文件 column.
pub const SOURCE_COLUMN_HEADER: &str = "来源";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateColumn {
    pub index: usize,
    pub name: String,
    pub normalized_name: String,
    pub path: Option<String>,
}

impl TemplateColumn {
    pub fn is_source_column(&self) -> bool {
        is_source_column_name(&self.name) || is_source_column_name(&self.normalized_name)
    }
}

pub fn is_source_column_name(name: &str) -> bool {
    matches!(name.trim(), "来源" | "来源文件")
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

impl TemplateSchema {
    pub fn source_column_index(&self) -> Option<usize> {
        self.columns
            .iter()
            .find(|column| column.is_source_column())
            .map(|column| column.index)
    }

    /// Appends 来源 after the template columns unless 来源 / 来源文件 already exists.
    pub fn with_source_column(&self) -> Self {
        let mut schema = self.clone();
        if schema.source_column_index().is_some() {
            return schema;
        }
        schema.columns.push(TemplateColumn {
            index: schema.columns.len(),
            name: SOURCE_COLUMN_HEADER.to_string(),
            normalized_name: SOURCE_COLUMN_HEADER.to_string(),
            path: None,
        });
        schema
    }
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

#[cfg(test)]
mod tests {
    use super::{is_source_column_name, TemplateColumn, TemplateSchema, SOURCE_COLUMN_HEADER};

    fn schema(names: &[&str]) -> TemplateSchema {
        TemplateSchema {
            id: "t".into(),
            file_name: "t.xlsx".into(),
            file_path: "t.xlsx".into(),
            sheet_name: "Sheet1".into(),
            header_start_row: 1,
            header_end_row: 1,
            data_start_row: 2,
            columns: names
                .iter()
                .enumerate()
                .map(|(index, name)| TemplateColumn {
                    index,
                    name: (*name).into(),
                    normalized_name: (*name).into(),
                    path: None,
                })
                .collect(),
        }
    }

    #[test]
    fn recognizes_source_headers() {
        assert!(is_source_column_name("来源"));
        assert!(is_source_column_name(" 来源文件 "));
        assert!(!is_source_column_name("来源说明"));
    }

    #[test]
    fn appends_source_column_when_missing() {
        let out = schema(&["姓名"]).with_source_column();
        assert_eq!(out.columns.len(), 2);
        assert_eq!(
            out.columns.last().map(|c| c.name.as_str()),
            Some(SOURCE_COLUMN_HEADER)
        );
    }

    #[test]
    fn does_not_duplicate_existing_source_column() {
        let out = schema(&["姓名", "来源文件"]).with_source_column();
        assert_eq!(out.columns.len(), 2);
        assert_eq!(out.source_column_index(), Some(1));
    }
}
