use async_trait::async_trait;
use rig_core::wasm_compat::{WasmCompatSend, WasmCompatSync};

use crate::application::bridge::BridgeChannels;
use crate::domain::models::log_entry::{AgentTag, LogLevel};
use crate::domain::models::source::SourceType;
use crate::domain::traits::agent::MuonAgent;
use crate::error::{MuonError, Result};
use crate::infrastructure::source_registry::SourceRegistry;

type SourceSink = std::sync::Arc<std::sync::Mutex<SourceRegistry>>;

const FEED_SNIPPET_MAX: usize = 160;

pub(crate) fn feed_snippet(s: &str) -> String {
    let count = s.chars().count();
    if count <= FEED_SNIPPET_MAX {
        return s.to_string();
    }
    let mut out: String = s.chars().take(FEED_SNIPPET_MAX).collect();
    out.push('…');
    out
}

pub(crate) fn record_observation_sources(sink: &SourceSink, tool_name: &str, result: &str) {
    if matches!(tool_name, "web_search" | "paper_search")
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(result)
        && let Some(results) = v.get("results").and_then(|r| r.as_array())
    {
        let source_type = if tool_name == "paper_search" {
            SourceType::Paper
        } else {
            SourceType::Web
        };
        if let Ok(mut g) = sink.lock() {
            for item in results {
                let Some(url) = item.get("url").and_then(|u| u.as_str()) else {
                    continue;
                };
                if url.is_empty() {
                    continue;
                }
                let title = item.get("title").and_then(|t| t.as_str()).unwrap_or("");
                let snippet = item.get("snippet").and_then(|s| s.as_str()).unwrap_or("");
                g.record_with_meta(url, source_type, title, snippet);
            }
        }
        return;
    }

    if let Ok(urls) =
        crate::application::pipeline_runner::citation_verifier::extract_urls(result)
        && let Ok(mut g) = sink.lock()
    {
        for url in &urls {
            g.record(url.as_str(), SourceType::Web);
        }
    }
}

#[async_trait]
pub trait ReActRunner: Send + Sync {
    async fn prompt_trace(
        &self,
        msg: String,
    ) -> std::result::Result<
        agent_rs::domain::agent::ReActTrace,
        agent_rs::domain::errors::ReActError,
    >;
    #[allow(dead_code)]
    async fn chat_prompt(
        &self,
        msg: String,
        history: &mut Vec<rig_core::message::Message>,
    ) -> std::result::Result<String, agent_rs::domain::errors::ReActError>;
}

struct NoCompactionRunner<M, P>(agent_rs::agent::react::BuiltReAct<M, P, ()>)
where
    M: rig_core::completion::CompletionModel + WasmCompatSend + WasmCompatSync + 'static,
    P: rig_core::agent::PromptHook<M> + WasmCompatSend + WasmCompatSync + 'static;

#[async_trait]
impl<M, P> ReActRunner for NoCompactionRunner<M, P>
where
    M: rig_core::completion::CompletionModel + WasmCompatSend + WasmCompatSync + 'static,
    P: rig_core::agent::PromptHook<M> + WasmCompatSend + WasmCompatSync + 'static,
{
    async fn prompt_trace(
        &self,
        msg: String,
    ) -> std::result::Result<
        agent_rs::domain::agent::ReActTrace,
        agent_rs::domain::errors::ReActError,
    > {
        self.0.prompt(msg).await
    }

    async fn chat_prompt(
        &self,
        msg: String,
        history: &mut Vec<rig_core::message::Message>,
    ) -> std::result::Result<String, agent_rs::domain::errors::ReActError> {
        self.0.chat(msg, history).await
    }
}

struct CompactionRunner<M, P, C>(agent_rs::agent::react::BuiltReAct<M, P, C>)
where
    M: rig_core::completion::CompletionModel + WasmCompatSend + WasmCompatSync + 'static,
    P: rig_core::agent::PromptHook<M> + WasmCompatSend + WasmCompatSync + 'static,
    C: rig_core::completion::Prompt + WasmCompatSend + WasmCompatSync + 'static;

#[async_trait]
impl<M, P, C> ReActRunner for CompactionRunner<M, P, C>
where
    M: rig_core::completion::CompletionModel + WasmCompatSend + WasmCompatSync + 'static,
    P: rig_core::agent::PromptHook<M> + WasmCompatSend + WasmCompatSync + 'static,
    C: rig_core::completion::Prompt + WasmCompatSend + WasmCompatSync + 'static,
{
    async fn prompt_trace(
        &self,
        msg: String,
    ) -> std::result::Result<
        agent_rs::domain::agent::ReActTrace,
        agent_rs::domain::errors::ReActError,
    > {
        self.0.prompt_compact(msg).await
    }

    async fn chat_prompt(
        &self,
        msg: String,
        history: &mut Vec<rig_core::message::Message>,
    ) -> std::result::Result<String, agent_rs::domain::errors::ReActError> {
        self.0.chat_compact(msg, history).await
    }
}

/// Type-erased ReAct agent implementing `MuonAgent` via `agent_rs`.
pub struct ReActAgent {
    tag: AgentTag,
    runner: Box<dyn ReActRunner>,
    #[allow(dead_code)]
    bridge: BridgeChannels,
}

impl ReActAgent {
    pub fn new(tag: AgentTag, runner: Box<dyn ReActRunner>, bridge: BridgeChannels) -> Self {
        Self {
            tag,
            runner,
            bridge,
        }
    }
}

#[async_trait]
impl MuonAgent for ReActAgent {
    fn tag(&self) -> AgentTag {
        self.tag
    }

    async fn prompt_raw(&self, prompt: &str) -> Result<String> {
        use agent_rs::observability::conventions::KIND_AGENT;
        use tracing::Instrument;

        let span = tracing::info_span!(
            "react_agent",
            "langsmith.span.kind" = KIND_AGENT,
            "openinference.span.kind" = "AGENT",
            "input.value" = prompt,
            "output.value" = tracing::field::Empty,
            "agent.tag" = self.tag.as_str(),
        );

        let result = async {
            self.runner
                .prompt_trace(prompt.to_string())
                .await
                .map_err(MuonError::from)
        }
        .instrument(span.clone())
        .await;

        match result {
            Ok(trace) => {
                let text = trace.final_answer.map(|fa| fa.text).unwrap_or_default();
                span.record("output.value", text.as_str());
                Ok(text)
            }
            Err(e) => Err(e),
        }
    }
}

/// Builds `ReActAgent` instances from pre-constructed rig `Agent<M, P>` objects.
pub struct ReActFactory<'a> {
    pub cfg: &'a crate::config::MuonConfig,
    pub bridge: BridgeChannels,
}

impl<'a> ReActFactory<'a> {
    pub fn new(cfg: &'a crate::config::MuonConfig, bridge: BridgeChannels) -> Self {
        Self { cfg, bridge }
    }

    /// Build a standard ReAct runner (no compaction) from a pre-built rig Agent.
    pub fn build_runner<M, P>(
        &self,
        agent: rig_core::agent::Agent<M, P>,
        tag: AgentTag,
        max_cycles: usize,
        tool_timeout_secs: u64,
        source_sink: SourceSink,
    ) -> Box<dyn ReActRunner>
    where
        M: rig_core::completion::CompletionModel + WasmCompatSend + WasmCompatSync + 'static,
        P: rig_core::agent::PromptHook<M> + WasmCompatSend + WasmCompatSync + 'static,
        rig_core::agent::Agent<M, P>: Clone,
    {
        use agent_rs::agent::ReActExt;

        let built = agent
            .react()
            .max_cycles(max_cycles)
            .tool_timeout_secs(tool_timeout_secs)
            .on_thought({
                let b = BridgeChannels::new(self.bridge.events.clone());
                move |t: &agent_rs::domain::agent::Thought| {
                    b.log(
                        tag,
                        LogLevel::Info,
                        format!("thought: {}", feed_snippet(&t.reasoning)),
                    );
                }
            })
            .on_action({
                let b = BridgeChannels::new(self.bridge.events.clone());
                move |a: &agent_rs::domain::agent::Action| {
                    b.log(
                        tag,
                        LogLevel::Info,
                        format!(
                            "action: {} args={}",
                            a.tool_name,
                            feed_snippet(&a.args)
                        ),
                    );
                }
            })
            .on_observation({
                let b = BridgeChannels::new(self.bridge.events.clone());
                let sink = source_sink.clone();
                move |o: &agent_rs::domain::agent::Observation| {
                    b.log(
                        tag,
                        if o.is_error {
                            LogLevel::Warn
                        } else {
                            LogLevel::Info
                        },
                        format!("obs: {} => {}", o.tool_name, feed_snippet(&o.result)),
                    );
                    if !o.is_error {
                        record_observation_sources(&sink, &o.tool_name, &o.result);
                    }
                }
            })
            .on_final({
                let b = BridgeChannels::new(self.bridge.events.clone());
                move |_f: &agent_rs::domain::agent::FinalAnswer| {
                    b.log(tag, LogLevel::Info, "final answer".to_string());
                }
            })
            .on_error({
                let b = BridgeChannels::new(self.bridge.events.clone());
                move |e: &agent_rs::domain::errors::ReActError| {
                    let msg = format!("error: {e}");
                    let is_turn_limit = msg.to_lowercase().contains("max_turns")
                        || msg.to_lowercase().contains("max turns");
                    let level = if is_turn_limit { LogLevel::Warn } else { LogLevel::Error };
                    b.log(tag, level, msg);
                }
            })
            .with_span_emitter(crate::infrastructure::observability::Observability::span_emitter())
            .build();
        Box::new(NoCompactionRunner(built))
    }

    pub fn build_planner_runner<M, P>(
        &self,
        agent: rig_core::agent::Agent<M, P>,
        tag: AgentTag,
        max_cycles: usize,
        tool_timeout_secs: u64,
        source_sink: SourceSink,
    ) -> Box<dyn ReActRunner>
    where
        M: rig_core::completion::CompletionModel + WasmCompatSend + WasmCompatSync + 'static,
        P: rig_core::agent::PromptHook<M> + WasmCompatSend + WasmCompatSync + 'static,
        rig_core::agent::Agent<M, P>: Clone,
    {
        self.build_runner(agent, tag, max_cycles, tool_timeout_secs, source_sink)
    }

    pub fn build_researcher_runner<M, P>(
        &self,
        agent: rig_core::agent::Agent<M, P>,
        tag: AgentTag,
        max_cycles: usize,
        tool_timeout_secs: u64,
        source_sink: SourceSink,
    ) -> Box<dyn ReActRunner>
    where
        M: rig_core::completion::CompletionModel + WasmCompatSend + WasmCompatSync + 'static,
        P: rig_core::agent::PromptHook<M> + WasmCompatSend + WasmCompatSync + 'static,
        rig_core::agent::Agent<M, P>: Clone,
    {
        self.build_runner(agent, tag, max_cycles, tool_timeout_secs, source_sink)
    }

    /// Build a clarifier runner with automatic context compaction for multi-turn history.
    pub fn build_clarifier_runner<M, P>(
        &self,
        agent: rig_core::agent::Agent<M, P>,
        tag: AgentTag,
        max_cycles: usize,
        tool_timeout_secs: u64,
        threshold: usize,
        source_sink: SourceSink,
    ) -> Box<dyn ReActRunner>
    where
        M: rig_core::completion::CompletionModel + WasmCompatSend + WasmCompatSync + 'static,
        P: rig_core::agent::PromptHook<M> + WasmCompatSend + WasmCompatSync + 'static,
        rig_core::agent::Agent<M, P>: Clone,
    {
        use agent_rs::agent::ReActExt;

        let built = agent
            .react()
            .max_cycles(max_cycles)
            .tool_timeout_secs(tool_timeout_secs)
            .with_compaction()
            .threshold(threshold)
            .on_thought({
                let b = BridgeChannels::new(self.bridge.events.clone());
                move |t: &agent_rs::domain::agent::Thought| {
                    b.log(
                        tag,
                        LogLevel::Info,
                        format!("thought: {}", feed_snippet(&t.reasoning)),
                    );
                }
            })
            .on_action({
                let b = BridgeChannels::new(self.bridge.events.clone());
                move |a: &agent_rs::domain::agent::Action| {
                    b.log(
                        tag,
                        LogLevel::Info,
                        format!(
                            "action: {} args={}",
                            a.tool_name,
                            feed_snippet(&a.args)
                        ),
                    );
                }
            })
            .on_observation({
                let b = BridgeChannels::new(self.bridge.events.clone());
                let sink = source_sink.clone();
                move |o: &agent_rs::domain::agent::Observation| {
                    b.log(
                        tag,
                        if o.is_error {
                            LogLevel::Warn
                        } else {
                            LogLevel::Info
                        },
                        format!("obs: {} => {}", o.tool_name, feed_snippet(&o.result)),
                    );
                    if !o.is_error {
                        record_observation_sources(&sink, &o.tool_name, &o.result);
                    }
                }
            })
            .on_final({
                let b = BridgeChannels::new(self.bridge.events.clone());
                move |_f: &agent_rs::domain::agent::FinalAnswer| {
                    b.log(tag, LogLevel::Info, "final answer".to_string());
                }
            })
            .on_error({
                let b = BridgeChannels::new(self.bridge.events.clone());
                move |e: &agent_rs::domain::errors::ReActError| {
                    let msg = format!("error: {e}");
                    let is_turn_limit = msg.to_lowercase().contains("max_turns")
                        || msg.to_lowercase().contains("max turns");
                    let level = if is_turn_limit { LogLevel::Warn } else { LogLevel::Error };
                    b.log(tag, level, msg);
                }
            })
            .with_span_emitter(crate::infrastructure::observability::Observability::span_emitter())
            .build();
        Box::new(CompactionRunner(built))
    }
}
