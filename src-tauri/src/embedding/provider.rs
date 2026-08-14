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
    use super::{cosine_similarity, create_provider, EmbeddingProvider};

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
}
