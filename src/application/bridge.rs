use crate::application::pipeline::PipelineStage;
use crate::domain::agents::clarifier::ClarifierResult;
use crate::domain::models::log_entry::LogEntry;

#[derive(Debug)]
pub enum AgentEvent {
    StageChanged(PipelineStage),
    Log(LogEntry),
    ClarifierQuestion {
        question: String,
        responder: tokio::sync::oneshot::Sender<String>,
    },
    ClarificationComplete {
        log: String,
    },
    PlanProposed {
        plan: ClarifierResult,
        responder: tokio::sync::oneshot::Sender<PlanDecision>,
    },
    Completed {
        report: crate::domain::models::report::ResearchReport,
        sources: Vec<crate::domain::models::source::Source>,
        session_id: uuid::Uuid,
    },
    SessionRestored {
        report: Option<crate::domain::models::report::ResearchReport>,
        sources: Vec<crate::domain::models::source::Source>,
        session_id: uuid::Uuid,
    },
    Final(String),
    Error(String),
}

#[derive(Debug, Clone)]
pub enum PlanDecision {
    Approve,
    Reject,
    Feedback(String),
}

#[derive(Clone)]
pub struct BridgeChannels {
    pub events: tokio::sync::mpsc::UnboundedSender<AgentEvent>,
}

impl BridgeChannels {
    pub fn new(events: tokio::sync::mpsc::UnboundedSender<AgentEvent>) -> Self {
        Self { events }
    }

    pub fn log(
        &self,
        tag: crate::domain::models::log_entry::AgentTag,
        level: crate::domain::models::log_entry::LogLevel,
        msg: impl Into<String>,
    ) {
        let _ = self.events.send(AgentEvent::Log(LogEntry {
            timestamp: chrono::Utc::now(),
            agent_tag: tag,
            message: msg.into(),
            level,
        }));
    }

    pub fn stage(&self, s: PipelineStage) {
        let _ = self.events.send(AgentEvent::StageChanged(s));
    }

    pub fn error(&self, msg: impl Into<String>) {
        let _ = self.events.send(AgentEvent::Error(msg.into()));
    }
}
