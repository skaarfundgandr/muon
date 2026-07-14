use async_trait::async_trait;
use serde::Deserialize;

use crate::application::config::{
    FirecrawlCategory, FirecrawlOptions, SerperOptions, TavilyOptions, TavilySearchDepth, TavilyTopic,
};
use crate::domain::error::MuonError;
use crate::domain::models::source::{Source, SourceType, VerificationStatus};
use crate::domain::traits::search_provider::SearchProvider;

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
}

impl TavilyProvider {
    pub fn new(api_key: String, max_results: Option<usize>, options: TavilyOptions) -> Self {
        Self {
            api_key,
            max_results,
            options,
        }
    }
}

#[async_trait]
impl SearchProvider for TavilyProvider {
    async fn search(&self, query: &str, max: usize) -> Result<Vec<Source>, MuonError> {
        let tavily = tavily::Tavily::builder(&self.api_key)
            .build()
            .map_err(|e| MuonError::Search {
                provider: "tavily".into(),
                message: e.to_string(),
            })?;

        let effective_max = self.max_results.unwrap_or(max);
        let depth_str = match self.options.search_depth {
            TavilySearchDepth::Advanced => "advanced",
            _ => "basic",
        };

        let req = {
            let mut r = tavily::SearchRequest::new(&self.api_key, query)
                .search_depth(depth_str)
                .max_results(effective_max as i32);
            if self.options.include_answer {
                r = r.include_answer(true);
            }
            if self.options.include_raw_content {
                r = r.include_raw_content(true);
            }
            if !self.options.include_domains.is_empty() {
                r = r.include_domains(self.options.include_domains.clone());
            }
            if !self.options.exclude_domains.is_empty() {
                r = r.exclude_domains(self.options.exclude_domains.clone());
            }
            if let Some(ref topic) = self.options.topic {
                let label = match topic {
                    TavilyTopic::News => "news",
                    TavilyTopic::Finance => "finance",
                    TavilyTopic::General => "general",
                };
                r = r.topic(label);
            }
            r
        };

        let resp = tavily.call(&req).await.map_err(|e| MuonError::Search {
            provider: "tavily".into(),
            message: e.to_string(),
        })?;

        Ok(resp
            .results
            .into_iter()
            .map(|r| Source {
                url: r.url,
                title: r.title,
                snippet: r.content,
                relevance: r.score as f64,
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
}

impl FirecrawlProvider {
    pub fn new(api_key: String, max_results: Option<usize>, options: FirecrawlOptions) -> Self {
        Self {
            api_key,
            max_results,
            options,
        }
    }
}

#[async_trait]
impl SearchProvider for FirecrawlProvider {
    async fn search(&self, query: &str, max: usize) -> Result<Vec<Source>, MuonError> {
        let client = firecrawl::Client::new(&self.api_key).map_err(|e| MuonError::Search {
            provider: "firecrawl".into(),
            message: e.to_string(),
        })?;

        let effective_max = self.max_results.unwrap_or(max);

        let categories = if self.options.categories.is_empty() {
            None
        } else {
            Some(
                self.options
                    .categories
                    .iter()
                    .map(|c| match c {
                        FirecrawlCategory::Github => firecrawl::SearchCategory::Github,
                        FirecrawlCategory::Research => firecrawl::SearchCategory::Research,
                        FirecrawlCategory::Pdf => firecrawl::SearchCategory::Pdf,
                    })
                    .collect(),
            )
        };
        let formats: Vec<firecrawl::Format> = if self.options.scrape_formats.is_empty() {
            vec![firecrawl::Format::Markdown]
        } else {
            self.options
                .scrape_formats
                .iter()
                .map(|f| {
                    if f.eq_ignore_ascii_case("markdown") {
                        firecrawl::Format::Markdown
                    } else if f.eq_ignore_ascii_case("html") {
                        firecrawl::Format::Html
                    } else {
                        firecrawl::Format::Markdown
                    }
                })
                .collect()
        };
        let include_domains = if self.options.include_domains.is_empty() {
            None
        } else {
            Some(self.options.include_domains.clone())
        };
        let exclude_domains = if self.options.exclude_domains.is_empty() {
            None
        } else {
            Some(self.options.exclude_domains.clone())
        };

        let options = firecrawl::SearchOptions {
            limit: Some(effective_max as u32),
            sources: Some(vec![firecrawl::SearchSource::Web]),
            categories,
            include_domains,
            exclude_domains,
            location: self.options.location.clone(),
            scrape_options: Some(firecrawl::ScrapeOptions {
                formats: Some(formats),
                only_main_content: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        };

        let resp = client
            .search(query, Some(options))
            .await
            .map_err(|e| MuonError::Search {
                provider: "firecrawl".into(),
                message: e.to_string(),
            })?;

        let results = resp.data.web.unwrap_or_default();
        let mut sources = Vec::with_capacity(results.len());
        for item in results {
            match item {
                firecrawl::SearchResultOrDocument::WebResult(wr) => {
                    let title = wr.title.unwrap_or_default();
                    let snippet = wr.description.unwrap_or_default();
                    sources.push(Source {
                        url: wr.url,
                        title,
                        snippet,
                        relevance: 0.0,
                        source_type: SourceType::Web,
                        verified: false,
                        verification_status: VerificationStatus::Unverified,
                        embedding_id: None,
                    });
                }
                firecrawl::SearchResultOrDocument::Document(doc) => {
                    let url = doc
                        .metadata
                        .as_ref()
                        .and_then(|m| m.source_url.clone())
                        .unwrap_or_default();
                    if url.is_empty() {
                        continue;
                    }
                    let title = doc
                        .metadata
                        .as_ref()
                        .and_then(|m| m.title.clone())
                        .unwrap_or_default();
                    let snippet = doc.markdown.unwrap_or_default();
                    sources.push(Source {
                        url,
                        title,
                        snippet,
                        relevance: 0.0,
                        source_type: SourceType::Web,
                        verified: false,
                        verification_status: VerificationStatus::Unverified,
                        embedding_id: None,
                    });
                }
            }
        }
        Ok(sources)
    }

    fn provider_name(&self) -> &'static str {
        "firecrawl"
    }
}

pub struct SerperProvider {
    api_key: String,
    max_results: Option<usize>,
    options: SerperOptions,
}

impl SerperProvider {
    pub fn new(api_key: String, max_results: Option<usize>, options: SerperOptions) -> Self {
        Self {
            api_key,
            max_results,
            options,
        }
    }
}

#[async_trait]
impl SearchProvider for SerperProvider {
    async fn search(&self, query: &str, max: usize) -> Result<Vec<Source>, MuonError> {
        let service = serper_sdk::SearchService::new(self.api_key.clone()).map_err(|e| {
            MuonError::Search {
                provider: "serper".into(),
                message: e.to_string(),
            }
        })?;

        let effective_max = self.max_results.unwrap_or(max);

        let mut sq = serper_sdk::SearchQuery::new(query.to_string()).map_err(|e| {
            MuonError::Search {
                provider: "serper".into(),
                message: e.to_string(),
            }
        })?;
        sq = sq.with_num_results(effective_max as u32);
        if let Some(ref gl) = self.options.gl {
            sq = sq.with_country(gl.clone());
        }
        if let Some(ref hl) = self.options.hl {
            sq = sq.with_language(hl.clone());
        }

        let resp = service.search(&sq).await.map_err(|e| MuonError::Search {
            provider: "serper".into(),
            message: e.to_string(),
        })?;

        let results = resp.organic_results().to_vec();
        Ok(results
            .into_iter()
            .map(|r| {
                let pos = r.position;
                let relevance = if pos == 0 { 0.0 } else { 1.0 / (pos as f64) };
                Source {
                    url: r.link,
                    title: r.title,
                    snippet: r.snippet.unwrap_or_default(),
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
