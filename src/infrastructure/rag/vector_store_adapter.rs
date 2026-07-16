use async_trait::async_trait;
use rig_core::vector_store::VectorStoreIndex;
use std::path::PathBuf;

use super::rag_store::RagContext;
use crate::domain::error::MuonError;
use crate::domain::models::source::{Source, SourceType, VerificationStatus};
use crate::domain::traits::vector_store::VectorStore;

const META_BEGIN: &str = "<<<muon_rag";
const META_END: &str = ">>>";

pub fn temp_rag_path(_url: &str) -> PathBuf {
    std::env::temp_dir().join(format!("muon-rag-{}.txt", uuid::Uuid::new_v4()))
}

pub fn pack_rag_content(url: &str, title: &str, content: &str) -> String {
    let url = oneline(url);
    let title = oneline(title);
    format!("{META_BEGIN}\nurl: {url}\ntitle: {title}\n{META_END}\n{content}")
}

pub fn unpack_rag_content(content: &str) -> (Option<String>, Option<String>, String) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with(META_BEGIN) {
        return (None, None, content.to_string());
    }
    let Some(rest) = trimmed.strip_prefix(META_BEGIN) else {
        return (None, None, content.to_string());
    };
    let rest = rest.trim_start_matches(['\r', '\n']);
    let Some((header, body)) = rest.split_once(META_END) else {
        return (None, None, content.to_string());
    };
    let mut url = None;
    let mut title = None;
    for line in header.lines() {
        if let Some(v) = line.strip_prefix("url:") {
            url = Some(v.trim().to_string());
        } else if let Some(v) = line.strip_prefix("title:") {
            title = Some(v.trim().to_string());
        }
    }
    (url, title, body.trim_start_matches(['\r', '\n']).to_string())
}

fn oneline(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c == '\n' || c == '\r' || c == '>' {
                ' '
            } else {
                c
            }
        })
        .collect::<String>()
        .trim()
        .to_string()
}

#[async_trait]
impl VectorStore for RagContext {
    async fn add(&self, source: &Source, content: &str) -> Result<usize, MuonError> {
        let path = temp_rag_path(&source.url);
        let packed = pack_rag_content(&source.url, &source.title, content);

        tokio::fs::write(&path, packed)
            .await
            .map_err(|e| MuonError::Database(format!("failed to write temp rag file: {e}")))?;

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let chunk_count = self
            .indexer
            .add(&path)
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;

        let _ = tokio::fs::remove_file(&path).await;

        if chunk_count > 0 && !source.url.is_empty() {
            let store = self.indexer.pipeline().store();
            store
                .rewrite_source(&file_name, &source.url)
                .await
                .map_err(|e| MuonError::Database(e.to_string()))?;
        }

        Ok(chunk_count)
    }

    async fn query(&self, text: &str, k: usize) -> Result<Vec<Source>, MuonError> {
        let req = rig_core::vector_store::request::VectorSearchRequest::builder()
            .query(text.to_string())
            .samples(k as u64)
            .build();

        let hits = self
            .vector_index
            .top_n_ids(req)
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;

        if hits.is_empty() {
            return Ok(Vec::new());
        }

        let ids_i64: Vec<i64> = hits
            .iter()
            .map(|(_, id_str)| {
                id_str
                    .parse::<i64>()
                    .map_err(|e| MuonError::Database(format!("bad chunk id: {e}")))
            })
            .collect::<Result<Vec<_>, MuonError>>()?;

        let store = self.indexer.pipeline().store();
        let rows = store
            .get_chunks_by_ids(&ids_i64)
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;

        let mut by_id = std::collections::HashMap::new();
        for row in rows {
            by_id.insert(row.id, row);
        }

        let mut results = Vec::with_capacity(hits.len());
        for (score, id_str) in &hits {
            if *score < self.similarity_threshold {
                continue;
            }
            let id_i64 = id_str
                .parse::<i64>()
                .map_err(|e| MuonError::Database(format!("bad chunk id: {e}")))?;

            if let Some(row) = by_id.remove(&id_i64) {
                let (meta_url, meta_title, body) = unpack_rag_content(&row.content);
                let url = meta_url
                    .filter(|u| !u.is_empty())
                    .unwrap_or_else(|| row.source.clone());
                let title = meta_title.filter(|t| !t.is_empty()).unwrap_or_default();
                results.push(Source {
                    url,
                    title,
                    snippet: body,
                    relevance: *score,
                    source_type: SourceType::Knowledge,
                    verified: false,
                    verification_status: VerificationStatus::Unverified,
                    embedding_id: Some(row.id.to_string()),
                });
            }
        }

        Ok(results)
    }

    async fn save_index(&self) -> Result<(), MuonError> {
        self.indexer
            .pipeline()
            .save(&self.index_path)
            .await
            .map_err(|e| MuonError::Database(e.to_string()))
    }
}
