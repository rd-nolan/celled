use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;

use crate::error::AppError;

pub trait EmbeddingProvider: Send + Sync {
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, AppError>;
    fn model_version(&self) -> &str;
    fn backend_name(&self) -> &str;
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }
    let mut dot = 0.0f32;
    let mut na = 0.0f32;
    let mut nb = 0.0f32;
    for i in 0..a.len() {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na.sqrt() * nb.sqrt())
    }
}

/// Header similarity used by the embedding match step.
///
/// Combines embedding cosine with ordered character coverage (so a short
/// header that is a subsequence of a longer one, e.g. 学号 ⊂ 学生编号, scores
/// highly) and an IDF-weighted character Jaccard over the template corpus.
/// No named synonym list is involved.
pub fn header_similarity(
    source_text: &str,
    source_vec: &[f32],
    target_text: &str,
    target_vec: &[f32],
    corpus: &[String],
) -> f32 {
    let cosine = cosine_similarity(source_vec, target_vec).clamp(0.0, 1.0);
    let coverage = ordered_char_coverage(source_text, target_text);
    let len_ratio = char_len_ratio(source_text, target_text);
    let coverage_score = coverage * (0.5 + 0.5 * len_ratio.sqrt());
    let overlap = idf_weighted_jaccard(source_text, target_text, corpus);
    (0.40 * cosine + 0.40 * coverage_score + 0.20 * overlap).clamp(0.0, 1.0)
}

fn significant_chars(text: &str) -> Vec<char> {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Fraction of the shorter header's characters that appear in order in the longer one.
fn ordered_char_coverage(a: &str, b: &str) -> f32 {
    let ca = significant_chars(a);
    let cb = significant_chars(b);
    if ca.is_empty() || cb.is_empty() {
        return 0.0;
    }
    let (short, long) = if ca.len() <= cb.len() {
        (&ca, &cb)
    } else {
        (&cb, &ca)
    };
    let mut j = 0usize;
    let mut matched = 0usize;
    for &ch in short.iter() {
        while j < long.len() && long[j] != ch {
            j += 1;
        }
        if j < long.len() {
            matched += 1;
            j += 1;
        }
    }
    let coverage = matched as f32 / short.len() as f32;
    if short.len() < 2 {
        coverage * 0.5
    } else {
        coverage
    }
}

fn char_len_ratio(a: &str, b: &str) -> f32 {
    let na = significant_chars(a).len().max(1);
    let nb = significant_chars(b).len().max(1);
    let (short, long) = if na <= nb { (na, nb) } else { (nb, na) };
    short as f32 / long as f32
}

fn idf_weighted_jaccard(a: &str, b: &str, corpus: &[String]) -> f32 {
    let ca: HashSet<char> = significant_chars(a).into_iter().collect();
    let cb: HashSet<char> = significant_chars(b).into_iter().collect();
    if ca.is_empty() || cb.is_empty() {
        return 0.0;
    }

    let n = corpus.len().max(1) as f32;
    let mut df: HashMap<char, usize> = HashMap::new();
    for text in corpus {
        let uniq: HashSet<char> = significant_chars(text).into_iter().collect();
        for ch in uniq {
            *df.entry(ch).or_insert(0) += 1;
        }
    }

    let idf = |ch: char| {
        let d = df.get(&ch).copied().unwrap_or(0) as f32;
        ((n + 1.0) / (d + 1.0)).ln() + 1.0
    };

    let inter: f32 = ca.intersection(&cb).map(|&ch| idf(ch)).sum();
    let union: f32 = ca.union(&cb).map(|&ch| idf(ch)).sum();
    if union <= 0.0 {
        0.0
    } else {
        inter / union
    }
}

pub fn l2_normalize(vector: &mut [f32]) {
    let mut norm = 0.0f32;
    for v in vector.iter() {
        norm += *v * *v;
    }
    let norm = norm.sqrt();
    if norm > 0.0 {
        for v in vector.iter_mut() {
            *v /= norm;
        }
    }
}

pub fn create_provider(model_dir: Option<&Path>) -> Arc<dyn EmbeddingProvider> {
    #[cfg(feature = "onnx")]
    {
        if let Some(dir) = model_dir {
            match crate::embedding::OnnxEmbeddingProvider::load(dir) {
                Ok(provider) => return Arc::new(provider),
                Err(err) => {
                    eprintln!("ONNX embedding unavailable ({err}), falling back to mock provider");
                }
            }
        }
    }
    #[cfg(not(feature = "onnx"))]
    let _ = model_dir;
    Arc::new(crate::embedding::MockEmbeddingProvider::default())
}

#[cfg(test)]
mod tests {
    use super::{cosine_similarity, create_provider, header_similarity, EmbeddingProvider};

    #[test]
    fn cosine_of_identical_vectors_is_one() {
        let a = vec![0.2, 0.4, 0.8];
        assert!((cosine_similarity(&a, &a) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn cosine_of_orthogonal_vectors_is_zero() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!(cosine_similarity(&a, &b).abs() < 1e-6);
    }

    #[test]
    fn missing_model_uses_mock_backend() {
        let provider = create_provider(None);
        assert_eq!(provider.backend_name(), "mock");
    }

    #[test]
    fn student_id_headers_score_above_embedding_threshold() {
        let provider = create_provider(None);
        let texts = [
            "学号".to_string(),
            "学生编号".to_string(),
            "姓名".to_string(),
        ];
        let vectors = provider.embed(&texts).unwrap();
        let close = header_similarity("学号", &vectors[0], "学生编号", &vectors[1], &texts);
        let far = header_similarity("学号", &vectors[0], "姓名", &vectors[2], &texts);
        assert!(
            close > 0.52,
            "学号 vs 学生编号 score {close} should clear the embedding threshold"
        );
        assert!(
            close > far,
            "学号 vs 学生编号 ({close}) should beat 学号 vs 姓名 ({far})"
        );
    }
}
