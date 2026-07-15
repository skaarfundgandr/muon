use async_trait::async_trait;
use rig_core::vector_store::VectorStoreIndex;
use std::path::PathBuf;

use super::rag_store::RagContext;
use crate::domain::error::MuonError;
use crate::domain::models::source::{Source, SourceType, VerificationStatus};
use crate::domain::traits::vector_store::VectorStore;

const TEMP_SLUG_MAX: usize = 48;

pub fn temp_rag_path(url: &str) -> PathBuf {
    let mut slug: String = url
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    if slug.len() > TEMP_SLUG_MAX {
        slug.truncate(TEMP_SLUG_MAX);
    }
    while slug.ends_with('_') {
        slug.pop();
    }
    if slug.is_empty() {
        slug.push_str("src");
    }
    std::env::temp_dir().join(format!(
        "muon-rag-{}-{}.txt",
        slug,
        uuid::Uuid::new_v4()
    ))
}

#[async_trait]
impl VectorStore for RagContext {
    async fn add(&self, source: &Source, content: &str) -> Result<Option<String>, MuonError> {
        let path = temp_rag_path(&source.url);

        tokio::fs::write(&path, content)
            .await
            .map_err(|e| MuonError::Database(format!("failed to write temp rag file: {e}")))?;

        let chunk_count = self
            .indexer
            .add(&path)
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;

        let _ = tokio::fs::remove_file(&path).await;

        if chunk_count == 0 {
            return Ok(None);
        }

        let file_name = path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        Ok(Some(format!("{file_name}-{chunk_count}")))
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
            let id_i64 = id_str
                .parse::<i64>()
                .map_err(|e| MuonError::Database(format!("bad chunk id: {e}")))?;

            if let Some(row) = by_id.remove(&id_i64) {
                results.push(Source {
                    url: row.source.clone(),
                    title: row.source.clone(),
                    snippet: row.content.clone(),
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
