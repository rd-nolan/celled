use crate::domain::MatchCandidate;

#[derive(Debug, Clone)]
pub struct HistoryHit {
    pub target_header: String,
    pub normalized_target_header: String,
    pub usage_count: i64,
}

pub trait HistoryLookup: Send + Sync {
    fn find(&self, normalized_source: &str) -> Option<HistoryHit>;
}

#[derive(Debug, Default)]
pub struct MemoryHistory {
    pub hits: Vec<(String, HistoryHit)>,
}

impl HistoryLookup for MemoryHistory {
    fn find(&self, normalized_source: &str) -> Option<HistoryHit> {
        self.hits
            .iter()
            .filter(|(source, _)| source == normalized_source)
            .max_by_key(|(_, hit)| hit.usage_count)
            .map(|(_, hit)| hit.clone())
    }
}

pub fn history_candidate(
    template_columns: &[(usize, String, String)],
    hit: &HistoryHit,
) -> Option<MatchCandidate> {
    template_columns.iter().find_map(|(idx, name, normalized)| {
        if normalized == &hit.normalized_target_header || name == &hit.target_header {
            Some(MatchCandidate {
                template_column_index: *idx,
                template_header: name.clone(),
                score: 0.97,
            })
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{HistoryHit, HistoryLookup, MemoryHistory};

    #[test]
    fn prefers_highest_usage_count() {
        let history = MemoryHistory {
            hits: vec![
                (
                    "组织机构".into(),
                    HistoryHit {
                        target_header: "单位名称".into(),
                        normalized_target_header: "单位名称".into(),
                        usage_count: 1,
                    },
                ),
                (
                    "组织机构".into(),
                    HistoryHit {
                        target_header: "所属单位".into(),
                        normalized_target_header: "所属单位".into(),
                        usage_count: 5,
                    },
                ),
            ],
        };
        let hit = history.find("组织机构").unwrap();
        assert_eq!(hit.target_header, "所属单位");
    }
}
