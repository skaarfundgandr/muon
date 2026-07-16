use async_trait::async_trait;
use rig_core::wasm_compat::{WasmCompatSend, WasmCompatSync};

use crate::application::bridge::BridgeChannels;
use crate::domain::error::{MuonError, Result};
use crate::domain::models::log_entry::{AgentTag, LogLevel};
use crate::domain::models::source::SourceType;
use crate::domain::traits::agent::MuonAgent;
use crate::infrastructure::source_registry::SourceRegistry;

type SourceSink = std::sync::Arc<std::sync::Mutex<SourceRegistry>>;

const FEED_SNIPPET_MAX: usize = 160;

pub const REMINDER_FINALIZE: &str = "\
<system-reminder>\n\
You are approaching the cycle limit.\n\
Finalize your answer now.\n\
</system-reminder>";

pub const REMINDER_ORCHESTRATOR: &str = "\
<system-reminder>\n\
You are approaching the ReAct cycle limit. Stop delegating.\n\
Synthesize a complete markdown research report now from tool\n\
findings already gathered.\n\
</system-reminder>";

pub const REMINDER_CLARIFIER: &str = "\
<system-reminder>\n\
You are approaching the ReAct cycle limit.\n\
Respond with your final strict JSON decision now\n\
(clarification or plan fields only — no markdown report).\n\
</system-reminder>";

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
                let score = item
                    .get("score")
                    .or_else(|| item.get("relevance"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                g.record_with_meta(url, source_type, title, snippet, score);
            }
        }
        return;
    }

    if tool_name == "fetch_page"
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(result)
        && let Some(url) = v.get("url").and_then(|u| u.as_str())
        && !url.is_empty()
    {
        let text = v.get("text").and_then(|t| t.as_str()).unwrap_or("");
        let title = v.get("title").and_then(|t| t.as_str());
        if let Ok(mut g) = sink.lock() {
            g.enrich_page(url, title.unwrap_or(""), text);
        }
        return;
    }

    if let Ok(urls) = crate::application::pipeline_runner::citation_verifier::extract_urls(result)
        && let Ok(mut g) = sink.lock()
    {
        for url in &urls {
            g.record(url.as_str(), SourceType::Web);
        }
    }
}

fn is_cycle_limit_error(e: &agent_rs::domain::errors::ReActError) -> bool {
    matches!(
        e,
        agent_rs::domain::errors::ReActError::MaxCyclesExceeded { .. }
    )
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

struct NoCompactionRunner<M>
where
    M: rig_core::completion::CompletionModel + WasmCompatSend + WasmCompatSync + 'static,
{
    inner: agent_rs::agent::react::BuiltReAct<M, ()>,
}

#[async_trait]
impl<M> ReActRunner for NoCompactionRunner<M>
where
    M: rig_core::completion::CompletionModel + WasmCompatSend + WasmCompatSync + 'static,
{
    async fn prompt_trace(
        &self,
        msg: String,
    ) -> std::result::Result<
        agent_rs::domain::agent::ReActTrace,
        agent_rs::domain::errors::ReActError,
    > {
        self.inner.prompt(msg).await
    }

    async fn chat_prompt(
        &self,
        msg: String,
        history: &mut Vec<rig_core::message::Message>,
    ) -> std::result::Result<String, agent_rs::domain::errors::ReActError> {
        self.inner.chat(msg, history).await
    }
}

struct CompactionRunner<M, C>
where
    M: rig_core::completion::CompletionModel + WasmCompatSend + WasmCompatSync + 'static,
    C: rig_core::completion::Prompt + WasmCompatSend + WasmCompatSync + 'static,
{
    inner: agent_rs::agent::react::BuiltReAct<M, C>,
}

#[async_trait]
impl<M, C> ReActRunner for CompactionRunner<M, C>
where
    M: rig_core::completion::CompletionModel + WasmCompatSend + WasmCompatSync + 'static,
    C: rig_core::completion::Prompt + WasmCompatSend + WasmCompatSync + 'static,
{
    async fn prompt_trace(
        &self,
        msg: String,
    ) -> std::result::Result<
        agent_rs::domain::agent::ReActTrace,
        agent_rs::domain::errors::ReActError,
    > {
        self.inner.prompt_compact(msg).await
    }

    async fn chat_prompt(
        &self,
        msg: String,
        history: &mut Vec<rig_core::message::Message>,
    ) -> std::result::Result<String, agent_rs::domain::errors::ReActError> {
        self.inner.chat_compact(msg, history).await
    }
}

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

        let result = async { self.runner.prompt_trace(prompt.to_string()).await }
            .instrument(span.clone())
            .await;

        match result {
            Ok(trace) => {
                let text = trace.final_answer.map(|fa| fa.text).unwrap_or_default();
                let otel_out =
                    crate::infrastructure::observability::otel_attr_value(text.as_str());
                span.record("output.value", otel_out.as_str());
                Ok(text)
            }
            Err(e) => Err(map_react_prompt_error(self.tag, e)),
        }
    }
}

fn map_react_prompt_error(tag: AgentTag, err: agent_rs::domain::errors::ReActError) -> MuonError {
    use agent_rs::domain::errors::ReActError;
    match err {
        ReActError::MaxCyclesExceeded { cycles } => MuonError::MaxCycles {
            agent: tag.as_str().to_string(),
            cycles,
        },
        ReActError::NoToolCallsAndNoFinalAnswer { cycle } => {
            let message = format!("empty output (cycle {cycle}); check max_tokens");
            tracing::warn!(
                target: "muon::agent",
                agent = tag.as_str(),
                cycle,
                "{message}"
            );
            MuonError::Agent {
                agent: tag.as_str().to_string(),
                message,
            }
        }
        other => MuonError::Agent {
            agent: tag.as_str().to_string(),
            message: other.to_string(),
        },
    }
}

pub struct ReActFactory<'a> {
    pub cfg: &'a crate::application::config::MuonConfig,
    pub bridge: BridgeChannels,
}

impl<'a> ReActFactory<'a> {
    pub fn new(cfg: &'a crate::application::config::MuonConfig, bridge: BridgeChannels) -> Self {
        Self { cfg, bridge }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn build_runner<M>(
        &self,
        agent: rig_core::agent::Agent<M>,
        tag: AgentTag,
        max_cycles: usize,
        tool_timeout_secs: u64,
        source_sink: SourceSink,
        cycle_reminder: Option<&str>,
        react_tool_feed: bool,
    ) -> Box<dyn ReActRunner>
    where
        M: rig_core::completion::CompletionModel + WasmCompatSend + WasmCompatSync + 'static,
        rig_core::agent::Agent<M>: Clone,
    {
        use agent_rs::agent::ReActExt;

        let reminder = cycle_reminder.map(str::to_string);
        let mut builder = agent
            .react()
            .max_cycles(max_cycles)
            .tool_timeout_secs(tool_timeout_secs)
            .set_cycle_limit_reminder_msg(reminder)
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
            .on_final({
                let b = BridgeChannels::new(self.bridge.events.clone());
                move |_f: &agent_rs::domain::agent::FinalAnswer| {
                    b.log(tag, LogLevel::Info, "final answer".to_string());
                }
            })
            .on_error({
                let b = BridgeChannels::new(self.bridge.events.clone());
                move |e: &agent_rs::domain::errors::ReActError| {
                    let msg = format!("error: {}", feed_snippet(&e.to_string()));
                    let level = if is_cycle_limit_error(e) {
                        LogLevel::Warn
                    } else {
                        LogLevel::Error
                    };
                    b.log(tag, level, msg);
                }
            })
            .with_span_emitter(crate::infrastructure::observability::Observability::span_emitter());

        if react_tool_feed {
            builder = builder
                .on_action({
                    let b = BridgeChannels::new(self.bridge.events.clone());
                    move |a: &agent_rs::domain::agent::Action| {
                        b.log(
                            tag,
                            LogLevel::Info,
                            format!("action: {} args={}", a.tool_name, feed_snippet(&a.args)),
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
                });
        }

        Box::new(NoCompactionRunner {
            inner: builder.build(),
        })
    }

    pub fn build_planner_runner<M>(
        &self,
        agent: rig_core::agent::Agent<M>,
        tag: AgentTag,
        max_cycles: usize,
        tool_timeout_secs: u64,
        source_sink: SourceSink,
    ) -> Box<dyn ReActRunner>
    where
        M: rig_core::completion::CompletionModel + WasmCompatSend + WasmCompatSync + 'static,
        rig_core::agent::Agent<M>: Clone,
    {
        self.build_runner(
            agent,
            tag,
            max_cycles,
            tool_timeout_secs,
            source_sink,
            Some(REMINDER_FINALIZE),
            true,
        )
    }

    pub fn build_researcher_runner<M>(
        &self,
        agent: rig_core::agent::Agent<M>,
        tag: AgentTag,
        max_cycles: usize,
        tool_timeout_secs: u64,
        source_sink: SourceSink,
    ) -> Box<dyn ReActRunner>
    where
        M: rig_core::completion::CompletionModel + WasmCompatSend + WasmCompatSync + 'static,
        rig_core::agent::Agent<M>: Clone,
    {
        self.build_runner(
            agent,
            tag,
            max_cycles,
            tool_timeout_secs,
            source_sink,
            Some(REMINDER_FINALIZE),
            true,
        )
    }

    pub fn build_clarifier_runner<M>(
        &self,
        agent: rig_core::agent::Agent<M>,
        tag: AgentTag,
        max_cycles: usize,
        tool_timeout_secs: u64,
        threshold: usize,
        source_sink: SourceSink,
    ) -> Box<dyn ReActRunner>
    where
        M: rig_core::completion::CompletionModel + WasmCompatSend + WasmCompatSync + 'static,
        rig_core::agent::Agent<M>: Clone,
    {
        use agent_rs::agent::ReActExt;

        let built = agent
            .react()
            .max_cycles(max_cycles)
            .tool_timeout_secs(tool_timeout_secs)
            .set_cycle_limit_reminder_msg(Some(REMINDER_CLARIFIER.to_string()))
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
                        format!("action: {} args={}", a.tool_name, feed_snippet(&a.args)),
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
                    let msg = format!("error: {}", feed_snippet(&e.to_string()));
                    let level = if is_cycle_limit_error(e) {
                        LogLevel::Warn
                    } else {
                        LogLevel::Error
                    };
                    b.log(tag, level, msg);
                }
            })
            .with_span_emitter(crate::infrastructure::observability::Observability::span_emitter())
            .build();
        Box::new(CompactionRunner { inner: built })
    }
}
