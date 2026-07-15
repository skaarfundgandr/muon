use std::sync::Arc;

use rig_core::tool::Tool;
use serde::{Deserialize, Serialize};

use crate::domain::error::MuonError;
use crate::domain::traits::vector_store::VectorStore;

const NAME: &str = "knowledge_search";

#[derive(Debug, Deserialize)]
pub struct KnowledgeSearchArgs {
    pub query: String,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

fn default_top_k() -> usize {
    5
}

#[derive(Debug, Serialize)]
pub struct KnowledgeSearchResult {
    pub url: String,
    pub title: String,
    pub snippet: String,
    pub score: f64,
}

#[derive(Debug, Serialize)]
pub struct KnowledgeSearchOutput {
    pub results: Vec<KnowledgeSearchResult>,
}

pub struct KnowledgeSearchTool {
    vs: Arc<dyn VectorStore>,
}

impl KnowledgeSearchTool {
    pub fn new(vs: Arc<dyn VectorStore>) -> Self {
        Self { vs }
    }
}

impl Tool for KnowledgeSearchTool {
    const NAME: &'static str = NAME;
    type Error = MuonError;
    type Args = KnowledgeSearchArgs;
    type Output = KnowledgeSearchOutput;

    fn description(&self) -> String {
        "Search the local knowledge base (RAG corpus) for relevant documents. Returns matching snippets and source URLs.".to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "The search query." },
                "top_k": { "type": "integer", "description": "Number of results (default 5).", "default": 5 }
            },
            "required": ["query"]
        })
    }

    fn call(
        &self,
        args: Self::Args,
    ) -> impl std::future::Future<Output = Result<Self::Output, Self::Error>>
    + rig_core::wasm_compat::WasmCompatSend {
        let vs = self.vs.clone();
        async move {
            let top_k = if args.top_k == 0 { 5 } else { args.top_k };
            let sources = vs.query(&args.query, top_k).await?;
            Ok(KnowledgeSearchOutput {
                results: sources
                    .into_iter()
                    .map(|s| KnowledgeSearchResult {
                        url: s.url,
                        title: s.title,
                        snippet: s.snippet,
                        score: s.relevance,
                    })
                    .collect(),
            })
        }
    }
}
