use crate::domain::ExcelPreview;
use crate::excel::SheetData;

const PREVIEW_BEFORE: usize = 2;
const PREVIEW_AFTER: usize = 7;

pub fn build_preview(sheet: &SheetData, header_row: usize) -> ExcelPreview {
    let header_idx = header_row.saturating_sub(1);
    let start_idx = header_idx.saturating_sub(PREVIEW_BEFORE);
    let end_idx = (header_idx + PREVIEW_AFTER + 1).min(sheet.rows.len());
    let rows = if start_idx < end_idx {
        sheet.rows[start_idx..end_idx].to_vec()
    } else {
        Vec::new()
    };

    ExcelPreview {
        sheet_name: sheet.name.clone(),
        header_row,
        start_row: start_idx + 1,
        rows,
    }
}

pub fn headers_at(
    sheet: &SheetData,
    header_row: usize,
) -> Result<Vec<String>, crate::error::AppError> {
    let idx = header_row.saturating_sub(1);
    sheet
        .rows
        .get(idx)
        .cloned()
        .ok_or(crate::error::AppError::InvalidHeaderRow)
}

pub fn sample_values(
    sheet: &SheetData,
    data_start_row: usize,
    column: usize,
    limit: usize,
) -> Vec<String> {
    let start = data_start_row.saturating_sub(1);
    sheet
        .rows
        .iter()
        .skip(start)
        .filter_map(|row| row.get(column).map(|v| v.trim().to_string()))
        .filter(|v| !v.is_empty())
        .take(limit)
        .collect()
}
