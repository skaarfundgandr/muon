use std::future::Future;
use std::sync::Arc;

use rig_core::completion::ToolDefinition;
use rig_core::tool::Tool;
use serde::{Deserialize, Serialize};

use crate::domain::traits::search_provider::SearchProvider;
use crate::domain::error::MuonError;

const NAME: &str = "web_search";

#[derive(Debug, Deserialize)]
pub struct WebSearchArgs {
    pub query: String,
    #[serde(default = "default_max")]
    pub max: usize,
}

fn default_max() -> usize {
    5
}

#[derive(Debug, Serialize)]
pub struct WebSearchOutput {
    pub results: Vec<WebSearchResult>,
}

#[derive(Debug, Serialize)]
pub struct WebSearchResult {
    pub url: String,
    pub title: String,
    pub snippet: String,
}

pub struct WebSearchTool {
    provider: Arc<dyn SearchProvider>,
}

impl WebSearchTool {
    pub fn new(provider: Arc<dyn SearchProvider>) -> Self {
        Self { provider }
    }
}

impl Tool for WebSearchTool {
    const NAME: &'static str = NAME;
    type Error = MuonError;
    type Args = WebSearchArgs;
    type Output = WebSearchOutput;

    fn definition(
        &self,
        _prompt: String,
    ) -> impl Future<Output = ToolDefinition> + rig_core::wasm_compat::WasmCompatSend + rig_core::wasm_compat::WasmCompatSync
    {
        std::future::ready(ToolDefinition {
            name: NAME.to_string(),
            description: "Search the web for fresh results. Returns URLs, titles, and snippets.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "The search query." },
                    "max": { "type": "integer", "description": "Max results (default 5).", "default": 5 }
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
        let provider = self.provider.clone();
        async move {
            let max = if args.max == 0 { 5 } else { args.max };
            let sources = provider.search(&args.query, max).await?;
            Ok(WebSearchOutput {
                results: sources
                    .into_iter()
                    .map(|s| WebSearchResult {
                        url: s.url,
                        title: s.title,
                        snippet: s.snippet,
                    })
                    .collect(),
            })
        }
    }
}
