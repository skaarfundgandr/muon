use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;

use crate::config::{FirecrawlOptions, SerperOptions, TavilyOptions};
use crate::domain::models::source::{Source, SourceType, VerificationStatus};
use crate::domain::traits::search_provider::SearchProvider;
use crate::domain::error::MuonError;

pub struct BraveProvider {
    api_key: String,
    http: reqwest::Client,
    base_url: String,
}

impl BraveProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            http: reqwest::Client::new(),
            base_url: "https://api.search.brave.com".into(),
        }
    }
}

#[derive(Deserialize)]
struct BraveResponse {
    web: Option<BraveWeb>,
}

#[derive(Deserialize)]
struct BraveWeb {
    results: Option<Vec<BraveResult>>,
}

#[derive(Deserialize)]
struct BraveResult {
    title: String,
    url: String,
    description: String,
}

#[async_trait]
impl SearchProvider for BraveProvider {
    async fn search(&self, query: &str, max: usize) -> Result<Vec<Source>, MuonError> {
        let url = format!(
            "{}/api/v1/web/search?q={}&count={}",
            self.base_url,
            super::percent_encode(query),
            max
        );
        let resp = self
            .http
            .get(&url)
            .header("X-Subscription-Token", &self.api_key)
            .send()
            .await
            .map_err(|e| MuonError::Search {
                provider: "brave".into(),
                message: e.to_string(),
            })?;

        let body: BraveResponse = resp.json().await.map_err(|e| MuonError::Search {
            provider: "brave".into(),
            message: e.to_string(),
        })?;

        let results = body.web.and_then(|w| w.results).unwrap_or_default();

        Ok(results
            .into_iter()
            .map(|r| Source {
                url: r.url,
                title: r.title,
                snippet: r.description,
                relevance: 0.0,
                source_type: SourceType::Web,
                verified: false,
                verification_status: VerificationStatus::Unverified,
                embedding_id: None,
            })
            .collect())
    }

    fn provider_name(&self) -> &'static str {
        "brave"
    }
}

pub struct TavilyProvider {
    api_key: String,
    max_results: Option<usize>,
    options: TavilyOptions,
    http: reqwest::Client,
}

impl TavilyProvider {
    pub fn new(api_key: String, max_results: Option<usize>, options: TavilyOptions) -> Self {
        Self {
            api_key,
            max_results,
            options,
            http: reqwest::Client::new(),
        }
    }
}

#[derive(Deserialize)]
struct TavilyResponse {
    results: Vec<TavilyResult>,
}

#[derive(Deserialize)]
struct TavilyResult {
    title: String,
    url: String,
    content: String,
    score: f64,
}

#[async_trait]
impl SearchProvider for TavilyProvider {
    async fn search(&self, query: &str, max: usize) -> Result<Vec<Source>, MuonError> {
        let effective_max = self.max_results.unwrap_or(max);
        let search_depth = serde_json::to_value(&self.options.search_depth)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| "basic".into());
        let topic = self
            .options
            .topic
            .as_ref()
            .and_then(|t| serde_json::to_value(t).ok())
            .and_then(|v| v.as_str().map(String::from));

        let mut body = json!({
            "query": query,
            "search_depth": search_depth,
            "max_results": effective_max,
            "include_answer": self.options.include_answer,
            "include_raw_content": self.options.include_raw_content,
            "include_domains": self.options.include_domains,
            "exclude_domains": self.options.exclude_domains,
        });

        if let Some(ref t) = topic {
            body["topic"] = json!(t);
        }
        if let Some(ref tr) = self.options.time_range {
            body["time_range"] = json!(tr);
        }

        let resp = self
            .http
            .post("https://api.tavily.com/search")
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| MuonError::Search {
                provider: "tavily".into(),
                message: e.to_string(),
            })?;

        let body: TavilyResponse = resp.json().await.map_err(|e| MuonError::Search {
            provider: "tavily".into(),
            message: e.to_string(),
        })?;

        Ok(body
            .results
            .into_iter()
            .map(|r| Source {
                url: r.url,
                title: r.title,
                snippet: r.content,
                relevance: r.score,
                source_type: SourceType::Web,
                verified: false,
                verification_status: VerificationStatus::Unverified,
                embedding_id: None,
            })
            .collect())
    }

    fn provider_name(&self) -> &'static str {
        "tavily"
    }
}

pub struct FirecrawlProvider {
    api_key: String,
    max_results: Option<usize>,
    options: FirecrawlOptions,
    http: reqwest::Client,
}

impl FirecrawlProvider {
    pub fn new(api_key: String, max_results: Option<usize>, options: FirecrawlOptions) -> Self {
        Self {
            api_key,
            max_results,
            options,
            http: reqwest::Client::new(),
        }
    }
}

#[derive(Deserialize)]
struct FirecrawlResponse {
    data: Vec<FirecrawlResult>,
}

#[derive(Deserialize)]
struct FirecrawlResult {
    title: Option<String>,
    url: Option<String>,
    description: Option<String>,
    markdown: Option<String>,
}

#[async_trait]
impl SearchProvider for FirecrawlProvider {
    async fn search(&self, query: &str, max: usize) -> Result<Vec<Source>, MuonError> {
        let effective_max = self.max_results.unwrap_or(max);
        let formats = if self.options.scrape_formats.is_empty() {
            vec!["markdown".to_string()]
        } else {
            self.options.scrape_formats.clone()
        };

        let body = json!({
            "query": query,
            "limit": effective_max,
            "sources": ["web"],
            "scrapeOptions": {
                "formats": formats,
                "onlyMainContent": true,
            },
        });

        let resp = self
            .http
            .post("https://api.firecrawl.dev/v2/search")
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| MuonError::Search {
                provider: "firecrawl".into(),
                message: e.to_string(),
            })?;

        let body: FirecrawlResponse = resp.json().await.map_err(|e| MuonError::Search {
            provider: "firecrawl".into(),
            message: e.to_string(),
        })?;

        Ok(body
            .data
            .into_iter()
            .filter_map(|r| {
                let url = r.url?;
                let title = r.title.unwrap_or_default();
                let snippet = match (&r.description, &r.markdown) {
                    (Some(desc), Some(md)) if !desc.is_empty() => {
                        format!("{desc}\n\n{md}")
                    }
                    (Some(desc), _) if !desc.is_empty() => desc.clone(),
                    (_, Some(md)) => md.clone(),
                    _ => String::new(),
                };
                Some(Source {
                    url,
                    title,
                    snippet,
                    relevance: 0.0,
                    source_type: SourceType::Web,
                    verified: false,
                    verification_status: VerificationStatus::Unverified,
                    embedding_id: None,
                })
            })
            .collect())
    }

    fn provider_name(&self) -> &'static str {
        "firecrawl"
    }
}

pub struct SerperProvider {
    api_key: String,
    max_results: Option<usize>,
    options: SerperOptions,
    http: reqwest::Client,
}

impl SerperProvider {
    pub fn new(api_key: String, max_results: Option<usize>, options: SerperOptions) -> Self {
        Self {
            api_key,
            max_results,
            options,
            http: reqwest::Client::new(),
        }
    }
}

#[derive(Deserialize)]
struct SerperResponse {
    organic: Vec<SerperResult>,
}

#[derive(Deserialize)]
struct SerperResult {
    title: String,
    link: String,
    snippet: String,
    position: Option<usize>,
}

#[async_trait]
impl SearchProvider for SerperProvider {
    async fn search(&self, query: &str, max: usize) -> Result<Vec<Source>, MuonError> {
        let effective_max = self.max_results.unwrap_or(max);
        let mut body = json!({
            "q": query,
            "num": effective_max,
            "autocorrect": self.options.autocorrect,
        });

        if let Some(ref gl) = self.options.gl {
            body["gl"] = json!(gl);
        }
        if let Some(ref hl) = self.options.hl {
            body["hl"] = json!(hl);
        }
        if let Some(ref tbs) = self.options.tbs {
            body["tbs"] = json!(tbs);
        }

        let resp = self
            .http
            .post("https://google.serper.dev/search")
            .header("X-API-KEY", &self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| MuonError::Search {
                provider: "serper".into(),
                message: e.to_string(),
            })?;

        let body: SerperResponse = resp.json().await.map_err(|e| MuonError::Search {
            provider: "serper".into(),
            message: e.to_string(),
        })?;

        Ok(body
            .organic
            .into_iter()
            .map(|r| {
                let pos = r.position.unwrap_or(0);
                let relevance = if pos == 0 {
                    0.0
                } else {
                    1.0 / (pos as f64)
                };
                Source {
                    url: r.link,
                    title: r.title,
                    snippet: r.snippet,
                    relevance,
                    source_type: SourceType::Web,
                    verified: false,
                    verification_status: VerificationStatus::Unverified,
                    embedding_id: None,
                }
            })
            .collect())
    }

    fn provider_name(&self) -> &'static str {
        "serper"
    }
}
