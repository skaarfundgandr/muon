use std::collections::HashSet;
use std::sync::Arc;

use async_trait::async_trait;
use futures::future::join_all;

use crate::domain::models::source::Source;
use crate::domain::traits::search_provider::SearchProvider;
use crate::domain::error::MuonError;

pub(crate) fn percent_encode(input: &str) -> String {
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

pub struct CompositeSearchProvider {
    providers: Vec<Arc<dyn SearchProvider>>,
}

impl CompositeSearchProvider {
    pub fn new(providers: Vec<Arc<dyn SearchProvider>>) -> Self {
        Self { providers }
    }
}

#[async_trait]
impl SearchProvider for CompositeSearchProvider {
    async fn search(&self, query: &str, max: usize) -> Result<Vec<Source>, MuonError> {
        let q = query.to_string();
        let handles: Vec<_> = self
            .providers
            .iter()
            .cloned()
            .map(|p| {
                let q = q.clone();
                async move { p.search(&q, max).await }
            })
            .collect();
        let results = join_all(handles).await;
        let mut all = Vec::new();
        let mut seen = HashSet::new();
        for res in results {
            match res {
                Ok(srcs) => {
                    for s in srcs {
                        if seen.insert(s.url.clone()) {
                            all.push(s);
                        }
                    }
                }
                Err(e) => tracing::warn!(target: "muon::search", "provider failed: {e}"),
            }
        }
        Ok(all)
    }

    fn provider_name(&self) -> &'static str {
        "composite"
    }
}
