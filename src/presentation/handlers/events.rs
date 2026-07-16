use std::sync::Arc;
use std::time::{Duration, Instant};

use crossterm::event::{MouseButton, MouseEventKind};

use crate::application::bridge::AgentEvent;
use crate::domain::models::log_entry::{AgentTag, LogEntry, LogLevel};
use crate::presentation::views::{SettingsTab, View};

use super::mouse::handle_mouse_click;
use super::scroll::handle_scroll;
use crate::presentation::state::AppState;
use crate::presentation::types::{ActivePopup, ClarifierPending, Event, PlanApprovalFocus};

pub(crate) fn handle_event(app: &mut AppState, event: Event) {
    match event {
        Event::Key(key) => {
            crate::presentation::handlers::handle_key(app, key);
        }
        Event::Mouse(mouse) => {
            app.mouse_col = mouse.column;
            app.mouse_row = mouse.row;
            for form in &mut app.forms {
                form.mouse_col = mouse.column;
                form.mouse_row = mouse.row;
            }
            app.query_input.mouse_col = mouse.column;
            app.query_input.mouse_row = mouse.row;
            if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
                handle_mouse_click(app, mouse.column, mouse.row);
            }
            // Mouse wheel scroll for Providers and Tools tabs
            if let MouseEventKind::ScrollDown = mouse.kind {
                handle_scroll(app, 1);
            }
            if let MouseEventKind::ScrollUp = mouse.kind {
                handle_scroll(app, -1);
            }
        }
        Event::Tick => {
            app.tick_count += 1;
            if let Some((started, _, _)) = app.status_flash
                && started.elapsed() > Duration::from_secs(4)
            {
                app.status_flash = None;
            }
            #[allow(clippy::collapsible_if)]
            if let Some(ref mut rx) = app.config_reload_rx {
                if let Ok(new_config) = rx.try_recv() {
                    tracing::info!(target: "muon::config", "config reloaded from disk");
                    let _ = app.try_apply_config(new_config);
                }
            }
        }
        Event::AgentEvent(AgentEvent::StageChanged(stage)) => {
            app.pipeline.set_stage(stage);
        }
        Event::AgentEvent(AgentEvent::Completed {
            report,
            sources,
            session_id,
        }) => {
            app.pipeline_handle = None;
            app.last_report = Some(report);
            app.last_sources = sources;
            app.report_scroll = 0;
            app.source_scroll = 0;
            app.pipeline.finish();
            app.clarifier_focused = false;
            app.export_session_id = Some(session_id);
            if let Some(active) = app.sessions.active()
                && active.id != session_id
            {
                app.sessions
                    .insert_front(crate::application::session::SessionSummary {
                        id: session_id,
                        title: app
                            .last_report
                            .as_ref()
                            .map(|r| r.title.clone())
                            .unwrap_or_else(|| "Session".into()),
                        query: active.query.clone(),
                        created_at: active.created_at,
                        is_active: true,
                    });
            }
            app.drain_pending_config();
        }
        Event::AgentEvent(AgentEvent::SessionRestored {
            report,
            sources,
            session_id,
        }) => {
            app.export_session_id = Some(session_id);
            app.last_report = report;
            app.last_sources = sources;
            app.report_scroll = 0;
            app.source_scroll = 0;
            app.live_feed_scroll = 0;
            if app.last_report.is_some() {
                app.pipeline.finish();
                app.router.transition(View::Results);
            } else {
                app.query_input.buffer = app
                    .sessions
                    .active()
                    .map(|s| s.query.clone())
                    .unwrap_or_default();
                app.query_input.cursor = app.query_input.buffer.len();
                app.query_input.active = true;
                app.router.transition(View::Dashboard);
            }
        }
        Event::AgentEvent(AgentEvent::Log(entry)) => {
            let tag = entry.agent_tag;
            if matches!(tag, AgentTag::Sys) {
                tracing::debug!(target: "muon::sys", "{}", entry.message);
            }
            if let Some(sid) = app
                .export_session_id
                .or_else(|| app.sessions.active().map(|s| s.id))
                && let Some(infra) = app.infra.as_ref().map(Arc::clone)
            {
                let log = entry.clone();
                tokio::spawn(async move {
                    let _ = infra.session_store.append_log(sid, &log).await;
                });
            }
            app.push_live_feed(entry);
        }
        Event::AgentEvent(AgentEvent::ClarifierQuestion {
            question,
            responder,
        }) => {
            app.query_input.active = false;
            app.clarifier_pending = Some(ClarifierPending {
                question,
                responder,
            });
            app.clarifier_focused = true;
        }
        Event::AgentEvent(AgentEvent::ClarificationComplete { log }) => {
            app.last_clarifier_log = log;
            app.clarifier_focused = false;
        }
        Event::AgentEvent(AgentEvent::PlanProposed { plan, responder }) => {
            app.active_popup = Some(ActivePopup::PlanApproval {
                plan,
                responder,
                focus: PlanApprovalFocus::Approve,
                feedback_buffer: String::new(),
                feedback_cursor: 0,
            });
        }
        Event::AgentEvent(AgentEvent::Final(text)) => {
            tracing::info!(target: "muon::final", "{}", text);
        }
        Event::AgentEvent(AgentEvent::Error(msg)) => {
            tracing::error!(target: "muon::agent", "{}", msg);
            app.pipeline_handle = None;
            app.clarifier_focused = false;
            app.clarifier_pending = None;
            let is_cancel = msg.contains("cancelled") || msg.contains("Canceled");
            app.push_live_feed(LogEntry {
                timestamp: chrono::Utc::now(),
                agent_tag: AgentTag::Sys,
                message: msg.clone(),
                level: if is_cancel {
                    LogLevel::Warn
                } else {
                    LogLevel::Error
                },
            });
            if is_cancel {
                app.pipeline.cancel();
            } else {
                app.pipeline.fail();
            }
            app.status_flash = Some((
                Instant::now(),
                msg,
                if is_cancel {
                    crate::presentation::components::chrome::toast::ToastKind::Info
                } else {
                    crate::presentation::components::chrome::toast::ToastKind::Error
                },
            ));
            app.drain_pending_config();
        }
        Event::SessionDeleteResult {
            id,
            ok,
            error,
            restored,
        } => {
            if !ok {
                if let Some(summary) = restored {
                    app.sessions.insert_front(summary);
                }
                let msg = error.unwrap_or_else(|| "delete failed".to_string());
                app.status_flash = Some((
                    Instant::now(),
                    format!("Delete failed: {msg}"),
                    crate::presentation::components::chrome::toast::ToastKind::Error,
                ));
                app.push_live_feed(LogEntry {
                    timestamp: chrono::Utc::now(),
                    agent_tag: AgentTag::Sys,
                    message: format!("Session {id} delete failed: {msg}"),
                    level: LogLevel::Error,
                });
            }
            // On Ok: optimistic UI already removed; nothing to do.
        }
        Event::ModelsFetched {
            provider_index,
            result,
        } => match result {
            Ok(models) => {
                if let Some(p) = app.config.providers.get_mut(provider_index) {
                    p.models = models;
                    app.forms[SettingsTab::Providers as usize].dirty = true;
                    app.status_flash = Some((
                        Instant::now(),
                        "Models updated".into(),
                        crate::presentation::components::chrome::toast::ToastKind::Success,
                    ));
                }
            }
            Err(e) => {
                app.status_flash = Some((
                    Instant::now(),
                    format!("Fetch models failed: {e}"),
                    crate::presentation::components::chrome::toast::ToastKind::Error,
                ));
            }
        },
        Event::InfraRebuilt(Ok(infra)) => {
            app.infra = Some(infra);
            app.status_flash = Some((
                Instant::now(),
                "Agents reloaded from config".into(),
                crate::presentation::components::chrome::toast::ToastKind::Success,
            ));
        }
        Event::InfraRebuilt(Err(e)) => {
            app.status_flash = Some((
                Instant::now(),
                format!("Infra rebuild failed: {e}"),
                crate::presentation::components::chrome::toast::ToastKind::Error,
            ));
        }
        Event::RagIndexed { idx, summary } => {
            if let Some(entry) = app.config.data_sources.rag_indexes.get_mut(idx) {
                if summary.errors.is_empty() {
                    entry.status = "● indexed".to_string();
                    app.status_flash = Some((
                        Instant::now(),
                        format!("Indexed {} files ({} chunks)", summary.total_files, summary.total_chunks),
                        crate::presentation::components::chrome::toast::ToastKind::Success,
                    ));
                } else {
                    entry.status = "⚠ error".to_string();
                    app.status_flash = Some((
                        Instant::now(),
                        format!("Index completed with {} errors", summary.errors.len()),
                        crate::presentation::components::chrome::toast::ToastKind::Error,
                    ));
                }
                entry.chunks = summary.total_chunks as u64;
                app.forms[crate::presentation::views::SettingsTab::DataSources as usize].dirty = true;
            }
        }
    }
}
