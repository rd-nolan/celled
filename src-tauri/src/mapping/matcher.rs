use std::collections::{HashMap, HashSet};

use crate::domain::{HeaderMapping, MappingSource, MatchCandidate, SourceColumn, TemplateSchema};
use crate::embedding::{header_similarity, EmbeddingProvider};
use crate::error::AppError;
use crate::mapping::alias::AliasDictionary;
use crate::mapping::content::{
    combo_text, content_candidates, detect_kind, first_sample, ContentKind, CONTENT_THRESHOLD,
};
use crate::mapping::greedy::{Assignment, AssignmentStrategy};
use crate::mapping::history::{history_candidate, HistoryLookup};

const EMBEDDING_THRESHOLD: f32 = 0.52;
const TOP_K: usize = 3;

/// Match order: Exact → Normalized Exact → History → Alias → header Embedding
/// → first-row content inference. Content never steals a template column
/// already taken by a higher-priority header match.

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
                    mapping.candidates = top_k_candidates(
                        self.template,
                        self.template_embeddings,
                        &mapping.normalized_source_header,
                        &vector,
                    );
                }
            }
        }

        let assignments: Vec<Assignment> = mappings
            .iter()
            .filter_map(|mapping| {
                let candidate = mapping.candidates.first()?;
                let min_score = match mapping.source {
                    MappingSource::Embedding => EMBEDDING_THRESHOLD,
                    MappingSource::Content => CONTENT_THRESHOLD,
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
        self.apply_content_inference(sources, &mut mappings)?;
        Ok(mappings)
    }

    fn apply_content_inference(
        &self,
        sources: &[SourceColumn],
        mappings: &mut [HeaderMapping],
    ) -> Result<(), AppError> {
        let taken: HashSet<usize> = mappings
            .iter()
            .filter_map(|mapping| mapping.target_column_index)
            .collect();
        let free_targets: Vec<usize> = self
            .template
            .columns
            .iter()
            .map(|col| col.index)
            .filter(|index| !taken.contains(index))
            .collect();
        if free_targets.is_empty() {
            return Ok(());
        }

        let pending: Vec<&SourceColumn> = sources
            .iter()
            .filter(|source| {
                mappings.iter().any(|mapping| {
                    mapping.source_column_index == source.index
                        && mapping.source == MappingSource::Unmatched
                        && mapping.target_column_index.is_none()
                        && first_sample(source).is_some()
                })
            })
            .collect();
        if pending.is_empty() {
            return Ok(());
        }

        let mut texts = Vec::new();
        let mut text_index: HashMap<String, usize> = HashMap::new();
        let mut intern = |text: String| -> usize {
            if let Some(index) = text_index.get(&text) {
                return *index;
            }
            let index = texts.len();
            text_index.insert(text.clone(), index);
            texts.push(text);
            index
        };

        let mut jobs = Vec::new();
        for source in &pending {
            let Some(sample) = first_sample(source) else {
                continue;
            };
            let sample_idx = intern(sample.to_string());
            let combo_idx = combo_text(&source.header, sample).map(|text| intern(text));
            let kind = detect_kind(sample);
            let token_idxs: Vec<usize> = kind
                .map(|k| {
                    k.type_tokens()
                        .iter()
                        .map(|token| intern((*token).to_string()))
                        .collect()
                })
                .unwrap_or_default();
            jobs.push((source.index, sample_idx, combo_idx, kind, token_idxs));
        }

        let vectors = self.embedding.embed(&texts)?;
        let mut content_assignments = Vec::new();
        for (source_index, sample_idx, combo_idx, kind, token_idxs) in jobs {
            let Some(mapping) = mappings
                .iter_mut()
                .find(|m| m.source_column_index == source_index)
            else {
                continue;
            };
            let combo_vec = combo_idx.and_then(|idx| vectors.get(idx).map(|v| v.as_slice()));
            let tokens: &[&str] = kind.map(ContentKind::type_tokens).unwrap_or(&[]);
            let token_vecs: Vec<&[f32]> = token_idxs
                .iter()
                .filter_map(|idx| vectors.get(*idx).map(|v| v.as_slice()))
                .collect();
            let candidates = content_candidates(
                self.template,
                self.template_embeddings,
                &free_targets,
                &vectors[sample_idx],
                combo_vec,
                tokens,
                &token_vecs,
            );
            let Some(best) = candidates.first() else {
                continue;
            };
            if best.score < CONTENT_THRESHOLD {
                continue;
            }
            let assignment = Assignment {
                source_column_index: source_index,
                target_column_index: best.template_column_index,
                score: best.score,
            };
            mapping.source = MappingSource::Content;
            mapping.candidates = candidates;
            content_assignments.push(assignment);
        }

        let chosen = AssignmentStrategy::greedy(&content_assignments);
        apply_content_assignments(mappings, &chosen);
        Ok(())
    }
}

fn apply_content_assignments(mappings: &mut [HeaderMapping], chosen: &[Assignment]) {
    for mapping in mappings.iter_mut() {
        if mapping.source != MappingSource::Content {
            continue;
        }
        if let Some(found) = chosen
            .iter()
            .find(|a| a.source_column_index == mapping.source_column_index)
        {
            mapping.target_column_index = Some(found.target_column_index);
            if let Some(candidate) = mapping
                .candidates
                .iter()
                .find(|c| c.template_column_index == found.target_column_index)
            {
                mapping.target_header = Some(candidate.template_header.clone());
                mapping.score = Some(found.score);
            }
        } else {
            mapping.target_column_index = None;
            mapping.target_header = None;
            mapping.score = None;
            mapping.source = MappingSource::Unmatched;
        }
    }
}

fn exact_match(header: &str, template_cols: &[(usize, String, String)]) -> Option<MatchCandidate> {
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
    source_text: &str,
    vector: &[f32],
) -> Vec<MatchCandidate> {
    let corpus: Vec<String> = template
        .columns
        .iter()
        .map(|col| col.normalized_name.clone())
        .collect();
    let mut scored: Vec<MatchCandidate> = template
        .columns
        .iter()
        .zip(template_embeddings.iter())
        .map(|(col, emb)| MatchCandidate {
            template_column_index: col.index,
            template_header: col.name.clone(),
            score: header_similarity(source_text, vector, &col.normalized_name, emb, &corpus),
        })
        .collect();
    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
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
    use crate::embedding::EmbeddingProvider;
    use crate::embedding::MockEmbeddingProvider;
    use crate::mapping::alias::AliasDictionary;
    use crate::mapping::history::MemoryHistory;
    use crate::mapping::normalize_header;

    fn template() -> TemplateSchema {
        template_from_names(&["姓名", "身份证号", "手机号码", "所属部门"])
    }

    fn template_from_names(names: &[&str]) -> TemplateSchema {
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

    fn sources_with_samples(rows: &[(&str, &[&str])]) -> Vec<SourceColumn> {
        rows.iter()
            .enumerate()
            .map(|(i, (header, samples))| SourceColumn {
                index: i,
                header: (*header).into(),
                normalized_header: normalize_header(header),
                sample_values: samples.iter().map(|s| (*s).to_string()).collect(),
            })
            .collect()
    }

    fn match_sources(
        template: &TemplateSchema,
        sources: &[SourceColumn],
        alias: AliasDictionary,
    ) -> Vec<HeaderMapping> {
        let embedding = MockEmbeddingProvider::default();
        let history = MemoryHistory::default();
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
            template,
            template_embeddings: &embeddings,
            history: &history,
            alias: &alias,
            embedding: &embedding,
        };
        matcher.match_headers(sources).unwrap()
    }

    fn match_with(
        template: &TemplateSchema,
        headers: &[&str],
        alias: AliasDictionary,
    ) -> Vec<HeaderMapping> {
        match_sources(template, &sources(headers), alias)
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

    #[test]
    fn embedding_matches_student_id_headers_without_alias() {
        let alias = AliasDictionary::default();
        assert!(alias.canonical_for("学号").is_none());
        assert!(alias.canonical_for("学生编号").is_none());

        let template_xuehao = template_from_names(&["姓名", "学号", "班级", "手机号码"]);
        let mappings = match_with(&template_xuehao, &["学生编号"], alias.clone());
        assert_eq!(mappings[0].source, MappingSource::Embedding);
        assert_eq!(mappings[0].target_header.as_deref(), Some("学号"));
        assert!(
            mappings[0].score.unwrap() >= 0.52,
            "expected embedding score above threshold, got {:?}",
            mappings[0].score
        );

        let template_bianhao = template_from_names(&["姓名", "学生编号", "班级"]);
        let mappings = match_with(&template_bianhao, &["学号"], alias);
        assert_eq!(mappings[0].source, MappingSource::Embedding);
        assert_eq!(mappings[0].target_header.as_deref(), Some("学生编号"));
    }

    #[test]
    fn content_infers_email_and_date_from_first_row() {
        let template = template_from_names(&["邮箱", "日期", "姓名"]);
        let sources = sources_with_samples(&[("A", &["a@b.com"]), ("B", &["2024-01-02"])]);
        let mappings = match_sources(&template, &sources, AliasDictionary::default());
        assert_eq!(mappings[0].source, MappingSource::Content);
        assert_eq!(mappings[0].target_header.as_deref(), Some("邮箱"));
        assert_eq!(mappings[1].source, MappingSource::Content);
        assert_eq!(mappings[1].target_header.as_deref(), Some("日期"));
    }

    #[test]
    fn content_uses_first_non_empty_sample() {
        let template = template_from_names(&["邮箱", "姓名"]);
        let sources = sources_with_samples(&[("col1", &["", "  ", "a@b.com"])]);
        let mappings = match_sources(&template, &sources, AliasDictionary::default());
        assert_eq!(mappings[0].source, MappingSource::Content);
        assert_eq!(mappings[0].target_header.as_deref(), Some("邮箱"));
    }

    #[test]
    fn content_does_not_override_exact_header_match() {
        let template = template_from_names(&["日期", "邮箱"]);
        let sources = sources_with_samples(&[("邮箱", &["2024-01-02"])]);
        let mappings = match_sources(&template, &sources, AliasDictionary::default());
        assert_eq!(mappings[0].source, MappingSource::Exact);
        assert_eq!(mappings[0].target_header.as_deref(), Some("邮箱"));
    }

    #[test]
    fn content_does_not_steal_taken_template_column() {
        let template = template_from_names(&["邮箱", "姓名"]);
        let sources = sources_with_samples(&[("邮箱", &["ignored"]), ("A", &["a@b.com"])]);
        let mappings = match_sources(&template, &sources, AliasDictionary::default());
        assert_eq!(mappings[0].source, MappingSource::Exact);
        assert_eq!(mappings[0].target_header.as_deref(), Some("邮箱"));
        assert_eq!(mappings[1].source, MappingSource::Unmatched);
        assert!(mappings[1].target_header.is_none());
    }

    #[test]
    fn content_ignores_unrelated_numbers() {
        let template = template_from_names(&["姓名", "邮箱"]);
        let sources = sources_with_samples(&[("col1", &["42"])]);
        let mappings = match_sources(&template, &sources, AliasDictionary::default());
        assert_eq!(mappings[0].source, MappingSource::Unmatched);
        assert!(mappings[0].target_header.is_none());
    }

    #[test]
    fn embedding_header_match_is_not_overridden_by_content() {
        let template = template_from_names(&["学号", "邮箱"]);
        let sources = sources_with_samples(&[("学生编号", &["a@b.com"])]);
        let mappings = match_sources(&template, &sources, AliasDictionary::default());
        assert_eq!(mappings[0].source, MappingSource::Embedding);
        assert_eq!(mappings[0].target_header.as_deref(), Some("学号"));
    }
}
