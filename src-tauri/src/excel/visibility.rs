use std::collections::HashSet;
use std::io::Read;
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;
use zip::ZipArchive;

use crate::error::AppError;

/// Returns 1-indexed Excel row numbers marked hidden in the worksheet XML.
pub fn hidden_rows_for_sheet(path: &Path, sheet_name: &str) -> Result<HashSet<usize>, AppError> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if !matches!(ext.as_str(), "xlsx" | "xlsm" | "xlsb") {
        return Ok(HashSet::new());
    }

    let file = std::fs::File::open(path)?;
    let mut archive =
        ZipArchive::new(file).map_err(|e| AppError::ExcelRead(format!("无法打开 xlsx：{e}")))?;

    let workbook_xml = read_zip_entry(&mut archive, "xl/workbook.xml")?;
    let rels_xml = read_zip_entry(&mut archive, "xl/_rels/workbook.xml.rels")?;
    let sheet_rid = sheet_relationship_id(&workbook_xml, sheet_name)?;
    let sheet_target = relationship_target(&rels_xml, &sheet_rid)?;
    let sheet_path = normalize_sheet_path(&sheet_target);
    let sheet_xml = read_zip_entry(&mut archive, &sheet_path)?;
    Ok(parse_hidden_rows(&sheet_xml))
}

fn read_zip_entry<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    name: &str,
) -> Result<String, AppError> {
    let mut file = archive
        .by_name(name)
        .map_err(|_| AppError::ExcelRead(format!("找不到 {name}")))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| AppError::ExcelRead(e.to_string()))?;
    Ok(contents)
}

fn sheet_relationship_id(workbook_xml: &str, sheet_name: &str) -> Result<String, AppError> {
    let mut reader = Reader::from_str(workbook_xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut in_sheets = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let local = local_name(&e);
                if local == "sheets" {
                    in_sheets = true;
                } else if in_sheets && local == "sheet" {
                    let name = attr_value(&e, b"name").unwrap_or_default();
                    if name == sheet_name {
                        return attr_value_by_local(&e, "id")
                            .ok_or_else(|| AppError::SheetNotFound);
                    }
                }
            }
            Ok(Event::End(e)) => {
                if end_local_name(&e) == "sheets" {
                    in_sheets = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(AppError::ExcelRead(e.to_string())),
            _ => {}
        }
        buf.clear();
    }

    Err(AppError::SheetNotFound)
}

fn relationship_target(rels_xml: &str, relationship_id: &str) -> Result<String, AppError> {
    let mut reader = Reader::from_str(rels_xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(e)) | Ok(Event::Start(e)) => {
                if local_name(&e) != "Relationship" {
                    buf.clear();
                    continue;
                }
                let id = attr_value(&e, b"Id").unwrap_or_default();
                if id == relationship_id {
                    return attr_value(&e, b"Target")
                        .ok_or_else(|| AppError::ExcelRead("缺少工作表路径".into()));
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(AppError::ExcelRead(e.to_string())),
            _ => {}
        }
        buf.clear();
    }

    Err(AppError::SheetNotFound)
}

fn normalize_sheet_path(target: &str) -> String {
    if target.starts_with('/') {
        target.trim_start_matches('/').to_string()
    } else {
        format!("xl/{target}")
    }
}

fn parse_hidden_rows(sheet_xml: &str) -> HashSet<usize> {
    let mut reader = Reader::from_str(sheet_xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut hidden = HashSet::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                if local_name(&e) != "row" {
                    buf.clear();
                    continue;
                }
                if !is_hidden(&e) {
                    buf.clear();
                    continue;
                }
                if let Some(row_number) = attr_value(&e, b"r").and_then(|r| r.parse().ok()) {
                    hidden.insert(row_number);
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    hidden
}

fn is_hidden(element: &quick_xml::events::BytesStart<'_>) -> bool {
    attr_value(element, b"hidden").is_some_and(|value| {
        value == "1" || value.eq_ignore_ascii_case("true")
    })
}

fn local_name(element: &quick_xml::events::BytesStart<'_>) -> String {
    String::from_utf8_lossy(element.local_name().as_ref()).into_owned()
}

fn end_local_name(element: &quick_xml::events::BytesEnd<'_>) -> String {
    String::from_utf8_lossy(element.local_name().as_ref()).into_owned()
}

fn attr_value(element: &quick_xml::events::BytesStart<'_>, key: &[u8]) -> Option<String> {
    element
        .attributes()
        .flatten()
        .find(|attr| attr.key.as_ref() == key)
        .map(|attr| String::from_utf8_lossy(&attr.value).into_owned())
}

fn attr_value_by_local(element: &quick_xml::events::BytesStart<'_>, local: &str) -> Option<String> {
    element.attributes().flatten().find_map(|attr| {
        let key = String::from_utf8_lossy(attr.key.as_ref());
        if key == local || key.ends_with(&format!(":{local}")) {
            Some(String::from_utf8_lossy(&attr.value).into_owned())
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hidden_row_numbers() {
        let xml = r#"
            <worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
              <sheetData>
                <row r="1"><c/></row>
                <row r="2" hidden="1"><c/></row>
                <row r="3"><c/></row>
              </sheetData>
            </worksheet>
        "#;
        let hidden = parse_hidden_rows(xml);
        assert!(hidden.contains(&2));
        assert_eq!(hidden.len(), 1);
    }
}
