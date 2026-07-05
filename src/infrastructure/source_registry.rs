use crate::domain::models::source::{Source, SourceType, VerificationStatus};

/// Tracks discovered source URLs to avoid duplicates across the pipeline.
#[derive(Debug, Default, Clone)]
pub struct SourceRegistry {
    entries: Vec<Source>,
}

impl SourceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, url: impl Into<String>, source_type: SourceType) {
        let url: String = url.into();
        if self.entries.iter().any(|e| e.url == url) {
            return;
        }
        self.entries.push(Source {
            url,
            title: String::new(),
            snippet: String::new(),
            relevance: 0.0,
            source_type,
            verified: false,
            verification_status: VerificationStatus::Unverified,
            embedding_id: None,
        });
    }

    pub fn urls(&self) -> Vec<String> {
        self.entries.iter().map(|e| e.url.clone()).collect()
    }

    pub fn into_sources(self) -> Vec<Source> {
        self.entries
    }

    pub fn sources(&self) -> &[Source] {
        &self.entries
    }
}

type UrlExtractor = Box<dyn Fn(&str) -> Vec<String> + Send + Sync>;

/// Side-channel observer that extracts URLs from observation text and records
/// them in a shared `SourceRegistry`. This differs from
/// `agent_rs::agent::tools::RagSourceRegistry` which tracks RAG file paths.
pub struct RegistryTool {
    registry: std::sync::Arc<std::sync::Mutex<SourceRegistry>>,
    url_extractor: UrlExtractor,
}

impl RegistryTool {
    pub fn new<F>(registry: std::sync::Arc<std::sync::Mutex<SourceRegistry>>, url_extractor: F) -> Self
    where
        F: Fn(&str) -> Vec<String> + Send + Sync + 'static,
    {
        Self {
            registry,
            url_extractor: Box::new(url_extractor) as UrlExtractor,
        }
    }

    pub fn observe(&self, observation_result: &str, default_type: SourceType) {
        if let Ok(mut g) = self.registry.lock() {
            for u in (self.url_extractor)(observation_result) {
                g.record(u, default_type);
            }
        }
    }
}
