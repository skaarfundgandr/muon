use agent_rs::observability::{TracerHandle, init_tracing, shutdown_tracing};

use crate::config::LangSmithConfig;
use crate::domain::error::{MuonError, Result};

/// Manages OpenTelemetry tracing lifecycle for LangSmith export.
pub struct Observability {
    handle: Option<TracerHandle>,
}

impl Observability {
    /// Initialize tracing at process start. Reads config + `LANGSMITH_API_KEY` env; no hot-reload
    /// — restart the process to apply changes.
    pub fn init(service: &str, cfg: &LangSmithConfig) -> Result<Self> {
        let api_key = crate::config::expand_env(&cfg.api_key)
            .ok()
            .filter(|k| !k.is_empty())
            .or_else(|| std::env::var("LANGSMITH_API_KEY").ok())
            .filter(|k| !k.is_empty());

        let api_key = match api_key {
            Some(k) => k,
            None => return Ok(Self { handle: None }),
        };

        let mut agent_cfg = agent_rs::domain::observability::LangSmithConfig::from_env();
        agent_cfg.service_name = service.to_string();
        agent_cfg.api_key = api_key;
        if !cfg.project.is_empty() {
            agent_cfg.project = cfg.project.clone();
        }
        if !cfg.endpoint.is_empty() {
            agent_cfg.endpoint = cfg.endpoint.clone();
        }
        agent_cfg.console = cfg.console;
        agent_cfg.batch = cfg.batch;

        if cfg.batch_delay_ms != 0 && std::env::var("LANGSMITH_OTEL_BATCH_DELAY_MS").is_err() {
            // SAFETY: called once at process start before any threads are spawned.
            unsafe {
                std::env::set_var(
                    "LANGSMITH_OTEL_BATCH_DELAY_MS",
                    cfg.batch_delay_ms.to_string(),
                );
            }
        }

        let handle = init_tracing(&agent_cfg).map_err(|e| MuonError::Pipeline(e.to_string()))?;
        Ok(Self {
            handle: Some(handle),
        })
    }

    /// Returns a span emitter for ReAct observability integration.
    pub fn span_emitter() -> std::sync::Arc<dyn agent_rs::agent::react::ReActSpanEmitter> {
        std::sync::Arc::new(agent_rs::observability::react_spans::LangSmithReActEmitter)
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
