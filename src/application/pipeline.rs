use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineStage {
    Idle,
    IntentClassification,
    Clarification,
    ShallowResearch,
    DeepResearch,
    Complete,
    Cancelled,
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
        self.stage = PipelineStage::IntentClassification;
        self.started_at = Some(Utc::now());
        self.completed_at = None;
        self.current_step = 1;
    }

    pub fn advance(&mut self) {
        self.stage = match self.stage {
            PipelineStage::Idle | PipelineStage::Cancelled => PipelineStage::IntentClassification,
            PipelineStage::IntentClassification => PipelineStage::Clarification,
            PipelineStage::Clarification => PipelineStage::ShallowResearch,
            PipelineStage::ShallowResearch => PipelineStage::DeepResearch,
            PipelineStage::DeepResearch => PipelineStage::Complete,
            PipelineStage::Complete => PipelineStage::Complete,
        };
        self.current_step += 1;
        if self.stage == PipelineStage::Complete {
            self.completed_at = Some(Utc::now());
        }
    }

    pub fn cancel(&mut self) {
        self.stage = PipelineStage::Cancelled;
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
        !matches!(
            self.stage,
            PipelineStage::Idle | PipelineStage::Complete | PipelineStage::Cancelled
        )
    }
}
