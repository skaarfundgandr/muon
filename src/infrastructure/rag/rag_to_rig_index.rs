use std::sync::Arc;

use rig_core::vector_store::{
    TopNResults, VectorSearchRequest, VectorStoreError, VectorStoreIndexDyn,
};
use rig_core::vector_store::request::Filter;
use rig_core::wasm_compat::WasmBoxedFuture;

use super::rag_store::RagContext;

#[derive(Clone)]
pub struct RagToRigIndex {
    inner: Arc<RagContext>,
}

impl RagToRigIndex {
    pub fn new(inner: Arc<RagContext>) -> Self {
        Self { inner }
    }
}

impl VectorStoreIndexDyn for RagToRigIndex {
    fn top_n<'a>(
        &'a self,
        req: VectorSearchRequest<Filter<serde_json::Value>>,
    ) -> WasmBoxedFuture<'a, TopNResults> {
        Box::pin(async move {
            VectorStoreIndexDyn::top_n(&self.inner.vector_index, req).await
        })
    }

    fn top_n_ids<'a>(
        &'a self,
        req: VectorSearchRequest<Filter<serde_json::Value>>,
    ) -> WasmBoxedFuture<'a, Result<Vec<(f64, String)>, VectorStoreError>> {
        Box::pin(async move {
            VectorStoreIndexDyn::top_n_ids(&self.inner.vector_index, req).await
        })
    }
}
