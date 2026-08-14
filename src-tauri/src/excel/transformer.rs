use crate::domain::{HeaderMapping, TemplateSchema};

/// Rearranges source data rows into template column order.
pub struct ExcelTransformer;

impl ExcelTransformer {
    pub fn transform(
        source_rows: &[Vec<String>],
        mappings: &[HeaderMapping],
        template: &TemplateSchema,
    ) -> Vec<Vec<String>> {
        let mut index_by_target: Vec<Option<usize>> = vec![None; template.columns.len()];
        for mapping in mappings {
            if let Some(target) = mapping.target_column_index {
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
                        index_by_target
                            .get(col.index)
                            .and_then(|src| *src)
                            .and_then(|src_idx| row.get(src_idx).cloned())
                            .unwrap_or_default()
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
        let out = ExcelTransformer::transform(&rows, &mappings, &template());
        assert_eq!(
            out,
            vec![vec!["张三".to_string(), "13812345678".to_string()]]
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
        );
        merged.extend(ExcelTransformer::transform(
            &[vec!["13900000000".into(), "李四".into()]],
            &map_b,
            &template,
        ));
        assert_eq!(
            merged,
            vec![
                vec!["张三".to_string(), String::new()],
                vec!["李四".to_string(), "13900000000".to_string()],
            ]
        );
    }
}
