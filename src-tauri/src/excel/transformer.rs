use crate::domain::{HeaderMapping, TemplateSchema};

/// Rearranges source data rows into template column order and fills 来源.
pub struct ExcelTransformer;

impl ExcelTransformer {
    pub fn transform(
        source_rows: &[Vec<String>],
        mappings: &[HeaderMapping],
        template: &TemplateSchema,
        source_label: &str,
    ) -> Vec<Vec<String>> {
        let template = template.with_source_column();
        let source_col = template.source_column_index();
        let width = template
            .columns
            .iter()
            .map(|col| col.index + 1)
            .max()
            .unwrap_or(0);
        let mut index_by_target: Vec<Option<usize>> = vec![None; width];
        for mapping in mappings {
            if let Some(target) = mapping.target_column_index {
                if source_col == Some(target) {
                    continue;
                }
                if let Some(slot) = index_by_target.get_mut(target) {
                    *slot = Some(mapping.source_column_index);
                }
            }
        }

        source_rows
            .iter()
            .map(|row| {
                template
                    .columns
                    .iter()
                    .map(|col| {
                        if source_col == Some(col.index) {
                            source_label.to_string()
                        } else {
                            index_by_target
                                .get(col.index)
                                .and_then(|src| *src)
                                .and_then(|src_idx| row.get(src_idx).cloned())
                                .unwrap_or_default()
                        }
                    })
                    .collect()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::ExcelTransformer;
    use crate::domain::{HeaderMapping, MappingSource, TemplateColumn, TemplateSchema};

    fn template() -> TemplateSchema {
        TemplateSchema {
            id: "t1".into(),
            file_name: "t.xlsx".into(),
            file_path: "t.xlsx".into(),
            sheet_name: "Sheet1".into(),
            header_start_row: 1,
            header_end_row: 1,
            data_start_row: 2,
            columns: vec![
                TemplateColumn {
                    index: 0,
                    name: "姓名".into(),
                    normalized_name: "姓名".into(),
                    path: None,
                },
                TemplateColumn {
                    index: 1,
                    name: "手机号码".into(),
                    normalized_name: "手机号码".into(),
                    path: None,
                },
            ],
        }
    }

    fn mapping(
        source_index: usize,
        source_header: &str,
        target_index: usize,
        target_header: &str,
    ) -> HeaderMapping {
        HeaderMapping {
            source_column_index: source_index,
            source_header: source_header.into(),
            normalized_source_header: source_header.into(),
            target_column_index: Some(target_index),
            target_header: Some(target_header.into()),
            score: Some(1.0),
            source: MappingSource::Exact,
            candidates: vec![],
        }
    }

    #[test]
    fn rearranges_columns_to_template_order() {
        let mappings = vec![
            HeaderMapping {
                source_column_index: 0,
                source_header: "联系电话".into(),
                normalized_source_header: "联系电话".into(),
                target_column_index: Some(1),
                target_header: Some("手机号码".into()),
                score: Some(1.0),
                source: MappingSource::Alias,
                candidates: vec![],
            },
            HeaderMapping {
                source_column_index: 1,
                source_header: "姓名".into(),
                normalized_source_header: "姓名".into(),
                target_column_index: Some(0),
                target_header: Some("姓名".into()),
                score: Some(1.0),
                source: MappingSource::Exact,
                candidates: vec![],
            },
        ];
        let rows = vec![vec!["13812345678".into(), "张三".into()]];
        let out = ExcelTransformer::transform(&rows, &mappings, &template(), "data.xlsx");
        assert_eq!(
            out,
            vec![vec![
                "张三".to_string(),
                "13812345678".to_string(),
                "data.xlsx".to_string()
            ]]
        );
    }

    #[test]
    fn appends_rows_from_two_sources_into_template_columns() {
        let template = template();
        let map_a = vec![HeaderMapping {
            source_column_index: 0,
            source_header: "用户姓名".into(),
            normalized_source_header: "用户姓名".into(),
            target_column_index: Some(0),
            target_header: Some("姓名".into()),
            score: Some(1.0),
            source: MappingSource::Alias,
            candidates: vec![],
        }];
        let map_b = vec![
            HeaderMapping {
                source_column_index: 1,
                source_header: "姓名".into(),
                normalized_source_header: "姓名".into(),
                target_column_index: Some(0),
                target_header: Some("姓名".into()),
                score: Some(1.0),
                source: MappingSource::Exact,
                candidates: vec![],
            },
            HeaderMapping {
                source_column_index: 0,
                source_header: "电话".into(),
                normalized_source_header: "电话".into(),
                target_column_index: Some(1),
                target_header: Some("手机号码".into()),
                score: Some(1.0),
                source: MappingSource::Alias,
                candidates: vec![],
            },
        ];
        let mut merged = ExcelTransformer::transform(
            &[vec!["张三".into(), "ignored".into()]],
            &map_a,
            &template,
            "a.xlsx",
        );
        merged.extend(ExcelTransformer::transform(
            &[vec!["13900000000".into(), "李四".into()]],
            &map_b,
            &template,
            "b.xlsx",
        ));
        assert_eq!(
            merged,
            vec![
                vec!["张三".to_string(), String::new(), "a.xlsx".to_string()],
                vec![
                    "李四".to_string(),
                    "13900000000".to_string(),
                    "b.xlsx".to_string()
                ],
            ]
        );
    }

    #[test]
    fn fills_existing_source_column_instead_of_duplicating() {
        let mut template = template();
        template.columns.push(TemplateColumn {
            index: 2,
            name: "来源文件".into(),
            normalized_name: "来源文件".into(),
            path: None,
        });
        let mappings = vec![
            mapping(0, "姓名", 0, "姓名"),
            mapping(1, "备注", 2, "来源文件"),
        ];
        let out = ExcelTransformer::transform(
            &[vec!["张三".into(), "should-not-copy".into()]],
            &mappings,
            &template,
            "data-a.xlsx",
        );
        assert_eq!(
            out,
            vec![vec![
                "张三".to_string(),
                String::new(),
                "data-a.xlsx".to_string()
            ]]
        );
    }
}
