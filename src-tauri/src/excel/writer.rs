use std::path::Path;

use rust_xlsxwriter::Workbook;

use crate::domain::TemplateSchema;
use crate::error::AppError;

/// Writes converted workbooks. Style preservation is intentionally out of scope for V1.
pub struct ExcelWriter;

impl ExcelWriter {
    pub fn write_workbook(
        path: &Path,
        template: &TemplateSchema,
        data_rows: &[Vec<String>],
    ) -> Result<(), AppError> {
        let mut workbook = Workbook::new();
        let sheet_name = sanitize_sheet_name(&template.sheet_name);
        let worksheet = workbook
            .add_worksheet()
            .set_name(&sheet_name)
            .map_err(|e| AppError::ExcelWriteError(e.to_string()))?;

        for (col, column) in template.columns.iter().enumerate() {
            worksheet
                .write_string(0, col as u16, &column.name)
                .map_err(|e| AppError::ExcelWriteError(e.to_string()))?;
        }

        for (row_idx, row) in data_rows.iter().enumerate() {
            for (col, value) in row.iter().enumerate() {
                worksheet
                    .write_string((row_idx as u32) + 1, col as u16, value)
                    .map_err(|e| AppError::ExcelWriteError(e.to_string()))?;
            }
        }

        workbook
            .save(path)
            .map_err(|e| AppError::ExcelWriteError(e.to_string()))?;
        Ok(())
    }
}

fn sanitize_sheet_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| match c {
            ':' | '\\' | '/' | '?' | '*' | '[' | ']' => ' ',
            other => other,
        })
        .collect();
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        "Sheet1".into()
    } else {
        trimmed.chars().take(31).collect()
    }
}
