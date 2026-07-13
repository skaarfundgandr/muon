use std::sync::Arc;

use crate::config::MuonConfig;
use crate::domain::traits::search_provider::SearchProvider;

use super::CompositeSearchProvider;
use super::paper_search::ArxivProvider;
use super::web_search::{BraveProvider, FirecrawlProvider, SerperProvider, TavilyProvider};

pub fn resolve_web_provider(cfg: &MuonConfig) -> Option<Arc<dyn SearchProvider>> {
    if !cfg.data_sources.web_search {
        return None;
    }
    let providers: Vec<Arc<dyn SearchProvider>> = cfg
        .search
        .providers
        .iter()
        .filter_map(build_one_web)
        .collect();
    if providers.is_empty() {
        return None;
    }
    Some(Arc::new(CompositeSearchProvider::new(providers)) as Arc<dyn SearchProvider>)
}

fn build_one_web(p: &crate::config::SearchProviderConfig) -> Option<Arc<dyn SearchProvider>> {
    let key = match crate::config::expand_env(&p.api_key) {
        Ok(k) => k,
        Err(e) => {
            tracing::warn!(target: "muon::search", "skipping '{}': {e}", p.name);
            return None;
        }
    };
    use crate::config::SearchProviderType::*;
    let provider: Arc<dyn SearchProvider> = match p.provider_type {
        Tavily => Arc::new(TavilyProvider::new(key, p.max_results, p.tavily.clone())),
        Firecrawl => Arc::new(FirecrawlProvider::new(
            key,
            p.max_results,
            p.firecrawl.clone(),
        )),
        Brave => Arc::new(BraveProvider::new(key)),
        Serper => Arc::new(SerperProvider::new(key, p.max_results, p.serper.clone())),
    };
    Some(provider)
}

pub fn resolve_paper_providers(cfg: &MuonConfig) -> Vec<Arc<dyn SearchProvider>> {
    if !cfg.data_sources.paper_search {
        return Vec::new();
    }
    let mut v: Vec<Arc<dyn SearchProvider>> = Vec::new();
    if cfg.search.papers.arxiv_enabled {
        v.push(Arc::new(ArxivProvider::new()) as Arc<dyn SearchProvider>);
    }
    v
}
