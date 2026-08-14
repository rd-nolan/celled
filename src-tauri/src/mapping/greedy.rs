use crate::domain::{HeaderMapping, MappingSource};

#[derive(Debug, Clone)]
pub struct Assignment {
    pub source_column_index: usize,
    pub target_column_index: usize,
    pub score: f32,
}

/// Greedy one-to-one assignment. Replaceable later with Hungarian algorithm.
pub struct AssignmentStrategy;

impl AssignmentStrategy {
    pub fn greedy(candidates: &[Assignment]) -> Vec<Assignment> {
        let mut ranked = candidates.to_vec();
        ranked.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut used_source = Vec::new();
        let mut used_target = Vec::new();
        let mut chosen = Vec::new();

        for item in ranked {
            if used_source.contains(&item.source_column_index)
                || used_target.contains(&item.target_column_index)
            {
                continue;
            }
            used_source.push(item.source_column_index);
            used_target.push(item.target_column_index);
            chosen.push(item);
        }
        chosen
    }

    pub fn apply(mappings: &mut [HeaderMapping], assignments: &[Assignment]) {
        for mapping in mappings.iter_mut() {
            if let Some(found) = assignments
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
            } else if mapping.source != MappingSource::Manual {
                mapping.target_column_index = None;
                mapping.target_header = None;
                mapping.score = None;
                mapping.source = MappingSource::Unmatched;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Assignment, AssignmentStrategy};

    #[test]
    fn greedy_avoids_duplicate_targets() {
        let chosen = AssignmentStrategy::greedy(&[
            Assignment {
                source_column_index: 0,
                target_column_index: 1,
                score: 0.91,
            },
            Assignment {
                source_column_index: 1,
                target_column_index: 1,
                score: 0.88,
            },
            Assignment {
                source_column_index: 1,
                target_column_index: 2,
                score: 0.70,
            },
        ]);
        assert_eq!(chosen.len(), 2);
        assert!(chosen.iter().any(|a| a.source_column_index == 0 && a.target_column_index == 1));
        assert!(chosen.iter().any(|a| a.source_column_index == 1 && a.target_column_index == 2));
    }
}
