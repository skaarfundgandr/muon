use std::future::Future;

use rig_core::completion::ToolDefinition;
use rig_core::tool::Tool;
use serde::{Deserialize, Serialize};

use crate::error::MuonError;

const NAME: &str = "fetch_page";

#[derive(Debug, Deserialize)]
pub struct FetchPageArgs {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct FetchPageOutput {
    pub text: String,
    pub title: Option<String>,
}

pub struct FetchPageTool {
    http: reqwest::Client,
    max_chars: usize,
}

impl FetchPageTool {
    pub fn new(http: reqwest::Client, max_chars: usize) -> Self {
        Self { http, max_chars }
    }
}

impl Tool for FetchPageTool {
    const NAME: &'static str = NAME;
    type Error = MuonError;
    type Args = FetchPageArgs;
    type Output = FetchPageOutput;

    fn definition(
        &self,
        _prompt: String,
    ) -> impl Future<Output = ToolDefinition> + rig_core::wasm_compat::WasmCompatSend + rig_core::wasm_compat::WasmCompatSync
    {
        std::future::ready(ToolDefinition {
            name: NAME.to_string(),
            description: "Fetch a web page and return its text content. HTML tags are stripped and content is truncated.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": { "type": "string", "description": "The URL to fetch." }
                },
                "required": ["url"]
            }),
        })
    }

    fn call(
        &self,
        args: Self::Args,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> + rig_core::wasm_compat::WasmCompatSend
    {
        let http = self.http.clone();
        let max_chars = self.max_chars;
        async move {
            let resp = http
                .get(&args.url)
                .send()
                .await
                .map_err(|e| MuonError::Search {
                    provider: "fetch".into(),
                    message: format!("failed to fetch {}: {e}", args.url),
                })?;

            let status = resp.status();
            if !status.is_success() {
                return Err(MuonError::Search {
                    provider: "fetch".into(),
                    message: format!("{} returned status {status}", args.url),
                });
            }

            let html = resp.text().await.map_err(|e| MuonError::Search {
                provider: "fetch".into(),
                message: format!("failed to read body from {}: {e}", args.url),
            })?;

            let (text, title) = html_to_text(&html, max_chars);
            Ok(FetchPageOutput { text, title })
        }
    }
}

fn html_to_text(html: &str, max_chars: usize) -> (String, Option<String>) {
    // Extract title from <title>...</title>
    let title = {
        let re = regex::Regex::new(r"(?is)<title[^>]*>(.*?)</title>").ok();
        re.and_then(|r| r.captures(html).and_then(|c| c.get(1)).map(|m| strip_tags_single(m.as_str()).trim().to_string()))
    };

    // Strip all HTML tags
    let tag_re = regex::Regex::new(r"(?s)<[^>]+>").ok();
    let mut text = match tag_re {
        Some(re) => re.replace_all(html, " ").to_string(),
        None => html.to_string(),
    };

    // Decode common HTML entities
    text = text.replace("&amp;", "&");
    text = text.replace("&lt;", "<");
    text = text.replace("&gt;", ">");
    text = text.replace("&quot;", "\"");
    text = text.replace("&#39;", "'");
    text = text.replace("&nbsp;", " ");

    // Collapse whitespace
    let ws_re = regex::Regex::new(r"[ \t]+").ok();
    if let Some(re) = ws_re {
        text = re.replace_all(&text, " ").to_string();
    }
    let nl_re = regex::Regex::new(r"\n{3,}").ok();
    if let Some(re) = nl_re {
        text = re.replace_all(&text, "\n\n").to_string();
    }

    text = text.trim().to_string();

    // Truncate
    if text.len() > max_chars {
        text.truncate(max_chars);
        // Try to break at a sentence boundary
        if let Some(pos) = text.rfind('.')
            && pos > max_chars / 2
        {
            text.truncate(pos + 1);
        }
    }

    (text, title)
}

fn strip_tags_single(html: &str) -> String {
    let re = regex::Regex::new(r"(?s)<[^>]+>").ok();
    match re {
        Some(r) => r.replace_all(html, " ").to_string(),
        None => html.to_string(),
    }
}
