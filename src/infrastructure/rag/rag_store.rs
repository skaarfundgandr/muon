use std::path::PathBuf;

use crate::application::config::MuonConfig;
use crate::domain::error::MuonError;
use crate::infrastructure::util::expand_tilde;
use agent_rs::agent::embeddings::ort::ep::{
    CoreML, CPU, CUDA, ExecutionProviderDispatch, OpenVINO, ROCm,
};
use agent_rs::agent::embeddings::{EmbeddingService, FastembedEmbeddingModel, FastembedModel};
use agent_rs::rag::RagPipeline;

pub struct RagContext {
    pub vector_index: agent_rs::rag::TurboVectorIndex,
    pub indexer: agent_rs::rag::RagIndexer,
    pub embedder: EmbeddingService<FastembedEmbeddingModel>,
    pub index_path: PathBuf,
    pub similarity_threshold: f64,
    pub warning: Option<String>,
}

fn build_providers() -> Vec<ExecutionProviderDispatch> {
    vec![
        CUDA::default().build(),
        ROCm::default().build(),
        OpenVINO::default().build(),
        CoreML::default().build(),
        CPU::default().build(),
    ]
}

fn resolve_embedding_model(raw: &str) -> Result<(FastembedModel, Option<String>), MuonError> {
    const LEGACY_MAP: &[(&str, &str)] = &[
        ("Xenova/bge-small-en-v1.5", "BGESmallENV15"),
        ("Xenova/all-MiniLM-L6-v2", "AllMiniLML6V2"),
        ("Xenova/all-mpnet-base-v2", "AllMpnetBaseV2"),
        ("Xenova/multilingual-e5-large", "MultilingualE5Large"),
    ];
    let (name, warning) = match LEGACY_MAP.iter().find(|(old, _)| *old == raw) {
        Some((old, new)) => (
            *new,
            Some(format!(
                "Legacy embedding model '{old}' mapped to '{new}' — update config (Settings → Advanced)"
            )),
        ),
        None => (raw, None),
    };
    let variant = name
        .parse()
        .map_err(|e: String| MuonError::Config(e))?;
    Ok((variant, warning))
}

impl RagContext {
    pub async fn open(cfg: &MuonConfig) -> Result<Self, MuonError> {
        let (variant, model_warning) = resolve_embedding_model(&cfg.advanced.embedding_model)?;
        if let Some(ref w) = model_warning {
            tracing::warn!("{w}");
        }

        let cache_dir = dirs::data_dir()
            .ok_or_else(|| {
                MuonError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "data directory not found",
                ))
            })?
            .join("muon")
            .join("fastembed_cache");

        let providers = build_providers();
        let svc_for_pipeline = EmbeddingService::from_fastembed_with_providers_and_cache_dir(
            variant.clone(),
            providers.clone(),
            &cache_dir,
        )
        .map_err(|e| MuonError::Database(e.to_string()))?;
        let kept_embedder = EmbeddingService::from_fastembed_with_providers_and_cache_dir(
            variant,
            providers,
            &cache_dir,
        )
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
            warning: model_warning,
        })
    }
}
