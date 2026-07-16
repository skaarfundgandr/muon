use async_trait::async_trait;
use rig_core::agent::{AgentHook, Flow, HookContext, StepEvent};
use rig_core::completion::{CompletionModel, PromptError};
use rig_core::message::{AssistantContent, Message, ToolResultContent, UserContent};
use rig_core::wasm_compat::{WasmCompatSend, WasmCompatSync};

use crate::application::bridge::BridgeChannels;
use crate::domain::error::{MuonError, Result};
use crate::domain::models::log_entry::{AgentTag, LogLevel};
use crate::domain::traits::agent::MuonAgent;
use crate::infrastructure::agent_rs::react_agents::{feed_snippet, record_observation_sources};
use crate::infrastructure::source_registry::SourceRegistry;

type SourceSink = std::sync::Arc<std::sync::Mutex<SourceRegistry>>;

#[derive(Clone)]
pub struct ResearcherHook {
    bridge: BridgeChannels,
    tag: AgentTag,
    sink: SourceSink,
}

impl<M> AgentHook<M> for ResearcherHook
where
    M: CompletionModel,
{
    async fn on_event(&self, _ctx: &HookContext, event: StepEvent<'_, M>) -> Flow {
        match event {
            StepEvent::ToolCall { tool_name, args, .. } => {
                self.bridge.log(
                    self.tag,
                    LogLevel::Info,
                    format!("action: {tool_name} args={}", feed_snippet(args)),
                );
                Flow::Continue
            }
            StepEvent::ToolResult { tool_name, result, .. } => {
                self.bridge.log(
                    self.tag,
                    LogLevel::Info,
                    format!("obs: {tool_name} => {}", feed_snippet(result)),
                );
                record_observation_sources(&self.sink, tool_name, result);
                Flow::Continue
            }
            _ => Flow::Continue,
        }
    }
}

#[async_trait]
trait ManagedRunner: Send + Sync {
    async fn run(&self, prompt: &str) -> std::result::Result<String, PromptError>;
}

struct BuiltManagedRunner<M>(agent_rs::agent::BuiltManagedAgent<M, ()>)
where
    M: CompletionModel + WasmCompatSend + WasmCompatSync + 'static;

#[async_trait]
impl<M> ManagedRunner for BuiltManagedRunner<M>
where
    M: CompletionModel + WasmCompatSend + WasmCompatSync + 'static,
{
    async fn run(&self, prompt: &str) -> std::result::Result<String, PromptError> {
        self.0.prompt(prompt).await
    }
}

pub struct ManagedAgent {
    tag: AgentTag,
    runner: Box<dyn ManagedRunner>,
    bridge: BridgeChannels,
    timeout_secs: u64,
}

impl ManagedAgent {
    pub fn from_rig_agent_with_hook<M>(
        tag: AgentTag,
        agent: rig_core::agent::Agent<M>,
        bridge: BridgeChannels,
        timeout_secs: u64,
    ) -> Self
    where
        M: CompletionModel + WasmCompatSend + WasmCompatSync + 'static,
        rig_core::agent::Agent<M>: Clone,
    {
        let managed = agent_rs::agent::ManagedExt::managed(&agent)
            .max_retries(2)
            .build();
        Self {
            tag,
            runner: Box::new(BuiltManagedRunner(managed)),
            bridge,
            timeout_secs,
        }
    }
}

pub fn researcher_hook(bridge: BridgeChannels, tag: AgentTag, sink: SourceSink) -> ResearcherHook {
    ResearcherHook { bridge, tag, sink }
}

fn extract_text_from_history(history: &[Message]) -> String {
    let mut assistant_texts = Vec::new();
    let mut tool_bits = Vec::new();

    for msg in history {
        match msg {
            Message::Assistant { content, .. } => {
                for item in content.iter() {
                    if let AssistantContent::Text(t) = item
                        && !t.text.trim().is_empty()
                    {
                        assistant_texts.push(t.text.clone());
                    }
                }
            }
            Message::User { content } => {
                for item in content.iter() {
                    if let UserContent::ToolResult(tr) = item {
                        for part in tr.content.iter() {
                            if let ToolResultContent::Text(t) = part
                                && !t.text.trim().is_empty()
                            {
                                tool_bits.push(t.text.clone());
                            }
                        }
                    }
                }
            }
            Message::System { .. } => {}
        }
    }

    if let Some(last) = assistant_texts
        .into_iter()
        .rev()
        .find(|s| !s.trim().is_empty())
    {
        return last;
    }
    if !tool_bits.is_empty() {
        return format!(
            "## Partial research findings (turn limit reached)\n\n\
             Tool observations gathered before the turn limit:\n\n{}",
            tool_bits.join("\n\n---\n\n")
        );
    }
    "## Partial research findings\n\n\
     The researcher reached its tool-call limit before producing a final answer. \
     Insufficient structured findings were available."
        .to_string()
}

#[async_trait]
impl MuonAgent for ManagedAgent {
    fn tag(&self) -> AgentTag {
        self.tag
    }

    async fn prompt_raw(&self, prompt: &str) -> Result<String> {
        use agent_rs::observability::conventions::KIND_AGENT;
        use tracing::Instrument;

        let span = tracing::info_span!(
            "managed_agent",
            "langsmith.span.kind" = KIND_AGENT,
            "openinference.span.kind" = "AGENT",
            "input.value" = prompt,
            "output.value" = tracing::field::Empty,
            "agent.tag" = self.tag.as_str(),
        );

        let timeout_dur = std::time::Duration::from_secs(self.timeout_secs.max(1));
        match tokio::time::timeout(timeout_dur, async {
            let result = async { self.runner.run(prompt).await }
                .instrument(span.clone())
                .await;

            match result {
                Ok(text) => {
                    let text = if text.trim().is_empty() {
                        "## Research findings\n\nNo content returned by the researcher.".to_string()
                    } else {
                        text
                    };
                    let otel_out =
                        crate::infrastructure::observability::otel_attr_value(text.as_str());
                    span.record("output.value", otel_out.as_str());
                    self.bridge
                        .log(self.tag, LogLevel::Info, "final answer".to_string());
                    Ok(text)
                }
                Err(PromptError::MaxTurnsError {
                    max_turns,
                    chat_history,
                    ..
                }) => {
                    self.bridge.log(
                        self.tag,
                        LogLevel::Warn,
                        format!(
                            "researcher hit tool-call limit ({max_turns}); returning partial findings"
                        ),
                    );
                    let salvaged = extract_text_from_history(&chat_history);
                    let otel_out =
                        crate::infrastructure::observability::otel_attr_value(salvaged.as_str());
                    span.record("output.value", otel_out.as_str());
                    Ok(salvaged)
                }
                Err(e) => {
                    self.bridge
                        .log(self.tag, LogLevel::Error, format!("error: {e}"));
                    Err(MuonError::Agent {
                        agent: self.tag.as_str().to_string(),
                        message: e.to_string(),
                    })
                }
            }
        })
        .await
        {
            Ok(inner) => inner,
            Err(_) => Err(MuonError::Timeout {
                agent: self.tag.as_str().to_string(),
            }),
        }
    }
}
