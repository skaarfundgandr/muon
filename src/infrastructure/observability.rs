use std::sync::atomic::{AtomicBool, Ordering};

use agent_rs::observability::{TracerHandle, init_tracing, shutdown_tracing};

use crate::application::config::{LangSmithConfig, ObservabilityConfig};
use crate::domain::error::{MuonError, Result};

static OTEL_DEBUG: AtomicBool = AtomicBool::new(false);

const OTEL_ATTR_MAX_CHARS: usize = 16_384;

pub fn set_otel_debug(debug: bool) {
    OTEL_DEBUG.store(debug, Ordering::Relaxed);
}

pub fn otel_debug() -> bool {
    OTEL_DEBUG.load(Ordering::Relaxed)
}

pub fn otel_attr_value(s: &str) -> String {
    otel_attr_value_with(s, otel_debug())
}

pub fn otel_attr_value_with(s: &str, debug: bool) -> String {
    if debug {
        return s.to_string();
    }
    let count = s.chars().count();
    if count <= OTEL_ATTR_MAX_CHARS {
        return s.to_string();
    }
    let mut out: String = s.chars().take(OTEL_ATTR_MAX_CHARS).collect();
    out.push('…');
    out
}

pub struct Observability {
    handle: Option<TracerHandle>,
}

impl Observability {
    pub fn init(service: &str, cfg: &ObservabilityConfig) -> Result<Self> {
        set_otel_debug(cfg.debug);

        let api_key = crate::infrastructure::config::expand_env(&cfg.langsmith.api_key)
            .ok()
            .filter(|k| !k.is_empty())
            .or_else(|| std::env::var("LANGSMITH_API_KEY").ok())
            .filter(|k| !k.is_empty());

        let Some(api_key) = api_key else {
            return Ok(Self { handle: None });
        };

        let agent_cfg = map_langsmith_config(service, &cfg.langsmith, api_key);
        let handle = init_tracing(&agent_cfg).map_err(|e| MuonError::Pipeline(e.to_string()))?;
        Ok(Self {
            handle: Some(handle),
        })
    }

    pub fn span_emitter() -> std::sync::Arc<dyn agent_rs::agent::react::ReActSpanEmitter> {
        std::sync::Arc::new(agent_rs::observability::react_spans::LangSmithReActEmitter)
    }

    pub async fn shutdown(self) -> Result<()> {
        if let Some(handle) = self.handle {
            shutdown_tracing(handle)
                .await
                .map_err(|e| MuonError::Pipeline(e.to_string()))?;
        }
        Ok(())
    }
}

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
    agent_cfg.batch_delay_ms = cfg.batch_delay_ms;
    agent_cfg
}
