use std::path::PathBuf;

use crate::application::config::MuonConfig;
use crate::domain::error::MuonError;
use crate::infrastructure::util::expand_tilde;
use agent_rs::agent::embeddings::EmbeddingService;
use agent_rs::rag::RagPipeline;

pub struct RagContext {
    pub vector_index: agent_rs::rag::TurboVectorIndex,
    pub indexer: agent_rs::rag::RagIndexer,
    pub embedder: EmbeddingService<rig_fastembed::EmbeddingModel>,
    pub index_path: PathBuf,
    pub similarity_threshold: f64,
}

impl RagContext {
    pub async fn open(cfg: &MuonConfig) -> Result<Self, MuonError> {
        let variant: rig_fastembed::FastembedModel = cfg
            .advanced
            .embedding_model
            .parse()
            .map_err(|e: String| MuonError::Config(e))?;

        let svc_for_pipeline = EmbeddingService::from_fastembed(variant.clone())
            .map_err(|e| MuonError::Database(e.to_string()))?;
        let kept_embedder = EmbeddingService::from_fastembed(variant)
            .map_err(|e| MuonError::Database(e.to_string()))?;

        let expanded_db = expand_tilde(&cfg.advanced.rag_db_path);
        let index_path = expanded_db.with_extension("tvim");

        let built = RagPipeline::builder()
            .embedder(svc_for_pipeline)
            .db_path(&expanded_db)
            .index_path(index_path.clone())
            .extensions(["txt", "md"])
            .build()
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;

        Ok(Self {
            vector_index: built.vector_index,
            indexer: built.indexer,
            embedder: kept_embedder,
            index_path,
            similarity_threshold: cfg.advanced.similarity_threshold,
        })
    }
}
