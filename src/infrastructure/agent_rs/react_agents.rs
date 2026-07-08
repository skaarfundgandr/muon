use async_trait::async_trait;
use rig_core::wasm_compat::{WasmCompatSend, WasmCompatSync};

use crate::application::bridge::BridgeChannels;
use crate::domain::models::log_entry::{AgentTag, LogLevel};
use crate::domain::models::source::SourceType;
use crate::domain::traits::agent::MuonAgent;
use crate::error::{MuonError, Result};
use crate::infrastructure::source_registry::SourceRegistry;

type SourceSink = std::sync::Arc<std::sync::Mutex<SourceRegistry>>;

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
        let trace = self
            .runner
            .prompt_trace(prompt.to_string())
            .await
            .map_err(MuonError::from)?;
        let text = trace.final_answer.map(|fa| fa.text).unwrap_or_default();
        Ok(text)
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
                    b.log(tag, LogLevel::Info, format!("thought: {}", t.reasoning));
                }
            })
            .on_action({
                let b = BridgeChannels::new(self.bridge.events.clone());
                move |a: &agent_rs::domain::agent::Action| {
                    b.log(
                        tag,
                        LogLevel::Info,
                        format!("action: {} args={}", a.tool_name, a.args),
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
                        format!("obs: {} => {}", o.tool_name, o.result),
                    );
                    if !o.is_error
                        && let Ok(urls) =
                            crate::application::pipeline_runner::citation_verifier::extract_urls(
                                &o.result,
                            )
                        && let Ok(mut g) = sink.lock()
                    {
                        for url in &urls {
                            g.record(url.as_str(), SourceType::Web);
                        }
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
                    let msg = format!("error: {e:?}");
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
                    b.log(tag, LogLevel::Info, format!("thought: {}", t.reasoning));
                }
            })
            .on_action({
                let b = BridgeChannels::new(self.bridge.events.clone());
                move |a: &agent_rs::domain::agent::Action| {
                    b.log(
                        tag,
                        LogLevel::Info,
                        format!("action: {} args={}", a.tool_name, a.args),
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
                        format!("obs: {} => {}", o.tool_name, o.result),
                    );
                    if !o.is_error
                        && let Ok(urls) =
                            crate::application::pipeline_runner::citation_verifier::extract_urls(
                                &o.result,
                            )
                        && let Ok(mut g) = sink.lock()
                    {
                        for url in &urls {
                            g.record(url.as_str(), SourceType::Web);
                        }
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
                    let msg = format!("error: {e:?}");
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
