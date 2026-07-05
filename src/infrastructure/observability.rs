use agent_rs::observability::{TracerHandle, init_tracing, shutdown_tracing};

use crate::error::{MuonError, Result};

/// Manages OpenTelemetry tracing lifecycle for LangSmith export.
pub struct Observability {
    handle: Option<TracerHandle>,
}

impl Observability {
    /// Initialize tracing. Returns a no-op handle if `LANGSMITH_API_KEY` is unset.
    pub fn init(service: &str) -> Result<Self> {
        let api_key = std::env::var("LANGSMITH_API_KEY").ok();
        if api_key.as_deref().is_some_and(|k| !k.is_empty()) {
            let mut cfg = agent_rs::domain::observability::LangSmithConfig::from_env();
            cfg.service_name = service.to_string();
            let handle =
                init_tracing(&cfg).map_err(|e| MuonError::Pipeline(e.to_string()))?;
            return Ok(Self {
                handle: Some(handle),
            });
        }
        Ok(Self { handle: None })
    }

    /// Returns a span emitter for ReAct observability integration.
    pub fn span_emitter() -> std::sync::Arc<dyn agent_rs::agent::react::ReActSpanEmitter> {
        std::sync::Arc::new(
            agent_rs::observability::react_spans::LangSmithReActEmitter,
        )
    }

    /// Flush pending spans and shut down the tracer provider.
    pub async fn shutdown(self) -> Result<()> {
        if let Some(handle) = self.handle {
            shutdown_tracing(handle)
                .await
                .map_err(|e| MuonError::Pipeline(e.to_string()))?;
        }
        Ok(())
    }
}
