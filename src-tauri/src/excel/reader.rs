use std::path::Path;

use calamine::{open_workbook_auto, Data, Reader};

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct SheetData {
    pub name: String,
    pub rows: Vec<Vec<String>>,
}

/// Reads Excel workbooks via calamine. Never send full sheets to the frontend.
pub struct ExcelReader;

impl ExcelReader {
    pub fn list_sheets(path: &Path) -> Result<Vec<String>, AppError> {
        ensure_excel(path)?;
        let workbook = open_workbook_auto(path)?;
        Ok(workbook.sheet_names())
    }

    pub fn first_non_empty_sheet(path: &Path) -> Result<String, AppError> {
        let sheets = Self::list_sheets(path)?;
        if sheets.is_empty() {
            return Err(AppError::EmptyWorksheet);
        }
        for name in &sheets {
            let data = Self::read_sheet(path, name, Some(8))?;
            if data.rows.iter().any(|row| row.iter().any(|c| !c.trim().is_empty())) {
                return Ok(name.clone());
            }
        }
        Ok(sheets[0].clone())
    }

    pub fn read_sheet(
        path: &Path,
        sheet_name: &str,
        max_rows: Option<usize>,
    ) -> Result<SheetData, AppError> {
        ensure_excel(path)?;
        let mut workbook = open_workbook_auto(path)?;
        let range = workbook
            .worksheet_range(sheet_name)
            .map_err(|_| AppError::SheetNotFound)?;

        if range.is_empty() {
            return Ok(SheetData {
                name: sheet_name.to_string(),
                rows: Vec::new(),
            });
        }

        let start_row = range.start().map(|s| s.0 as usize).unwrap_or(0);
        let start_col = range.start().map(|s| s.1 as usize).unwrap_or(0);
        let height = range.height();
        let width = range.width();
        let abs_width = start_col + width;
        let abs_height = match max_rows {
            Some(max) => (start_row + height).min(max),
            None => start_row + height,
        };

        let mut rows = vec![vec![String::new(); abs_width.max(1)]; abs_height];
        for r in 0..height {
            let abs_r = start_row + r;
            if abs_r >= abs_height {
                break;
            }
            for c in 0..width {
                if let Some(cell) = range.get((r, c)) {
                    rows[abs_r][start_col + c] = cell_to_string(cell);
                }
            }
        }

        Ok(SheetData {
            name: sheet_name.to_string(),
            rows,
        })
    }

    pub fn file_name(path: &Path) -> String {
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("workbook.xlsx")
            .to_string()
    }
}

fn ensure_excel(path: &Path) -> Result<(), AppError> {
    if !path.exists() {
        return Err(AppError::FileNotFound);
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if !matches!(ext.as_str(), "xlsx" | "xls" | "xlsm" | "xlsb" | "ods") {
        return Err(AppError::UnsupportedExcel);
    }
    Ok(())
}

pub fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => {
            if f.fract() == 0.0 && f.abs() < 1e15 {
                format!("{}", *f as i64)
            } else {
                format!("{f}")
            }
        }
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(dt) => dt.to_string(),
        Data::DateTimeIso(s) | Data::DurationIso(s) => s.clone(),
        Data::Error(e) => e.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::cell_to_string;
    use calamine::Data;

    #[test]
    fn converts_common_cell_types() {
        assert_eq!(cell_to_string(&Data::Empty), "");
        assert_eq!(cell_to_string(&Data::String("姓名".into())), "姓名");
        assert_eq!(cell_to_string(&Data::Float(138.0)), "138");
        assert_eq!(cell_to_string(&Data::Bool(true)), "true");
    }
}
