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
        self.record_with_meta(url, source_type, String::new(), String::new());
    }

    pub fn record_with_meta(
        &mut self,
        url: impl Into<String>,
        source_type: SourceType,
        title: impl Into<String>,
        snippet: impl Into<String>,
    ) {
        let url: String = url.into();
        let title: String = title.into();
        let snippet: String = snippet.into();
        if let Some(existing) = self.entries.iter_mut().find(|e| e.url == url) {
            if existing.title.is_empty() && !title.is_empty() {
                existing.title = title;
            }
            if existing.snippet.is_empty() && !snippet.is_empty() {
                existing.snippet = snippet;
            }
            return;
        }
        self.entries.push(Source {
            url,
            title,
            snippet,
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

    pub fn sources_mut(&mut self) -> &mut [Source] {
        &mut self.entries
    }

    pub fn clear(&mut self) {
        self.entries.clear();
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
