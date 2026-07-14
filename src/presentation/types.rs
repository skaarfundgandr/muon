use std::sync::Arc;

use crate::application::bridge::{AgentEvent, PlanDecision};
use crate::infrastructure::context::InfrastructureContext;
use crossterm::event::MouseEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanApprovalFocus {
    Approve,
    Reject,
    Feedback,
}

#[derive(Debug)]
pub enum ActivePopup {
    EditModels {
        provider_idx: usize,
        focus_idx: usize,
        edit_buffer: Option<String>,
        edit_cursor: usize,
        scroll_offset: usize,
    },
    ConfigureSearch {
        provider_idx: usize,
        focus_idx: usize,
        edit_buffer: Option<String>,
        edit_cursor: usize,
    },
    PlanApproval {
        plan: crate::domain::agents::clarifier::ClarifierResult,
        responder: tokio::sync::oneshot::Sender<crate::application::bridge::PlanDecision>,
        focus: PlanApprovalFocus,
        feedback_buffer: String,
        feedback_cursor: usize,
    },
}

#[derive(Debug)]
pub struct ClarifierPending {
    pub question: String,
    pub responder: tokio::sync::oneshot::Sender<String>,
}

#[derive(Debug)]
pub struct PlanPending {
    pub plan: crate::domain::agents::clarifier::ClarifierResult,
    pub responder: tokio::sync::oneshot::Sender<PlanDecision>,
}

#[derive(Debug)]
pub enum Event {
    Key(crossterm::event::KeyEvent),
    Mouse(MouseEvent),
    Tick,
    AgentEvent(AgentEvent),
    ModelsFetched {
        provider_index: usize,
        result: Result<Vec<crate::application::config::ProviderModel>, String>,
    },
    InfraRebuilt(Result<Arc<InfrastructureContext>, String>),
    SessionDeleteResult {
        id: uuid::Uuid,
        ok: bool,
        error: Option<String>,
        restored: Option<crate::application::session::SessionSummary>,
    },
    RagIndexed {
        idx: usize,
        summary: crate::application::services::IndexSummary,
    },
}
