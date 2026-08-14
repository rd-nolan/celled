use crate::embedding::{l2_normalize, EmbeddingProvider};
use crate::error::AppError;

const DIM: usize = 128;
const VERSION: &str = "mock-ngram@v1";

/// Character n-gram hashing encoder. Used when ONNX model files are absent.
/// Similar short headers still get useful cosine scores.
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
        let idx = hash_unit(&ch.to_string()) % DIM;
        vec[idx] += 1.0;
    }
    for window in chars.windows(2) {
        let gram = format!("{}{}", window[0], window[1]);
        let idx = hash_unit(&gram) % DIM;
        vec[idx] += 1.4;
    }
    l2_normalize(&mut vec);
    vec
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
}
