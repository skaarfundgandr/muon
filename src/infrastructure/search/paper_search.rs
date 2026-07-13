use async_trait::async_trait;

use crate::domain::error::MuonError;
use crate::domain::models::source::{Source, SourceType, VerificationStatus};
use crate::domain::traits::search_provider::SearchProvider;

pub struct ArxivProvider {
    http: reqwest::Client,
}

impl Default for ArxivProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ArxivProvider {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl SearchProvider for ArxivProvider {
    async fn search(&self, query: &str, max: usize) -> Result<Vec<Source>, MuonError> {
        let url = format!(
            "https://export.arxiv.org/api/query?search_query={}&max_results={}",
            super::percent_encode(query),
            max
        );

        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| MuonError::Search {
                provider: "arxiv".into(),
                message: e.to_string(),
            })?;

        let xml = resp.text().await.map_err(|e| MuonError::Search {
            provider: "arxiv".into(),
            message: e.to_string(),
        })?;

        let mut reader = quick_xml::Reader::from_str(&xml);
        let mut sources = Vec::new();
        let mut in_entry = false;
        let mut title = String::new();
        let mut summary = String::new();
        let mut id = String::new();
        let mut link = String::new();
        let mut buf = Vec::new();
        let mut current_tag = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Start(ref e)) => {
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if tag == "entry" {
                        in_entry = true;
                        title.clear();
                        summary.clear();
                        id.clear();
                        link.clear();
                    }
                    if in_entry {
                        current_tag.clone_from(&tag);
                        if tag == "link" {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"href" {
                                    link = String::from_utf8_lossy(&attr.value).to_string();
                                }
                            }
                        }
                    }
                }
                Ok(quick_xml::events::Event::Text(ref e)) => {
                    if in_entry {
                        let text = e.unescape().map_err(|err| MuonError::Search {
                            provider: "arxiv".into(),
                            message: err.to_string(),
                        })?;
                        match current_tag.as_str() {
                            "title" => title.push_str(&text),
                            "summary" => summary.push_str(&text),
                            "id" => id.push_str(&text),
                            _ => {}
                        }
                    }
                }
                Ok(quick_xml::events::Event::End(ref e)) => {
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if tag == "entry" && in_entry {
                        in_entry = false;
                        sources.push(Source {
                            url: if link.is_empty() {
                                id.clone()
                            } else {
                                link.clone()
                            },
                            title: title.trim().to_string(),
                            snippet: summary.trim().to_string(),
                            relevance: 0.0,
                            source_type: SourceType::Paper,
                            verified: false,
                            verification_status: VerificationStatus::Unverified,
                            embedding_id: None,
                        });
                    }
                }
                Ok(quick_xml::events::Event::Eof) => break,
                Err(e) => {
                    return Err(MuonError::Search {
                        provider: "arxiv".into(),
                        message: e.to_string(),
                    });
                }
                _ => {}
            }
            buf.clear();
        }

        Ok(sources)
    }

    fn provider_name(&self) -> &'static str {
        "arxiv"
    }
}
