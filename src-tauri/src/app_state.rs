use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::database::Database;
use crate::domain::{ImportSession, TemplateSchema};
use crate::embedding::{EmbeddingProvider, TemplateEmbeddingCache};
use crate::mapping::AliasDictionary;

#[derive(Clone)]
pub struct AppState {
    pub embedding: Arc<dyn EmbeddingProvider>,
    pub database: Arc<Database>,
    pub alias: Arc<AliasDictionary>,
    pub template_cache: Arc<Mutex<TemplateEmbeddingCache>>,
    pub template: Arc<Mutex<Option<TemplateSchema>>>,
    pub sessions: Arc<Mutex<HashMap<String, ImportSession>>>,
}

impl AppState {
    pub fn new(
        embedding: Arc<dyn EmbeddingProvider>,
        database: Database,
        alias: AliasDictionary,
    ) -> Self {
        Self {
            embedding,
            database: Arc::new(database),
            alias: Arc::new(alias),
            template_cache: Arc::new(Mutex::new(TemplateEmbeddingCache::default())),
            template: Arc::new(Mutex::new(None)),
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
