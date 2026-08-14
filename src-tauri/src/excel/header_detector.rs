use crate::domain::HeaderDetectionResult;

const SCAN_ROWS: usize = 15;

/// Heuristic header-row detector. Does not use embedding or LLM.
pub struct HeaderDetector;

impl HeaderDetector {
    /// Detect the most likely header row from sheet rows.
    /// `rows[0]` is Excel row 1. Returned `row_index` is 1-indexed.
    pub fn detect(rows: &[Vec<String>]) -> HeaderDetectionResult {
        let scan_len = rows.len().min(SCAN_ROWS);
        if scan_len == 0 {
            return HeaderDetectionResult {
                row_index: 1,
                confidence: 0.0,
                headers: Vec::new(),
            };
        }

        let mut best_idx = 0usize;
        let mut best_score = f32::MIN;

        for i in 0..scan_len {
            let score = score_row(rows, i, scan_len);
            if score > best_score {
                best_score = score;
                best_idx = i;
            }
        }

        let headers = nonempty_cells(&rows[best_idx]);
        let confidence = (best_score / 12.0).clamp(0.05, 0.99);

        HeaderDetectionResult {
            row_index: best_idx + 1,
            confidence,
            headers,
        }
    }
}

fn score_row(rows: &[Vec<String>], index: usize, scan_len: usize) -> f32 {
    let row = &rows[index];
    let nonempty = nonempty_cells(row);
    if nonempty.is_empty() {
        return -5.0;
    }

    let next = rows.get(index + 1).map(|r| r.as_slice());
    let non_empty_ratio = nonempty.len() as f32 / row.len().max(1) as f32;
    let text_ratio = text_cell_ratio(&nonempty);
    let unique_ratio = unique_ratio(&nonempty);
    let consecutive = consecutive_non_empty(row) as f32;
    let type_diff = next.map(|n| type_difference(row, n)).unwrap_or(0.0);
    let data_below = data_block_score(rows, index, scan_len);
    let extra_cols = extra_column_bonus(rows, index, scan_len);

    let mut score = 0.0;
    score += non_empty_ratio * 2.0;
    score += text_ratio * 2.4;
    score += unique_ratio * 1.6;
    score += (consecutive / 8.0).min(1.0) * 1.5;
    score += type_diff * 2.2;
    score += data_below * 2.0;
    score += extra_cols;

    if looks_like_title(row, rows, index, scan_len) {
        score -= 4.0;
    }
    if nonempty.len() <= 1 {
        score -= 2.2;
    }
    if text_ratio < 0.45 {
        score -= 1.8;
    }

    score
}

fn nonempty_cells(row: &[String]) -> Vec<String> {
    row.iter()
        .map(|c| c.trim().to_string())
        .filter(|c| !c.is_empty())
        .collect()
}

fn consecutive_non_empty(row: &[String]) -> usize {
    let mut best = 0usize;
    let mut current = 0usize;
    for cell in row {
        if cell.trim().is_empty() {
            best = best.max(current);
            current = 0;
        } else {
            current += 1;
        }
    }
    best.max(current)
}

fn unique_ratio(cells: &[String]) -> f32 {
    if cells.is_empty() {
        return 0.0;
    }
    let mut seen = Vec::new();
    for cell in cells {
        if !seen.contains(cell) {
            seen.push(cell.clone());
        }
    }
    seen.len() as f32 / cells.len() as f32
}

fn text_cell_ratio(cells: &[String]) -> f32 {
    if cells.is_empty() {
        return 0.0;
    }
    let text_count = cells.iter().filter(|c| !is_numeric_like(c)).count();
    text_count as f32 / cells.len() as f32
}

fn is_numeric_like(value: &str) -> bool {
    let t = value.trim();
    if t.is_empty() {
        return false;
    }
    if t.parse::<f64>().is_ok() {
        return true;
    }
    let chars: Vec<char> = t.chars().collect();
    let digits = chars.iter().filter(|c| c.is_ascii_digit()).count();
    digits as f32 / chars.len() as f32 > 0.55
}

fn type_difference(current: &[String], next: &[String]) -> f32 {
    let width = current.len().max(next.len());
    if width == 0 {
        return 0.0;
    }
    let mut hits = 0.0;
    let mut compared = 0.0;
    for i in 0..width {
        let a = current.get(i).map(|s| s.trim()).unwrap_or("");
        let b = next.get(i).map(|s| s.trim()).unwrap_or("");
        if a.is_empty() && b.is_empty() {
            continue;
        }
        compared += 1.0;
        if !a.is_empty() && !is_numeric_like(a) && is_numeric_like(b) {
            hits += 1.0;
        }
    }
    if compared == 0.0 {
        0.0
    } else {
        hits / compared
    }
}

fn data_block_score(rows: &[Vec<String>], index: usize, scan_len: usize) -> f32 {
    let header_width = nonempty_cells(&rows[index]).len();
    if header_width == 0 {
        return 0.0;
    }
    let mut similar = 0usize;
    for row in rows.iter().take(scan_len).skip(index + 1).take(4) {
        let width = nonempty_cells(row).len();
        if width >= header_width.saturating_sub(1) && width > 0 {
            similar += 1;
        }
    }
    (similar as f32 / 3.0).min(1.0)
}

fn extra_column_bonus(rows: &[Vec<String>], index: usize, scan_len: usize) -> f32 {
    let current = nonempty_cells(&rows[index]).len();
    let mut neighbor_max = 0usize;
    if index > 0 {
        neighbor_max = neighbor_max.max(nonempty_cells(&rows[index - 1]).len());
    }
    if index + 1 < scan_len {
        neighbor_max = neighbor_max.max(nonempty_cells(&rows[index + 1]).len());
    }
    if current > neighbor_max && current >= 3 {
        1.4
    } else if current + 1 >= neighbor_max && current >= 2 {
        0.4
    } else {
        0.0
    }
}

fn looks_like_title(row: &[String], rows: &[Vec<String>], index: usize, scan_len: usize) -> bool {
    let nonempty = nonempty_cells(row);
    if nonempty.is_empty() || nonempty.len() > 2 {
        return false;
    }

    let later_max = rows
        .iter()
        .take(scan_len)
        .skip(index + 1)
        .take(3)
        .map(|r| nonempty_cells(r).len())
        .max()
        .unwrap_or(0);

    if later_max >= nonempty.len() + 2 {
        return true;
    }

    nonempty.len() == 1 && nonempty[0].chars().count() >= 8
}

#[cfg(test)]
mod tests {
    use super::HeaderDetector;

    fn row(cells: &[&str]) -> Vec<String> {
        cells.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn detects_first_row_headers() {
        let rows = vec![row(&["姓名", "手机号"]), row(&["张三", "13812345678"])];
        let result = HeaderDetector::detect(&rows);
        assert_eq!(result.row_index, 1, "expected row 1, got {:?}", result);
    }

    #[test]
    fn skips_single_title_row() {
        let rows = vec![
            row(&["2026 年人员信息表"]),
            row(&["姓名", "手机号"]),
            row(&["张三", "13812345678"]),
        ];
        let result = HeaderDetector::detect(&rows);
        assert_eq!(result.row_index, 2, "expected row 2, got {:?}", result);
    }

    #[test]
    fn skips_two_title_rows() {
        let rows = vec![
            row(&["某某公司"]),
            row(&["2026 年人员导入数据"]),
            row(&["姓名", "手机号", "部门"]),
            row(&["张三", "13812345678", "技术部"]),
        ];
        let result = HeaderDetector::detect(&rows);
        assert_eq!(result.row_index, 3, "expected row 3, got {:?}", result);
    }
}
