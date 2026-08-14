use crate::domain::{
    HeaderMapping, MappingSource, MatchCandidate, SourceColumn, TemplateSchema,
};
use crate::embedding::{cosine_similarity, EmbeddingProvider};
use crate::error::AppError;
use crate::mapping::alias::AliasDictionary;
use crate::mapping::greedy::{Assignment, AssignmentStrategy};
use crate::mapping::history::{history_candidate, HistoryLookup};

const EMBEDDING_THRESHOLD: f32 = 0.52;
const TOP_K: usize = 3;

pub struct HeaderMatcher<'a> {
    pub template: &'a TemplateSchema,
    pub template_embeddings: &'a [Vec<f32>],
    pub history: &'a dyn HistoryLookup,
    pub alias: &'a AliasDictionary,
    pub embedding: &'a dyn EmbeddingProvider,
}

impl<'a> HeaderMatcher<'a> {
    pub fn match_headers(&self, sources: &[SourceColumn]) -> Result<Vec<HeaderMapping>, AppError> {
        let template_cols: Vec<(usize, String, String)> = self
            .template
            .columns
            .iter()
            .map(|c| (c.index, c.name.clone(), c.normalized_name.clone()))
            .collect();

        let mut mappings = Vec::with_capacity(sources.len());
        let mut pending_embed = Vec::new();

        for source in sources {
            let mut mapping = HeaderMapping {
                source_column_index: source.index,
                source_header: source.header.clone(),
                normalized_source_header: source.normalized_header.clone(),
                target_column_index: None,
                target_header: None,
                score: None,
                source: MappingSource::Unmatched,
                candidates: Vec::new(),
            };

            if let Some(candidate) = exact_match(&source.header, &template_cols) {
                mapping.source = MappingSource::Exact;
                mapping.candidates = vec![candidate];
            } else if let Some(candidate) =
                normalized_exact_match(&source.normalized_header, &template_cols)
            {
                mapping.source = MappingSource::NormalizedExact;
                mapping.candidates = vec![candidate];
            } else if let Some(hit) = self.history.find(&source.normalized_header) {
                if let Some(candidate) = history_candidate(&template_cols, &hit) {
                    mapping.source = MappingSource::History;
                    mapping.candidates = vec![candidate];
                }
            } else if let Some(canonical) = self.alias.canonical_for(&source.normalized_header) {
                if let Some(candidate) = normalized_exact_match(canonical, &template_cols) {
                    mapping.source = MappingSource::Alias;
                    mapping.candidates = vec![candidate];
                }
            }

            if mapping.candidates.is_empty() {
                pending_embed.push(source.index);
            }
            mappings.push(mapping);
        }

        if !pending_embed.is_empty() {
            let texts: Vec<String> = pending_embed
                .iter()
                .filter_map(|idx| {
                    sources
                        .iter()
                        .find(|s| s.index == *idx)
                        .map(|s| s.normalized_header.clone())
                })
                .collect();
            let vectors = self.embedding.embed(&texts)?;
            for (source_index, vector) in pending_embed.into_iter().zip(vectors.into_iter()) {
                if let Some(mapping) = mappings
                    .iter_mut()
                    .find(|m| m.source_column_index == source_index)
                {
                    mapping.source = MappingSource::Embedding;
                    mapping.candidates = top_k_candidates(self.template, self.template_embeddings, &vector);
                }
            }
        }

        let assignments: Vec<Assignment> = mappings
            .iter()
            .filter_map(|mapping| {
                let candidate = mapping.candidates.first()?;
                let min_score = match mapping.source {
                    MappingSource::Embedding => EMBEDDING_THRESHOLD,
                    MappingSource::Unmatched => f32::MAX,
                    _ => 0.0,
                };
                if candidate.score < min_score {
                    return None;
                }
                Some(Assignment {
                    source_column_index: mapping.source_column_index,
                    target_column_index: candidate.template_column_index,
                    score: candidate.score,
                })
            })
            .collect();

        let chosen = AssignmentStrategy::greedy(&assignments);
        AssignmentStrategy::apply(&mut mappings, &chosen);
        Ok(mappings)
    }
}

fn exact_match(
    header: &str,
    template_cols: &[(usize, String, String)],
) -> Option<MatchCandidate> {
    template_cols.iter().find_map(|(idx, name, _)| {
        if name == header {
            Some(MatchCandidate {
                template_column_index: *idx,
                template_header: name.clone(),
                score: 1.0,
            })
        } else {
            None
        }
    })
}

fn normalized_exact_match(
    normalized: &str,
    template_cols: &[(usize, String, String)],
) -> Option<MatchCandidate> {
    template_cols.iter().find_map(|(idx, name, norm)| {
        if norm == normalized {
            Some(MatchCandidate {
                template_column_index: *idx,
                template_header: name.clone(),
                score: 0.99,
            })
        } else {
            None
        }
    })
}

fn top_k_candidates(
    template: &TemplateSchema,
    template_embeddings: &[Vec<f32>],
    vector: &[f32],
) -> Vec<MatchCandidate> {
    let mut scored: Vec<MatchCandidate> = template
        .columns
        .iter()
        .zip(template_embeddings.iter())
        .map(|(col, emb)| MatchCandidate {
            template_column_index: col.index,
            template_header: col.name.clone(),
            score: cosine_similarity(vector, emb),
        })
        .collect();
    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(TOP_K);
    scored
}

pub fn mapping_conflict_message(
    mappings: &[HeaderMapping],
    source_column_index: usize,
    target_column_index: usize,
    target_header: &str,
) -> Option<String> {
    mappings.iter().find_map(|mapping| {
        if mapping.source_column_index != source_column_index
            && mapping.target_column_index == Some(target_column_index)
        {
            Some(format!(
                "“{target_header}”已映射给“{}”",
                mapping.source_header
            ))
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{mapping_conflict_message, HeaderMatcher};
    use crate::domain::{
        HeaderMapping, MappingSource, SourceColumn, TemplateColumn, TemplateSchema,
    };
    use crate::embedding::MockEmbeddingProvider;
    use crate::embedding::EmbeddingProvider;
    use crate::mapping::alias::AliasDictionary;
    use crate::mapping::history::MemoryHistory;
    use crate::mapping::normalize_header;

    fn template() -> TemplateSchema {
        let names = ["姓名", "身份证号", "手机号码", "所属部门"];
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
                .map(|(i, name)| TemplateColumn {
                    index: i,
                    name: (*name).into(),
                    normalized_name: normalize_header(name),
                    path: None,
                })
                .collect(),
        }
    }

    fn sources(headers: &[&str]) -> Vec<SourceColumn> {
        headers
            .iter()
            .enumerate()
            .map(|(i, name)| SourceColumn {
                index: i,
                header: (*name).into(),
                normalized_header: normalize_header(name),
                sample_values: vec![],
            })
            .collect()
    }

    #[test]
    fn exact_and_normalized_and_alias_match() {
        let template = template();
        let embedding = MockEmbeddingProvider::default();
        let history = MemoryHistory::default();
        let alias = AliasDictionary::builtin();
        let embeddings = embedding
            .embed(
                &template
                    .columns
                    .iter()
                    .map(|c| c.normalized_name.clone())
                    .collect::<Vec<_>>(),
            )
            .unwrap();
        let matcher = HeaderMatcher {
            template: &template,
            template_embeddings: &embeddings,
            history: &history,
            alias: &alias,
            embedding: &embedding,
        };
        let mappings = matcher
            .match_headers(&sources(&["姓名", "手机号码（必填）", "联系电话"]))
            .unwrap();
        assert_eq!(mappings[0].source, MappingSource::Exact);
        assert_eq!(mappings[0].target_header.as_deref(), Some("姓名"));
        assert_eq!(mappings[1].source, MappingSource::NormalizedExact);
        assert_eq!(mappings[1].target_header.as_deref(), Some("手机号码"));
        assert_eq!(mappings[2].source, MappingSource::Unmatched);
    }

    #[test]
    fn history_match_uses_previous_confirmation() {
        let template = template();
        let embedding = MockEmbeddingProvider::default();
        let history = MemoryHistory {
            hits: vec![(
                normalize_header("组织机构"),
                crate::mapping::history::HistoryHit {
                    target_header: "所属部门".into(),
                    normalized_target_header: normalize_header("所属部门"),
                    usage_count: 3,
                },
            )],
        };
        let alias = AliasDictionary::builtin();
        let embeddings = embedding
            .embed(
                &template
                    .columns
                    .iter()
                    .map(|c| c.normalized_name.clone())
                    .collect::<Vec<_>>(),
            )
            .unwrap();
        let matcher = HeaderMatcher {
            template: &template,
            template_embeddings: &embeddings,
            history: &history,
            alias: &alias,
            embedding: &embedding,
        };
        let mappings = matcher.match_headers(&sources(&["组织机构"])).unwrap();
        assert_eq!(mappings[0].source, MappingSource::History);
        assert_eq!(mappings[0].target_header.as_deref(), Some("所属部门"));
    }

    #[test]
    fn alias_match_phone_fields() {
        let template = template();
        let embedding = MockEmbeddingProvider::default();
        let history = MemoryHistory::default();
        let alias = AliasDictionary::builtin();
        let embeddings = embedding
            .embed(
                &template
                    .columns
                    .iter()
                    .map(|c| c.normalized_name.clone())
                    .collect::<Vec<_>>(),
            )
            .unwrap();
        let matcher = HeaderMatcher {
            template: &template,
            template_embeddings: &embeddings,
            history: &history,
            alias: &alias,
            embedding: &embedding,
        };
        let mappings = matcher.match_headers(&sources(&["联系电话"])).unwrap();
        assert_eq!(mappings[0].source, MappingSource::Alias);
        assert_eq!(mappings[0].target_header.as_deref(), Some("手机号码"));
    }

    #[test]
    fn reports_mapping_conflict() {
        let mappings = vec![
            HeaderMapping {
                source_column_index: 0,
                source_header: "联系电话".into(),
                normalized_source_header: "联系电话".into(),
                target_column_index: Some(2),
                target_header: Some("手机号码".into()),
                score: Some(1.0),
                source: MappingSource::Alias,
                candidates: vec![],
            },
            HeaderMapping {
                source_column_index: 1,
                source_header: "手机".into(),
                normalized_source_header: "手机".into(),
                target_column_index: None,
                target_header: None,
                score: None,
                source: MappingSource::Unmatched,
                candidates: vec![],
            },
        ];
        let msg = mapping_conflict_message(&mappings, 1, 2, "手机号码").unwrap();
        assert!(msg.contains("手机号码"));
        assert!(msg.contains("联系电话"));
    }
}
