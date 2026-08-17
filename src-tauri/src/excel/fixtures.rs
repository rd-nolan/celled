use rust_xlsxwriter::Workbook;
use std::path::Path;

use crate::error::AppError;

pub fn write_demo_fixtures(dir: &Path) -> Result<(), AppError> {
    std::fs::create_dir_all(dir)?;
    write_simple(
        &dir.join("template.xlsx"),
        "人员信息",
        &[
            &["2026 年员工信息导出表"],
            &["姓名", "身份证号", "手机号码", "所属部门"],
            &["示例", "110101199001011234", "13800000000", "技术部"],
        ],
    )?;
    write_simple(
        &dir.join("data-header-row-1.xlsx"),
        "Sheet1",
        &[
            &["用户姓名", "证件号码", "联系电话", "部门名称"],
            &["张三", "110101199001011234", "13812345678", "技术部"],
            &["李四", "110101199002022345", "13912345678", "产品部"],
        ],
    )?;
    write_simple(
        &dir.join("data-header-row-2.xlsx"),
        "Sheet1",
        &[
            &["2026 年人员信息表"],
            &["用户姓名", "证件号码", "联系电话", "部门名称"],
            &["王五", "110101199003033456", "13712345678", "设计部"],
        ],
    )?;
    write_simple(
        &dir.join("data-header-row-3.xlsx"),
        "Sheet1",
        &[
            &["某某公司"],
            &["2026 年人员导入数据"],
            &["用户姓名", "证件号码", "联系电话", "部门名称"],
            &["赵六", "110101199004044567", "13612345678", "运营部"],
        ],
    )?;
    write_simple(
        &dir.join("data-different-names.xlsx"),
        "Sheet1",
        &[
            &["员工姓名", "身份证号码", "移动电话", "所在部门"],
            &["钱七", "110101199005055678", "13512345678", "财务部"],
        ],
    )?;
    Ok(())
}

pub fn write_simple(path: &Path, sheet_name: &str, rows: &[&[&str]]) -> Result<(), AppError> {
    let mut workbook = Workbook::new();
    let worksheet = workbook
        .add_worksheet()
        .set_name(sheet_name)
        .map_err(|e| AppError::ExcelWriteError(e.to_string()))?;
    for (r, row) in rows.iter().enumerate() {
        for (c, value) in row.iter().enumerate() {
            worksheet
                .write_string(r as u32, c as u16, *value)
                .map_err(|e| AppError::ExcelWriteError(e.to_string()))?;
        }
    }
    workbook
        .save(path)
        .map_err(|e| AppError::ExcelWriteError(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::write_demo_fixtures;
    use crate::excel::{ExcelReader, HeaderDetector};
    use tempfile::tempdir;

    #[test]
    fn writes_repo_fixtures() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures");
        write_demo_fixtures(&dir).unwrap();
        assert!(dir.join("template.xlsx").exists());
    }

    #[test]
    fn fixture_header_rows_match_expectations() {
        let dir = tempdir().unwrap();
        write_demo_fixtures(dir.path()).unwrap();

        let cases = [
            ("template.xlsx", 2),
            ("data-header-row-1.xlsx", 1),
            ("data-header-row-2.xlsx", 2),
            ("data-header-row-3.xlsx", 3),
        ];
        for (name, expected) in cases {
            let path = dir.path().join(name);
            let sheet = ExcelReader::first_non_empty_sheet(&path).unwrap();
            let data = ExcelReader::read_sheet(&path, &sheet, Some(16), false).unwrap();
            let detected = HeaderDetector::detect(&data.rows, Some(&data), false);
            assert_eq!(detected.row_index, expected, "{name}");
        }
    }
}
