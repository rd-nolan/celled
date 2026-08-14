use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CachedTemplateEmbedding {
    pub template_id: String,
    pub model_version: String,
    pub vectors: Vec<Vec<f32>>,
}

#[derive(Debug, Default)]
pub struct TemplateEmbeddingCache {
    inner: HashMap<String, CachedTemplateEmbedding>,
}

impl TemplateEmbeddingCache {
    pub fn get(&self, template_id: &str, model_version: &str) -> Option<&[Vec<f32>]> {
        self.inner.get(template_id).and_then(|cached| {
            if cached.model_version == model_version {
                Some(cached.vectors.as_slice())
            } else {
                None
            }
        })
    }

    pub fn insert(&mut self, cached: CachedTemplateEmbedding) {
        self.inner.insert(cached.template_id.clone(), cached);
    }
}
