use agent_rs::observability::{TracerHandle, init_tracing, shutdown_tracing};

use crate::application::config::LangSmithConfig;
use crate::domain::error::{MuonError, Result};

/// Manages OpenTelemetry tracing lifecycle for LangSmith export.
pub struct Observability {
    handle: Option<TracerHandle>,
}

impl Observability {
    /// Initialize tracing at process start. Reads config + `LANGSMITH_API_KEY` env; no hot-reload
    /// — restart the process to apply changes.
    pub fn init(service: &str, cfg: &LangSmithConfig) -> Result<Self> {
        let api_key = crate::infrastructure::config::expand_env(&cfg.api_key)
            .ok()
            .filter(|k| !k.is_empty())
            .or_else(|| std::env::var("LANGSMITH_API_KEY").ok())
            .filter(|k| !k.is_empty());

        let api_key = match api_key {
            Some(k) => k,
            None => return Ok(Self { handle: None }),
        };

        let agent_cfg = map_langsmith_config(service, cfg, api_key);
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

/// Map muon TOML LangSmith settings onto agent_rs config.
///
/// `service` is the call-site fallback when `cfg.service_name` is empty.
/// Starts from `from_env()` so remaining OTEL/LangSmith env vars still apply,
/// then overlays explicit TOML fields.
pub fn map_langsmith_config(
    service: &str,
    cfg: &LangSmithConfig,
    api_key: String,
) -> agent_rs::domain::observability::LangSmithConfig {
    let mut agent_cfg = agent_rs::domain::observability::LangSmithConfig::from_env();
    agent_cfg.api_key = api_key;
    agent_cfg.service_name = if !cfg.service_name.is_empty() {
        cfg.service_name.clone()
    } else {
        service.to_string()
    };
    if !cfg.project.is_empty() {
        agent_cfg.project = cfg.project.clone();
    }
    if !cfg.endpoint.is_empty() {
        agent_cfg.endpoint = cfg.endpoint.clone();
    }
    agent_cfg.console = cfg.console;
    agent_cfg.batch = cfg.batch;
    agent_cfg
}
