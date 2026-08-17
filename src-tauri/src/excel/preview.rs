use crate::domain::ExcelPreview;
use crate::excel::SheetData;

const PREVIEW_BEFORE: usize = 2;
const PREVIEW_AFTER: usize = 7;

pub fn build_preview(sheet: &SheetData, header_row: usize, visible_only: bool) -> ExcelPreview {
    let header_idx = header_row.saturating_sub(1);
    let start_idx = header_idx.saturating_sub(PREVIEW_BEFORE);
    let end_idx = (header_idx + PREVIEW_AFTER + 1).min(sheet.rows.len());
    let rows = if start_idx < end_idx {
        sheet.rows[start_idx..end_idx]
            .iter()
            .enumerate()
            .filter(|(offset, _)| sheet.is_row_visible(start_idx + offset + 1, visible_only))
            .map(|(_, row)| row.clone())
            .collect()
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
    visible_only: bool,
) -> Result<Vec<String>, crate::error::AppError> {
    if !sheet.is_row_visible(header_row, visible_only) {
        return Err(crate::error::AppError::InvalidHeaderRow);
    }
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
    visible_only: bool,
) -> Vec<String> {
    let start = data_start_row.saturating_sub(1);
    sheet
        .rows
        .iter()
        .enumerate()
        .skip(start)
        .filter(|(idx, _)| sheet.is_row_visible(idx + 1, visible_only))
        .filter_map(|(_, row)| row.get(column).map(|v| v.trim().to_string()))
        .filter(|v| !v.is_empty())
        .take(limit)
        .collect()
}

pub fn visible_data_rows(sheet: &SheetData, data_start_row: usize, visible_only: bool) -> Vec<Vec<String>> {
    let start = data_start_row.saturating_sub(1);
    sheet
        .rows
        .iter()
        .enumerate()
        .skip(start)
        .filter(|(idx, _)| sheet.is_row_visible(idx + 1, visible_only))
        .map(|(_, row)| row.clone())
        .collect()
}
