use async_trait::async_trait;
use serde::Deserialize;

use crate::domain::models::source::{Source, SourceType, VerificationStatus};
use crate::domain::traits::search_provider::SearchProvider;
use crate::error::MuonError;

fn percent_encode(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len() * 3);
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            b' ' => encoded.push('+'),
            _ => {
                encoded.push('%');
                encoded.push_str(&format!("{:02X}", byte));
            }
        }
    }
    encoded
}

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
            percent_encode(query),
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

pub struct SearXngProvider {
    base_url: String,
    api_key: Option<String>,
    http: reqwest::Client,
}

impl SearXngProvider {
    pub fn new(base_url: String, api_key: Option<String>) -> Self {
        Self {
            base_url,
            api_key,
            http: reqwest::Client::new(),
        }
    }
}

#[derive(Deserialize)]
struct SearXngResponse {
    results: Option<Vec<SearXngResult>>,
}

#[derive(Deserialize)]
struct SearXngResult {
    title: String,
    url: String,
    content: Option<String>,
}

#[async_trait]
impl SearchProvider for SearXngProvider {
    async fn search(&self, query: &str, max: usize) -> Result<Vec<Source>, MuonError> {
        let mut url = format!(
            "{}/search?format=json&q={}&n={}",
            self.base_url,
            percent_encode(query),
            max
        );
        if let Some(ref key) = self.api_key {
            url = format!("{}&api_key={}", url, key);
        }

        let resp = self.http.get(&url).send().await.map_err(|e| MuonError::Search {
            provider: "searxng".into(),
            message: e.to_string(),
        })?;

        let body: SearXngResponse = resp.json().await.map_err(|e| MuonError::Search {
            provider: "searxng".into(),
            message: e.to_string(),
        })?;

        let results = body.results.unwrap_or_default();

        Ok(results
            .into_iter()
            .map(|r| Source {
                url: r.url,
                title: r.title,
                snippet: r.content.unwrap_or_default(),
                relevance: 0.0,
                source_type: SourceType::Web,
                verified: false,
                verification_status: VerificationStatus::Unverified,
                embedding_id: None,
            })
            .collect())
    }

    fn provider_name(&self) -> &'static str {
        "searxng"
    }
}
