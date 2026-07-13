use std::future::Future;
use std::sync::Arc;

use rig_core::completion::ToolDefinition;
use rig_core::tool::Tool;
use serde::{Deserialize, Serialize};

use crate::domain::error::MuonError;
use crate::domain::traits::search_provider::SearchProvider;

const NAME: &str = "paper_search";

#[derive(Debug, Deserialize)]
pub struct PaperSearchArgs {
    pub query: String,
    #[serde(default = "default_max")]
    pub max: usize,
}

fn default_max() -> usize {
    5
}

#[derive(Debug, Serialize)]
pub struct PaperSearchOutput {
    pub results: Vec<PaperSearchResult>,
}

#[derive(Debug, Serialize)]
pub struct PaperSearchResult {
    pub url: String,
    pub title: String,
    pub snippet: String,
    pub score: f64,
}

pub struct PaperSearchTool {
    providers: Vec<Arc<dyn SearchProvider>>,
}

impl PaperSearchTool {
    pub fn new(providers: Vec<Arc<dyn SearchProvider>>) -> Self {
        Self { providers }
    }
}

impl Tool for PaperSearchTool {
    const NAME: &'static str = NAME;
    type Error = MuonError;
    type Args = PaperSearchArgs;
    type Output = PaperSearchOutput;

    fn definition(
        &self,
        _prompt: String,
    ) -> impl Future<Output = ToolDefinition>
    + rig_core::wasm_compat::WasmCompatSend
    + rig_core::wasm_compat::WasmCompatSync {
        std::future::ready(ToolDefinition {
            name: NAME.to_string(),
            description: "Search academic papers on arXiv and Semantic Scholar. Returns URLs, titles, and snippets.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "The search query for academic papers." },
                    "max": { "type": "integer", "description": "Max results per provider (default 5).", "default": 5 }
                },
                "required": ["query"]
            }),
        })
    }

    fn call(
        &self,
        args: Self::Args,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> + rig_core::wasm_compat::WasmCompatSend
    {
        let providers = self.providers.clone();
        async move {
            let max = if args.max == 0 { 5 } else { args.max };
            let mut all_results: Vec<PaperSearchResult> = Vec::new();
            let mut seen_urls: std::collections::HashSet<String> = std::collections::HashSet::new();

            for provider in &providers {
                match provider.search(&args.query, max).await {
                    Ok(sources) => {
                        for s in sources {
                            if seen_urls.insert(s.url.clone()) {
                                all_results.push(PaperSearchResult {
                                    url: s.url,
                                    title: s.title,
                                    snippet: s.snippet,
                                    score: s.relevance,
                                });
                            }
                        }
                    }
                    Err(e) => {
                        // Log but continue — other providers may succeed.
                        tracing::warn!(
                            "paper_search provider {} failed: {}",
                            provider.provider_name(),
                            e
                        );
                    }
                }
            }

            Ok(PaperSearchOutput {
                results: all_results,
            })
        }
    }
}
