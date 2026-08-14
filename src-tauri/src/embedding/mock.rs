use crate::embedding::{l2_normalize, EmbeddingProvider};
use crate::error::AppError;

const DIM: usize = 128;
const VERSION: &str = "mock-ngram@v2";

/// Character n-gram hashing encoder. Used when ONNX model files are absent.
/// Unigrams, adjacent bigrams, and ordered skip-grams (non-adjacent pairs)
/// so shared characters that are not next to each other still align
/// (学+号 inside 学生编号).
#[derive(Debug, Default)]
pub struct MockEmbeddingProvider;

impl EmbeddingProvider for MockEmbeddingProvider {
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, AppError> {
        Ok(texts.iter().map(|text| encode(text)).collect())
    }

    fn model_version(&self) -> &str {
        VERSION
    }

    fn backend_name(&self) -> &str {
        "mock"
    }
}

fn encode(text: &str) -> Vec<f32> {
    let mut vec = vec![0.0f32; DIM];
    let chars: Vec<char> = text.chars().filter(|c| !c.is_whitespace()).collect();
    if chars.is_empty() {
        return vec;
    }

    for ch in &chars {
        add_feature(&mut vec, &ch.to_string(), 1.0);
    }
    for window in chars.windows(2) {
        add_feature(&mut vec, &format!("{}{}", window[0], window[1]), 1.4);
    }
    // Ordered skip-grams: pair every character with later non-adjacent ones.
    for i in 0..chars.len() {
        for j in (i + 2)..chars.len() {
            add_feature(&mut vec, &format!("{}{}", chars[i], chars[j]), 1.4);
        }
    }
    l2_normalize(&mut vec);
    vec
}

fn add_feature(vec: &mut [f32], unit: &str, weight: f32) {
    let idx = hash_unit(unit) % vec.len();
    vec[idx] += weight;
}

fn hash_unit(input: &str) -> usize {
    let mut hash: u64 = 1469598103934665603;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(1099511628211);
    }
    hash as usize
}

#[cfg(test)]
mod tests {
    use super::MockEmbeddingProvider;
    use crate::embedding::{cosine_similarity, EmbeddingProvider};

    #[test]
    fn similar_headers_score_higher_than_unrelated() {
        let provider = MockEmbeddingProvider;
        let vectors = provider
            .embed(&["部门名称".into(), "所属部门".into(), "身份证号".into()])
            .unwrap();
        let close = cosine_similarity(&vectors[0], &vectors[1]);
        let far = cosine_similarity(&vectors[0], &vectors[2]);
        assert!(close > far);
    }

    #[test]
    fn skip_grams_raise_student_id_cosine() {
        let provider = MockEmbeddingProvider;
        let vectors = provider
            .embed(&["学号".into(), "学生编号".into(), "姓名".into()])
            .unwrap();
        let close = cosine_similarity(&vectors[0], &vectors[1]);
        let far = cosine_similarity(&vectors[0], &vectors[2]);
        assert!(close > far);
    }
}
