use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::error::MuonError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PipelineStage {
    #[default]
    Idle,
    IntentClassification,
    Clarification,
    ShallowResearch,
    DeepResearch,
    CitationVerify,
    Report,
    Complete,
    Cancelled,
    Failed,
}

impl PipelineStage {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::IntentClassification => "IntentClassification",
            Self::Clarification => "Clarification",
            Self::ShallowResearch => "ShallowResearch",
            Self::DeepResearch => "DeepResearch",
            Self::CitationVerify => "CitationVerify",
            Self::Report => "Report",
            Self::Complete => "Complete",
            Self::Cancelled => "Cancelled",
            Self::Failed => "Failed",
        }
    }

    pub fn parse_stage(s: &str) -> Result<Self, MuonError> {
        match s {
            "Idle" => Ok(Self::Idle),
            "IntentClassification" => Ok(Self::IntentClassification),
            "Clarification" => Ok(Self::Clarification),
            "ShallowResearch" => Ok(Self::ShallowResearch),
            "DeepResearch" => Ok(Self::DeepResearch),
            "CitationVerify" => Ok(Self::CitationVerify),
            "Report" => Ok(Self::Report),
            "Complete" => Ok(Self::Complete),
            "Cancelled" => Ok(Self::Cancelled),
            "Failed" => Ok(Self::Failed),
            other => Err(MuonError::Database(format!(
                "unknown pipeline stage: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PipelineState {
    pub stage: PipelineStage,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub current_step: u64,
    pub total_steps: u64,
}

impl Default for PipelineState {
    fn default() -> Self {
        Self {
            stage: PipelineStage::Idle,
            started_at: None,
            completed_at: None,
            current_step: 0,
            total_steps: 5,
        }
    }
}

impl PipelineState {
    pub fn start(&mut self) {
        self.stage = PipelineStage::Idle;
        self.started_at = Some(Utc::now());
        self.completed_at = None;
        self.current_step = 0;
    }

    pub fn set_stage(&mut self, s: PipelineStage) {
        self.stage = s;
        self.current_step = self.current_step.saturating_add(1);
        if matches!(
            s,
            PipelineStage::Complete | PipelineStage::Cancelled | PipelineStage::Failed
        ) {
            self.completed_at = Some(Utc::now());
        }
    }

    pub fn finish(&mut self) {
        self.stage = PipelineStage::Complete;
        self.completed_at = Some(Utc::now());
    }

    pub fn cancel(&mut self) {
        self.stage = PipelineStage::Cancelled;
        self.completed_at = Some(Utc::now());
    }

    pub fn fail(&mut self) {
        self.stage = PipelineStage::Failed;
        self.completed_at = Some(Utc::now());
    }

    pub fn elapsed_secs(&self) -> u64 {
        match self.started_at {
            Some(start) => {
                let end = self.completed_at.unwrap_or_else(Utc::now);
                (end - start).num_seconds().max(0) as u64
            }
            None => 0,
        }
    }

    pub fn is_running(&self) -> bool {
        self.started_at.is_some() && self.completed_at.is_none()
    }

    pub fn clone_state_for_task(&self) -> Self {
        self.clone()
    }
}
