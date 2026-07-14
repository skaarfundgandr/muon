use std::path::Path;

use crate::domain::models::source::{Source, SourceType, VerificationStatus};
use crate::domain::traits::vector_store::VectorStore;
use crate::infrastructure::util::expand_tilde;

#[derive(Debug, Clone, Default)]
pub struct IndexSummary {
    pub total_chunks: usize,
    pub total_files: usize,
    pub errors: Vec<String>,
}

pub struct RagIndexerService;

impl RagIndexerService {
    pub async fn index(
        vector_store: &dyn VectorStore,
        path: &Path,
        kind: &str,
    ) -> IndexSummary {
        let expanded = expand_tilde(path);
        match kind.to_uppercase().as_str() {
            "DIRECTORY" => Self::index_directory(vector_store, &expanded).await,
            "FILE" => Self::index_file(vector_store, &expanded).await,
            "GLOB" => Self::index_glob(vector_store, &expanded).await,
            other => {
                let mut summary = IndexSummary::default();
                summary.errors.push(format!("unknown index kind: {other}"));
                summary
            }
        }
    }

    async fn index_directory(vector_store: &dyn VectorStore, dir: &Path) -> IndexSummary {
        let mut summary = IndexSummary::default();
        if !dir.is_dir() {
            summary.errors.push(format!("not a directory: {}", dir.display()));
            return summary;
        }
        if let Err(e) = Self::walk_recursive(vector_store, dir, &mut summary).await {
            summary.errors.push(e);
        }
        summary
    }

    async fn walk_recursive(
        vector_store: &dyn VectorStore,
        root: &Path,
        summary: &mut IndexSummary,
    ) -> Result<(), String> {
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            let mut entries =
                tokio::fs::read_dir(&dir).await.map_err(|e| format!("read_dir {}: {e}", dir.display()))?;
            while let Some(entry) = entries
                .next_entry()
                .await
                .map_err(|e| format!("entry {}: {e}", dir.display()))?
            {
                let ft = entry
                    .file_type()
                    .await
                    .map_err(|e| format!("file_type {}: {e}", entry.path().display()))?;
                if ft.is_dir() {
                    stack.push(entry.path());
                } else if ft.is_file() {
                    Self::index_file_path(vector_store, &entry.path(), summary).await;
                }
            }
        }
        Ok(())
    }

    async fn index_file(vector_store: &dyn VectorStore, path: &Path) -> IndexSummary {
        let mut summary = IndexSummary::default();
        Self::index_file_path(vector_store, path, &mut summary).await;
        summary
    }

    async fn index_glob(vector_store: &dyn VectorStore, path: &Path) -> IndexSummary {
        Self::index_directory(vector_store, path).await
    }

    async fn index_file_path(vector_store: &dyn VectorStore, path: &Path, summary: &mut IndexSummary) {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext.to_ascii_lowercase().as_str() {
            "txt" | "md" => {
                let content = match tokio::fs::read_to_string(path).await {
                    Ok(c) => c,
                    Err(e) => {
                        summary.errors.push(format!("read {}: {e}", path.display()));
                        return;
                    }
                };
                let source = Source {
                    url: path.to_string_lossy().to_string(),
                    title: path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    snippet: String::new(),
                    relevance: 0.0,
                    source_type: SourceType::Knowledge,
                    verified: false,
                    verification_status: VerificationStatus::Unverified,
                    embedding_id: None,
                };
                match vector_store.add(&source, &content).await {
                    Ok(Some(_id)) => {
                        tracing::debug!(target: "muon::rag", path = %path.display(), "indexed");
                        summary.total_chunks += 1;
                        summary.total_files += 1;
                    }
                    Ok(None) => {
                        tracing::debug!(target: "muon::rag", path = %path.display(), "no chunks produced");
                        summary.total_files += 1;
                    }
                    Err(e) => {
                        summary.errors.push(format!("vector_store.add {}: {e}", path.display()));
                    }
                }
            }
            "pdf" => {
                let md = match Self::extract_pdf_to_markdown(path) {
                    Ok(m) => m,
                    Err(e) => {
                        summary.errors.push(format!("pdf {}: {e}", path.display()));
                        return;
                    }
                };
                let source = Source {
                    url: path.to_string_lossy().to_string(),
                    title: path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    snippet: String::new(),
                    relevance: 0.0,
                    source_type: SourceType::Knowledge,
                    verified: false,
                    verification_status: VerificationStatus::Unverified,
                    embedding_id: None,
                };
                match vector_store.add(&source, &md).await {
                    Ok(Some(_id)) => {
                        tracing::debug!(target: "muon::rag", path = %path.display(), "indexed (pdf)");
                        summary.total_chunks += 1;
                        summary.total_files += 1;
                    }
                    Ok(None) => {
                        summary.errors.push(format!("no chunks from PDF {}", path.display()));
                        summary.total_files += 1;
                    }
                    Err(e) => {
                        summary.errors.push(format!("vector_store.add PDF {}: {e}", path.display()));
                    }
                }
            }
            other => {
                tracing::debug!(target: "muon::rag", path = %path.display(), ext = %other, "skipped unsupported extension");
            }
        }
    }

    fn extract_pdf_to_markdown(path: &Path) -> Result<String, String> {
        use pdf_oxide::converters::ConversionOptions;
        use pdf_oxide::document::PdfDocument;

        let doc = PdfDocument::open(path).map_err(|e| format!("pdf_oxide open: {e}"))?;
        let page_count = doc.page_count().map_err(|e| format!("pdf_oxide page_count: {e}"))?;
        let options = ConversionOptions::default();
        let mut result = String::new();
        for i in 0..page_count {
            let md = doc
                .to_markdown(i, &options)
                .map_err(|e| format!("pdf_oxide page {i}: {e}"))?;
            result.push_str(&md);
            result.push_str("\n\n");
        }
        Ok(result)
    }
}
