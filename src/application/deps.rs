use std::sync::{Arc, Mutex};

use crate::domain::traits::agent::MuonAgent;
use crate::domain::traits::session_store::SessionStore;
use crate::domain::traits::vector_store::VectorStore;
use crate::infrastructure::context::InfrastructureContext;
use crate::infrastructure::source_registry::SourceRegistry;

pub struct PipelineDeps {
    pub intent_classifier: Arc<dyn MuonAgent>,
    pub shallow: Arc<dyn MuonAgent>,
    pub clarifier: Arc<dyn MuonAgent>,
    pub deep_orchestrator: Arc<dyn MuonAgent>,
    pub planner: Arc<dyn MuonAgent>,
    pub researcher: Arc<dyn MuonAgent>,
    pub session_store: Arc<dyn SessionStore>,
    pub source_sink: Arc<Mutex<SourceRegistry>>,
    pub vector_store: Option<Arc<dyn VectorStore>>,
}

impl PipelineDeps {
    pub fn from_infra(infra: &InfrastructureContext) -> Self {
        Self {
            intent_classifier: Arc::clone(&infra.intent_classifier),
            shallow: Arc::clone(&infra.shallow),
            clarifier: Arc::clone(&infra.clarifier),
            deep_orchestrator: Arc::clone(&infra.deep_orchestrator),
            planner: Arc::clone(&infra.planner),
            researcher: Arc::clone(&infra.researcher),
            session_store: Arc::clone(&infra.session_store),
            source_sink: Arc::clone(&infra.source_sink),
            vector_store: infra.vector_store.as_ref().map(Arc::clone),
        }
    }
}
