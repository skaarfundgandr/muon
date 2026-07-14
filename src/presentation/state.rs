use std::sync::Arc;
use std::time::Instant;

use tokio::sync::mpsc;

use crate::application::bridge::{AgentEvent, BridgeChannels};
use crate::application::pipeline::{PipelineStage, PipelineState};
use crate::application::services::{ExportFormat, ExportRequest, ExportService};
use crate::application::session::SessionService;
use crate::application::config::{AgentSettings, MuonConfig};
use crate::domain::error::MuonError;
use crate::domain::models::log_entry::{AgentTag, LogEntry, LogLevel};
use crate::domain::models::session::{Session, SessionStatus};
use crate::infrastructure::context::InfrastructureContext;
use crate::presentation::click::ClickTarget;
use crate::presentation::components::query_input::QueryInput;
use crate::presentation::form::FormState;
use crate::presentation::views::{View, ViewRouter};

use super::clipboard::{HeldClipboard, copy_text_to_clipboard};
use super::types::{ActivePopup, ClarifierPending, Event, PlanPending};

const LIVE_FEED_MAX: usize = 50;
const LIVE_FEED_MSG_MAX: usize = 200;

fn truncate_feed_msg(mut msg: String) -> String {
    if msg.chars().count() > LIVE_FEED_MSG_MAX {
        msg = msg.chars().take(LIVE_FEED_MSG_MAX).collect();
        msg.push('…');
    }
    msg
}

#[derive(Debug)]
pub struct AppState {
    pub router: ViewRouter,
    pub running: bool,
    pub(crate) tick_count: u64,
    pub config: MuonConfig,
    pub agent_settings: AgentSettings,
    pub forms: [FormState; 6],
    pub query_input: QueryInput,
    pub sessions: SessionService,
    pub pipeline: PipelineState,
    pub mouse_col: u16,
    pub mouse_row: u16,
    pub term_cols: u16,
    pub term_rows: u16,
    pub hit_registry: Vec<ClickTarget>,
    pub clarifier_pending: Option<ClarifierPending>,
    pub clarifier_response: String,
    pub plan_pending: Option<PlanPending>,
    pub agent_tx: Option<mpsc::UnboundedSender<AgentEvent>>,
    pub infra: Option<Arc<InfrastructureContext>>,
    pub config_reload_rx: Option<mpsc::Receiver<MuonConfig>>,
    pub active_popup: Option<ActivePopup>,
    pub last_report: Option<crate::domain::models::report::ResearchReport>,
    pub last_sources: Vec<crate::domain::models::source::Source>,
    pub live_feed_entries: Vec<LogEntry>,
    pub last_clarifier_log: String,
    pub clarifier_focused: bool,
    pub live_feed_scroll: usize,
    pub report_scroll: usize,
    pub source_scroll: usize,
    pub session_scroll: usize,
    pub full_report_mode: bool,
    pub status_flash: Option<(
        Instant,
        String,
        crate::presentation::components::chrome::toast::ToastKind,
    )>,
    pub(crate) clipboard: Option<HeldClipboard>,
    pub export_session_id: Option<uuid::Uuid>,
    pub(crate) pipeline_handle: Option<tokio::task::JoinHandle<()>>,
    pub(crate) event_tx: Option<mpsc::UnboundedSender<Event>>,
    pub(crate) pending_config: Option<MuonConfig>,
}

impl AppState {
    pub fn abort_pipeline(&mut self) {
        if let Some(handle) = self.pipeline_handle.take() {
            handle.abort();
        }
        self.clarifier_pending = None;
        if let Some(popup) = self.active_popup.take() {
            drop(popup);
        }
        self.clarifier_focused = false;
        self.pipeline.cancel();

        // D5: handle.abort() drops the future so run_pipeline's mark_session_terminal
        // never runs. Best-effort persist Cancelled without blocking the UI.
        if let Some(id) = self.export_session_id
            && let Some(infra) = self.infra.as_ref().map(Arc::clone)
        {
            tokio::spawn(async move {
                let _ = infra
                    .session_store
                    .update_stage(id, PipelineStage::Cancelled.as_str())
                    .await;
            });
        }
    }

    pub fn spawn_pipeline(&mut self, query: &str) {
        if self.pipeline.is_running() || self.pipeline_handle.is_some() {
            self.abort_pipeline();
        }
        self.live_feed_entries.clear();
        self.last_clarifier_log.clear();
        self.live_feed_scroll = 0;
        self.report_scroll = 0;
        self.source_scroll = 0;
        self.full_report_mode = false;
        self.pipeline.start();
        let Some(agent_tx) = self.agent_tx.clone() else {
            return;
        };
        let Some(infra) = self.infra.as_ref().map(Arc::clone) else {
            return;
        };
        let session_id = self
            .sessions
            .active()
            .map(|s| s.id)
            .or(self.export_session_id);
        if let Some(id) = session_id {
            self.export_session_id = Some(id);
        }
        let bridge = BridgeChannels::new(agent_tx);
        let cfg = self.config.clone();
        let mut pipeline = self.pipeline.clone_state_for_task();
        let query = query.to_string();
        let handle = tokio::spawn(async move {
            let deps = crate::application::deps::PipelineDeps::from_infra(&infra);
            match crate::application::pipeline_runner::run_pipeline(
                &query,
                &mut pipeline,
                &cfg,
                &deps,
                &bridge,
                session_id,
            )
            .await
            {
                Ok(report) => {
                    let sources = if let Ok(sink) = infra.source_sink.lock() {
                        sink.sources().to_vec()
                    } else {
                        Vec::new()
                    };
                    let sid = session_id.unwrap_or_else(uuid::Uuid::new_v4);
                    let _ = bridge.events.send(AgentEvent::Completed {
                        report,
                        sources,
                        session_id: sid,
                    });
                }
                Err(e) => {
                    let msg = if matches!(e, MuonError::Cancelled) {
                        "pipeline cancelled".to_string()
                    } else {
                        format!("pipeline failed: {e}")
                    };
                    let _ = bridge.events.send(AgentEvent::Error(msg));
                }
            }
        });
        self.pipeline_handle = Some(handle);
    }

    pub fn try_apply_config(&mut self, new_config: MuonConfig) -> bool {
        if self.pipeline.is_running() || self.pipeline_handle.is_some() {
            self.pending_config = Some(new_config);
            self.status_flash = Some((
                Instant::now(),
                "Config change deferred until research finishes".into(),
                crate::presentation::components::chrome::toast::ToastKind::Info,
            ));
            return false;
        }
        self.config = new_config;
        if let Some(palette) =
            crate::presentation::theme::for_name(&self.config.display.visual_theme)
        {
            crate::presentation::theme::replace(palette);
        }
        true
    }

    pub fn drain_pending_config(&mut self) {
        if let Some(cfg) = self.pending_config.take() {
            let _ = self.try_apply_config(cfg);
        }
    }

    pub fn is_pipeline_busy(&self) -> bool {
        self.pipeline.is_running() || self.pipeline_handle.is_some()
    }

    pub fn request_infra_rebuild(&self) {
        let Some(tx) = self.event_tx.clone() else {
            return;
        };
        let Some(agent_tx) = self.agent_tx.clone() else {
            return;
        };
        let cfg = self.config.clone();
        tokio::spawn(async move {
            let bridge = BridgeChannels::new(agent_tx);
            let result = InfrastructureContext::new_live(&cfg, &bridge)
                .await
                .map(Arc::new)
                .map_err(|e| e.to_string());
            let _ = tx.send(Event::InfraRebuilt(result));
        });
    }

    pub fn delete_active_session(&mut self) {
        use crate::presentation::components::chrome::toast::ToastKind;
        let Some(index) = self.sessions.list().iter().position(|s| s.is_active) else {
            self.set_status_flash("No active session to delete", ToastKind::Error);
            return;
        };
        self.delete_session_at(index);
    }

    pub fn delete_session_at(&mut self, index: usize) {
        use crate::presentation::components::chrome::toast::ToastKind;
        // Resolve summary BEFORE remove (needed for restore-on-failure + id).
        let Some(summary) = self.sessions.get(index).cloned() else {
            self.set_status_flash("No session to delete", ToastKind::Error);
            return;
        };
        let id = summary.id;
        // Only abort the running pipeline if we're deleting the session it's running.
        if self.export_session_id == Some(id) && self.is_pipeline_busy() {
            self.abort_pipeline();
        }
        let Some(removed_id) = self.sessions.remove(index) else {
            self.set_status_flash("No session to delete", ToastKind::Error);
            return;
        };
        if self.export_session_id == Some(removed_id) {
            self.export_session_id = None;
            self.last_report = None;
            self.last_sources.clear();
        }
        let max = self.sessions.list().len().saturating_sub(1);
        if self.session_scroll > max {
            self.session_scroll = max;
        }
        self.set_status_flash("Session deleted", ToastKind::Info);
        if let Some(infra) = self.infra.as_ref().map(Arc::clone) {
            let event_tx = self.event_tx.clone();
            tokio::spawn(async move {
                match infra.session_store.delete(id).await {
                    Ok(()) => { /* Optimistic UI already removed; nothing to do. */ }
                    Err(e) => {
                        let err_msg = e.to_string();
                        if let Some(tx) = event_tx {
                            let _ = tx.send(Event::SessionDeleteResult {
                                id,
                                ok: false,
                                error: Some(err_msg),
                                restored: Some(summary),
                            });
                        } else {
                            tracing::error!("session delete failed (no event channel): {err_msg}");
                        }
                    }
                }
            });
        }
    }

    pub fn restore_session(&mut self, index: usize) {
        let Some(summary) = self.sessions.get(index).cloned() else {
            return;
        };
        self.sessions.select(index);
        self.export_session_id = Some(summary.id);
        let Some(infra) = self.infra.as_ref().map(Arc::clone) else {
            return;
        };
        let Some(agent_tx) = self.agent_tx.clone() else {
            return;
        };
        let session_id = summary.id;
        tokio::spawn(async move {
            let report = infra
                .session_store
                .get_report(session_id)
                .await
                .ok()
                .flatten();
            let sources = infra
                .session_store
                .get_sources(session_id)
                .await
                .unwrap_or_default();
            let _ = agent_tx.send(AgentEvent::SessionRestored {
                report,
                sources,
                session_id,
            });
        });
    }

    fn set_status_flash(
        &mut self,
        msg: impl Into<String>,
        kind: crate::presentation::components::chrome::toast::ToastKind,
    ) {
        self.status_flash = Some((Instant::now(), msg.into(), kind));
    }

    pub(crate) fn push_live_feed(&mut self, mut entry: LogEntry) {
        if !matches!(entry.level, LogLevel::Error | LogLevel::Warn) {
            entry.message = truncate_feed_msg(entry.message);
        }
        self.live_feed_entries.push(entry);
        let excess = self.live_feed_entries.len().saturating_sub(LIVE_FEED_MAX);
        if excess > 0 {
            self.live_feed_entries.drain(0..excess);
            self.live_feed_scroll = self
                .live_feed_scroll
                .min(self.live_feed_entries.len().saturating_sub(1));
        }
    }

    fn push_sys_log(&mut self, message: impl Into<String>, level: LogLevel) {
        self.push_live_feed(LogEntry {
            timestamp: chrono::Utc::now(),
            agent_tag: AgentTag::Sys,
            message: message.into(),
            level,
        });
    }

    fn session_stub_for_export(&mut self) -> Session {
        let now = chrono::Utc::now();
        let (id, query, created_at) = if let Some(active) = self.sessions.active() {
            self.export_session_id = Some(active.id);
            (active.id, active.query.clone(), active.created_at)
        } else if let Some(id) = self.export_session_id {
            (
                id,
                self.query_input.buffer.clone(),
                self.last_report.as_ref().map(|_| now).unwrap_or(now),
            )
        } else {
            let id = uuid::Uuid::new_v4();
            self.export_session_id = Some(id);
            (id, self.query_input.buffer.clone(), now)
        };
        Session {
            id,
            query,
            status: SessionStatus::Complete,
            pipeline_stage: PipelineStage::Complete,
            intent: None,
            plan: None,
            clarifier_result: None,
            sources: self.last_sources.clone(),
            report: self.last_report.clone(),
            logs: Vec::new(),
            stats: self
                .last_report
                .as_ref()
                .map(|r| r.stats.clone())
                .unwrap_or_default(),
            created_at,
            updated_at: now,
        }
    }

    pub fn action_export_pdf(&mut self) {
        use crate::presentation::components::chrome::toast::ToastKind;
        let Some(report) = self.last_report.clone() else {
            self.set_status_flash("No report to export", ToastKind::Error);
            self.push_sys_log("Export PDF failed: no report", LogLevel::Error);
            return;
        };
        let session = self.session_stub_for_export();
        match ExportService::export(ExportRequest {
            report: &report,
            session: &session,
            format: ExportFormat::Pdf,
            obsidian_vault: None,
            markdown_dir: None,
        }) {
            Ok(path) => {
                let msg = format!("PDF Exported to {}", path.display());
                self.set_status_flash(msg.clone(), ToastKind::Success);
                self.push_sys_log(msg, LogLevel::Info);
            }
            Err(e) => {
                let msg = format!("PDF export failed: {e}");
                self.set_status_flash(msg.clone(), ToastKind::Error);
                self.push_sys_log(msg, LogLevel::Error);
            }
        }
    }

    pub fn action_export_markdown(&mut self) {
        use crate::presentation::components::chrome::toast::ToastKind;
        let Some(report) = self.last_report.clone() else {
            self.set_status_flash("No report to export", ToastKind::Error);
            self.push_sys_log("Export MD failed: no report", LogLevel::Error);
            return;
        };
        let session = self.session_stub_for_export();
        match ExportService::export(ExportRequest {
            report: &report,
            session: &session,
            format: ExportFormat::Markdown,
            obsidian_vault: None,
            markdown_dir: None,
        }) {
            Ok(path) => {
                let msg = format!("Markdown Exported to {}", path.display());
                self.set_status_flash(msg.clone(), ToastKind::Success);
                self.push_sys_log(msg, LogLevel::Info);
            }
            Err(e) => {
                let msg = format!("Markdown export failed: {e}");
                self.set_status_flash(msg.clone(), ToastKind::Error);
                self.push_sys_log(msg, LogLevel::Error);
            }
        }
    }

    pub fn action_sync_obsidian(&mut self) {
        use crate::presentation::components::chrome::toast::ToastKind;
        let Some(report) = self.last_report.clone() else {
            self.set_status_flash("No report to sync", ToastKind::Error);
            self.push_sys_log("Sync Obsidian failed: no report", LogLevel::Error);
            return;
        };
        let vault = if !self.config.obsidian.vault_path.is_empty() {
            crate::infrastructure::util::expand_tilde(&self.config.obsidian.vault_path)
        } else {
            match std::env::var("MUON_OBSIDIAN_VAULT") {
                Ok(v) if !v.trim().is_empty() => std::path::PathBuf::from(v),
                _ => {
                    let msg = "Obsidian vault not configured: set [obsidian] vault_path in config.toml or MUON_OBSIDIAN_VAULT env";
                    self.set_status_flash(msg, ToastKind::Error);
                    self.push_sys_log(msg, LogLevel::Error);
                    return;
                }
            }
        };
        let session = self.session_stub_for_export();
        match ExportService::export(ExportRequest {
            report: &report,
            session: &session,
            format: ExportFormat::Obsidian,
            obsidian_vault: Some(vault.as_path()),
            markdown_dir: None,
        }) {
            Ok(path) => {
                let msg = format!("Obsidian synced to {}", path.display());
                self.set_status_flash(msg.clone(), ToastKind::Success);
                self.push_sys_log(msg, LogLevel::Info);
            }
            Err(e) => {
                let msg = format!("Obsidian sync failed: {e}");
                self.set_status_flash(msg.clone(), ToastKind::Error);
                self.push_sys_log(msg, LogLevel::Error);
            }
        }
    }

    pub fn action_new_query(&mut self) {
        use crate::presentation::components::chrome::toast::ToastKind;
        self.pipeline = PipelineState::default();
        self.last_report = None;
        self.last_sources.clear();
        self.report_scroll = 0;
        self.source_scroll = 0;
        self.full_report_mode = false;
        self.live_feed_scroll = 0;
        self.clarifier_focused = false;
        self.query_input.buffer.clear();
        self.query_input.cursor = 0;
        self.query_input.active = true;
        self.export_session_id = None;
        self.router.transition(View::Dashboard);
        self.set_status_flash("New query", ToastKind::Info);
    }

    pub fn action_refine_search(&mut self) {
        use crate::presentation::components::chrome::toast::ToastKind;
        let query = self
            .sessions
            .active()
            .map(|s| s.query.clone())
            .or_else(|| self.last_report.as_ref().map(|r| r.title.clone()))
            .unwrap_or_else(|| self.query_input.buffer.clone());
        self.query_input.buffer = query;
        self.query_input.cursor = self.query_input.buffer.len();
        self.query_input.active = true;
        self.clarifier_focused = false;
        self.full_report_mode = false;
        self.router.transition(View::Dashboard);
        self.set_status_flash("Refine search — edit query and submit", ToastKind::Info);
    }

    pub fn action_toggle_full_report(&mut self) {
        use crate::presentation::components::chrome::toast::ToastKind;
        self.full_report_mode = !self.full_report_mode;
        self.report_scroll = 0;
        let mode = if self.full_report_mode {
            "Full report view"
        } else {
            "Summary view"
        };
        self.set_status_flash(mode, ToastKind::Info);
    }

    pub fn action_copy_source_url(&mut self, url: &str) {
        use crate::presentation::components::chrome::toast::ToastKind;
        match copy_text_to_clipboard(url, &mut self.clipboard) {
            Ok(()) => {
                let msg = format!("Copied to clipboard: {url}");
                self.set_status_flash(msg.clone(), ToastKind::Success);
                self.push_sys_log(msg, LogLevel::Info);
            }
            Err(e) => {
                let msg = format!("Clipboard failed: {e}");
                self.set_status_flash(msg.clone(), ToastKind::Error);
                self.push_sys_log(msg, LogLevel::Error);
            }
        }
    }
}
