use crate::config::MuonConfig;
use crate::domain::traits::search_provider::SearchProvider;

use super::paper_search::{ArxivProvider, SemanticScholarProvider};
use super::web_search::{BraveProvider, SearXngProvider};

pub enum WebProviderKind {
    Brave,
    SearXNG,
}

pub fn resolve_web_provider(cfg: &MuonConfig) -> Option<Box<dyn SearchProvider>> {
    if !cfg.data_sources.web_search {
        return None;
    }
    if !cfg.tools.searxng_url.is_empty() {
        let api_key = if cfg.tools.searxng_api_key.is_empty() {
            None
        } else {
            Some(cfg.tools.searxng_api_key.clone())
        };
        return Some(Box::new(SearXngProvider::new(
            cfg.tools.searxng_url.clone(),
            api_key,
        )));
    }
    if !cfg.tools.brave_api_key.is_empty() {
        return Some(Box::new(BraveProvider::new(cfg.tools.brave_api_key.clone())));
    }
    None
}

pub fn resolve_paper_providers(cfg: &MuonConfig) -> Vec<Box<dyn SearchProvider>> {
    if !cfg.data_sources.paper_search {
        return Vec::new();
    }
    let mut providers: Vec<Box<dyn SearchProvider>> = Vec::new();
    let api_key = if cfg.tools.semantic_scholar_api_key.is_empty() {
        None
    } else {
        Some(cfg.tools.semantic_scholar_api_key.clone())
    };
    providers.push(Box::new(SemanticScholarProvider::new(api_key)));
    if cfg.tools.arxiv_enabled {
        providers.push(Box::new(ArxivProvider::new()));
    }
    providers
}
