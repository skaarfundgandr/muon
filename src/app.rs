use std::sync::Arc;
use std::time::Duration;

use crossterm::event::{poll, read, EnableMouseCapture, Event as CrosstermEvent, MouseButton, MouseEvent, MouseEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tokio::sync::mpsc;

use crate::application::bridge::{AgentEvent, BridgeChannels, PlanDecision};
use crate::application::pipeline::PipelineState;
use crate::config::MuonConfig;
use crate::domain::models::log_entry::{AgentTag, LogEntry, LogLevel};
use crate::infrastructure::context::InfrastructureContext;
use crate::presentation::click::{ClickAction, ClickTarget};
use crate::presentation::components::query_input::QueryInput;
use crate::presentation::form::FormState;
use crate::presentation::views::{RenderParams, SettingsTab, View, ViewRouter};
use crate::session::SessionService;

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
pub struct AppState {
    pub router: ViewRouter,
    pub running: bool,
    tick_count: u64,
    pub config: MuonConfig,
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
    pub live_feed_entries: Vec<crate::domain::models::log_entry::LogEntry>,
    pub last_clarifier_log: String,
    pub clarifier_focused: bool,
    pub live_feed_scroll: usize,
}

impl AppState {
    pub fn spawn_pipeline(&mut self, query: &str) {
        self.live_feed_entries.clear();
        self.last_clarifier_log.clear();
        self.live_feed_scroll = 0;
        let Some(agent_tx) = self.agent_tx.clone() else {
            return;
        };
        let Some(infra) = self.infra.as_ref().map(Arc::clone) else {
            return;
        };
        let bridge = BridgeChannels::new(agent_tx);
        let cfg = self.config.clone();
        let mut pipeline = self.pipeline.clone_state_for_task();
        let query = query.to_string();
        tokio::spawn(async move {
            match crate::application::pipeline_runner::run_pipeline(
                &query,
                &mut pipeline,
                &cfg,
                &infra,
                &bridge,
            )
            .await
            {
                Ok(report) => {
                    let sources = if let Ok(sink) = infra.source_sink.lock() {
                        sink.sources().to_vec()
                    } else {
                        Vec::new()
                    };
                    let _ = bridge.events.send(AgentEvent::Completed { report, sources });
                }
                Err(e) => {
                    let _ = bridge.events.send(AgentEvent::Error(format!("pipeline failed: {e}")));
                }
            }
        });
    }
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
    ConfigReloaded(Box<MuonConfig>),
}

fn setup_terminal() -> crate::error::Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) {
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
}

fn render(frame: &mut ratatui::Frame, app: &mut AppState) {
    app.term_cols = frame.area().width;
    app.term_rows = frame.area().height;
    app.hit_registry.clear();
    let view = app.router.active();
    let clarifier_question = app.clarifier_pending.as_ref().map(|c| c.question.as_str());
    let clarifier_response = app.clarifier_response.as_str();
    let clarifier_log = if app.last_clarifier_log.is_empty() {
        None
    } else {
        Some(app.last_clarifier_log.as_str())
    };
    view.render(frame, frame.area(), &mut RenderParams {
        query_input: &app.query_input,
        sessions: app.sessions.list(),
        pipeline: &app.pipeline,
        config: &app.config,
        forms: &app.forms,
        settings_tab: app.router.settings_tab(),
        hit_registry: &mut app.hit_registry,
        mouse_col: app.mouse_col,
        mouse_row: app.mouse_row,
        clarifier_question,
        clarifier_response,
        last_report: app.last_report.as_ref(),
        last_sources: app.last_sources.as_slice(),
        live_feed: app.live_feed_entries.as_slice(),
        live_feed_scroll: app.live_feed_scroll,
        clarifier_log,
        clarifier_focused: app.clarifier_focused,
    });

    if let Some(popup) = &app.active_popup {
        match popup {
            ActivePopup::EditModels { provider_idx, focus_idx, edit_buffer, edit_cursor, scroll_offset } => {
                crate::presentation::components::inputs::settings::providers::render_models_popup(
                    frame,
                    frame.area(),
                    &app.config,
                    *provider_idx,
                    *focus_idx,
                    *scroll_offset,
                    edit_buffer.as_deref(),
                    *edit_cursor,
                    &mut app.hit_registry,
                    app.mouse_col,
                    app.mouse_row,
                );
            }
            ActivePopup::ConfigureSearch { provider_idx, focus_idx, edit_buffer, edit_cursor } => {
                crate::presentation::components::inputs::settings::tools::render_configure_popup(
                    frame,
                    frame.area(),
                    &app.config,
                    *provider_idx,
                    *focus_idx,
                    edit_buffer.as_deref(),
                    *edit_cursor,
                    &mut app.hit_registry,
                    app.mouse_col,
                    app.mouse_row,
                );
            }
            ActivePopup::PlanApproval { plan, responder: _, focus, feedback_buffer, feedback_cursor } => {
                crate::presentation::components::panels::plan_approval::render(
                    frame,
                    frame.area(),
                    plan,
                    *focus,
                    feedback_buffer,
                    *feedback_cursor,
                    &mut app.hit_registry,
                    app.mouse_col,
                    app.mouse_row,
                );
            }
        }
    }
}

fn handle_mouse_click(app: &mut AppState, col: u16, row: u16) {
    for target in app.hit_registry.iter().rev() {
        if !crate::presentation::click::is_hovering(target.rect, col, row) {
            continue;
        }
        let mut close_popup_decision = None;
        if let Some(popup) = &mut app.active_popup {
            match popup {
                ActivePopup::EditModels { provider_idx, focus_idx, edit_buffer, edit_cursor, scroll_offset } => {
                    match &target.action {
                        ClickAction::ActivateField(idx) => {
                            *focus_idx = *idx;
                            *edit_buffer = None;
                            *edit_cursor = 0;
                            let model_idx = idx / 3;
                            let sub_idx = idx % 3;
                            if model_idx < app.config.providers[*provider_idx].models.len() {
                                let current = match sub_idx {
                                    0 => app.config.providers[*provider_idx].models[model_idx].name.clone(),
                                    1 => app.config.providers[*provider_idx].models[model_idx].model_id.clone(),
                                    _ => String::new(),
                                };
                                if sub_idx < 2 {
                                    *edit_buffer = Some(current.clone());
                                    *edit_cursor = current.len();
                                }
                            }
                            
                            // Adjust scroll_offset to keep focused model in view
                            let popup_h = 18u16.min(app.term_rows);
                            let inner_h = popup_h.saturating_sub(2);
                            let chunks_0_h = inner_h.saturating_sub(2);
                            let max_visible_models = ((chunks_0_h / 2) as usize).max(1);
                            if model_idx < *scroll_offset {
                                *scroll_offset = model_idx;
                            } else if model_idx >= *scroll_offset + max_visible_models {
                                *scroll_offset = model_idx + 1 - max_visible_models;
                            }
                            return;
                        }
                        ClickAction::RemoveModel(idx) => {
                            if *idx < app.config.providers[*provider_idx].models.len() {
                                app.config.providers[*provider_idx].models.remove(*idx);
                                *focus_idx = 0;
                                *scroll_offset = 0;
                                app.forms[SettingsTab::Providers as usize].dirty = true;
                            }
                            return;
                        }
                        ClickAction::AddModel => {
                            app.config.providers[*provider_idx].models.push(crate::config::ProviderModel {
                                name: String::new(),
                                model_id: String::new(),
                                description: String::new(),
                            });
                            let m = app.config.providers[*provider_idx].models.len();
                            *focus_idx = 3 * m - 3;
                            let popup_h = 18u16.min(app.term_rows);
                            let inner_h = popup_h.saturating_sub(2);
                            let chunks_0_h = inner_h.saturating_sub(2);
                            let max_visible_models = ((chunks_0_h / 2) as usize).max(1);
                            if m > max_visible_models {
                                *scroll_offset = m - max_visible_models;
                            } else {
                                *scroll_offset = 0;
                            }
                            app.forms[SettingsTab::Providers as usize].dirty = true;
                            return;
                        }
                        ClickAction::SwitchView(View::Settings) => {
                            app.active_popup = None;
                            return;
                        }
                        _ => {}
                    }
                }
                ActivePopup::ConfigureSearch { provider_idx, focus_idx, edit_buffer, edit_cursor } => {
                    match &target.action {
                        ClickAction::ActivateField(idx) => {
                            *focus_idx = *idx;
                            *edit_buffer = None;
                            *edit_cursor = 0;
                            if *idx < 3 {
                                let current = if *idx == 0 {
                                    app.config.search.providers[*provider_idx].name.clone()
                                } else if *idx == 1 {
                                    app.config.search.providers[*provider_idx].api_key.clone()
                                } else {
                                    app.config.search.providers[*provider_idx].max_results.map(|x| x.to_string()).unwrap_or_default()
                                };
                                *edit_buffer = Some(current.clone());
                                *edit_cursor = current.len();
                            }
                            return;
                        }
                        ClickAction::SwitchView(View::Settings) => {
                            app.active_popup = None;
                            return;
                        }
                        _ => {}
                    }
                }
                ActivePopup::PlanApproval { focus, .. } => {
                    match &target.action {
                        ClickAction::PlanApprove => {
                            close_popup_decision = Some(PlanDecision::Approve);
                        }
                        ClickAction::PlanReject => {
                            close_popup_decision = Some(PlanDecision::Reject);
                        }
                        ClickAction::PlanFeedback => {
                            if let ActivePopup::PlanApproval { feedback_buffer, .. } = popup {
                                close_popup_decision = Some(PlanDecision::Feedback(feedback_buffer.clone()));
                            }
                        }
                        ClickAction::PlanSelectFeedbackInput => {
                            *focus = PlanApprovalFocus::Feedback;
                            return;
                        }
                        _ => {}
                    }
                }
            }
            if close_popup_decision.is_none() {
                return;
            }
        }

        if let Some(decision) = close_popup_decision {
            if let Some(ActivePopup::PlanApproval { responder, .. }) = app.active_popup.take() {
                let _ = responder.send(decision);
            }
            return;
        }

        match &target.action {
            ClickAction::ActivateField(idx) => {
                let tab = app.router.settings_tab();
                let tab_idx = tab as usize;
                app.forms[tab_idx].focus = *idx;
                app.forms[tab_idx].reset_edit();
                let current = match tab {
                    SettingsTab::Providers => crate::presentation::components::inputs::settings::providers::get_field(&app.config, *idx),
                    SettingsTab::Agents => crate::presentation::components::inputs::settings::agents::get_field(&app.config.agents, *idx),
                    SettingsTab::Tools => crate::presentation::components::inputs::settings::tools::get_field(&app.config, *idx),
                    SettingsTab::DataSources => crate::presentation::components::inputs::settings::data_sources::get_field(&app.config, *idx),
                    SettingsTab::Display => crate::presentation::components::inputs::settings::display::get_field(&app.config.display, *idx),
                    SettingsTab::Advanced => crate::presentation::components::inputs::settings::advanced::get_field(&app.config.advanced, *idx),
                };
                let kind = match tab {
                    SettingsTab::Providers => crate::presentation::components::inputs::settings::providers::fields(&app.config)[*idx].kind,
                    SettingsTab::Agents => crate::presentation::components::inputs::settings::agents::fields()[*idx].kind,
                    SettingsTab::Tools => crate::presentation::components::inputs::settings::tools::fields(&app.config)[*idx].kind,
                    SettingsTab::DataSources => crate::presentation::components::inputs::settings::data_sources::fields(&app.config)[*idx].kind,
                    SettingsTab::Display => crate::presentation::components::inputs::settings::display::fields()[*idx].kind,
                    SettingsTab::Advanced => crate::presentation::components::inputs::settings::advanced::fields()[*idx].kind,
                };
                match kind {
                    crate::presentation::form::FieldKind::Text | crate::presentation::form::FieldKind::Number => {
                        app.forms[tab_idx].begin_edit(&current);
                    }
                    crate::presentation::form::FieldKind::Dropdown => {
                        if app.forms[tab_idx].dropdown_open {
                            app.forms[tab_idx].dropdown_open = false;
                        } else {
                            app.forms[tab_idx].open_dropdown();
                        }
                    }
                    crate::presentation::form::FieldKind::Checkbox => {
                        match tab {
                            SettingsTab::Providers => crate::presentation::components::inputs::settings::providers::toggle_field(&mut app.config, *idx),
                            SettingsTab::Agents => crate::presentation::components::inputs::settings::agents::toggle_field(&mut app.config.agents, *idx),
                            SettingsTab::Tools => crate::presentation::components::inputs::settings::tools::toggle_field(&mut app.config, *idx),
                            SettingsTab::DataSources => crate::presentation::components::inputs::settings::data_sources::toggle_field(&mut app.config, *idx),
                            SettingsTab::Display => crate::presentation::components::inputs::settings::display::toggle_field(&mut app.config.display, *idx),
                            SettingsTab::Advanced => crate::presentation::components::inputs::settings::advanced::toggle_field(&mut app.config.advanced, *idx),
                        }
                        app.forms[tab_idx].dirty = true;
                    }
                    crate::presentation::form::FieldKind::Button => {
                        match tab {
                            SettingsTab::DataSources => {
                                if *idx == 6 {
                                    let path = app.config.data_sources.source_path.clone();
                                    let kind = app.config.data_sources.source_type.to_uppercase();
                                    app.config.data_sources.rag_indexes.push(crate::config::RagIndexConfig {
                                        path,
                                        kind,
                                        status: "○ pending".to_string(),
                                        chunks: "0".to_string(),
                                    });
                                    app.forms[tab_idx].dirty = true;
                                }
                            }
                            SettingsTab::Providers => {
                                let n = app.config.providers.len();
                                if *idx == 5 * n {
                                    app.config.providers.push(crate::config::ProviderConfig {
                                        name: String::new(),
                                        base_url: String::new(),
                                        api_key: String::new(),
                                        models: Vec::new(),
                                        provider_type: crate::config::ProviderType::OpenAICompatible,
                                    });
                                    app.forms[tab_idx].focus = 5 * app.config.providers.len() - 5;
                                    app.forms[tab_idx].dirty = true;
                                }
                            }
                            SettingsTab::Tools => {
                                let n = app.config.search.providers.len();
                                if *idx == 5 * n + 1 {
                                    app.config.search.providers.push(crate::config::SearchProviderConfig {
                                        name: String::new(),
                                        provider_type: crate::config::SearchProviderType::Tavily,
                                        api_key: String::new(),
                                        max_results: None,
                                        tavily: Default::default(),
                                        firecrawl: Default::default(),
                                        brave: Default::default(),
                                        serper: Default::default(),
                                    });
                                    app.forms[tab_idx].focus = 5 * app.config.search.providers.len() - 5;
                                    app.forms[tab_idx].dirty = true;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            ClickAction::ToggleCheckbox(idx) => {
                let tab = app.router.settings_tab();
                let tab_idx = tab as usize;
                app.forms[tab_idx].focus = *idx;
                app.forms[tab_idx].reset_edit();
                match tab {
                    SettingsTab::Providers => crate::presentation::components::inputs::settings::providers::toggle_field(&mut app.config, *idx),
                    SettingsTab::Agents => crate::presentation::components::inputs::settings::agents::toggle_field(&mut app.config.agents, *idx),
                    SettingsTab::Tools => crate::presentation::components::inputs::settings::tools::toggle_field(&mut app.config, *idx),
                    SettingsTab::DataSources => crate::presentation::components::inputs::settings::data_sources::toggle_field(&mut app.config, *idx),
                    SettingsTab::Display => crate::presentation::components::inputs::settings::display::toggle_field(&mut app.config.display, *idx),
                    SettingsTab::Advanced => crate::presentation::components::inputs::settings::advanced::toggle_field(&mut app.config.advanced, *idx),
                }
                app.forms[tab_idx].dirty = true;
            }
            ClickAction::FocusField(idx) => {
                let tab = app.router.settings_tab();
                let tab_idx = tab as usize;
                app.forms[tab_idx].focus = *idx;
                app.forms[tab_idx].reset_edit();
            }
            ClickAction::SwitchSettingsTab(tab) => {
                app.router.set_settings_tab(*tab);
                let new_idx = *tab as usize;
                app.forms[new_idx].reset_edit();
                app.forms[new_idx].focus = 0;
            }
            ClickAction::SwitchView(v) => {
                app.router.transition(*v);
                if *v == View::Settings {
                    let tab_idx = app.router.settings_tab() as usize;
                    app.forms[tab_idx].focus = 0;
                    app.forms[tab_idx].reset_edit();
                }
            }
            ClickAction::ActivateQueryInput => {
                app.query_input.active = true;
                app.clarifier_focused = false;
            }
            ClickAction::SelectSession(idx) => {
                app.sessions.select(*idx);
            }
            ClickAction::SelectDropdownOption(idx) => {
                let tab = app.router.settings_tab();
                let tab_idx = tab as usize;
                let options: Vec<String> = match tab {
                    SettingsTab::Providers => crate::presentation::components::inputs::settings::providers::fields(&app.config)[app.forms[tab_idx].focus]
                        .options
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    SettingsTab::Agents => crate::presentation::components::inputs::settings::agents::options_for(
                        app.forms[tab_idx].focus,
                        &app.config,
                    ),
                    SettingsTab::Tools => crate::presentation::components::inputs::settings::tools::fields(&app.config)[app.forms[tab_idx].focus]
                        .options
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    SettingsTab::DataSources => crate::presentation::components::inputs::settings::data_sources::fields(&app.config)[app.forms[tab_idx].focus]
                        .options
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    SettingsTab::Display => crate::presentation::components::inputs::settings::display::fields()[app.forms[tab_idx].focus]
                        .options
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    SettingsTab::Advanced => crate::presentation::components::inputs::settings::advanced::fields()[app.forms[tab_idx].focus]
                        .options
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                };
                if *idx < options.len() {
                    let val = options[*idx].clone();
                    if val.starts_with("<no models") {
                        app.forms[tab_idx].dropdown_open = false;
                        return;
                    }
                    app.forms[tab_idx].dropdown_open = false;
                    match tab {
                        SettingsTab::Providers => {
                            crate::presentation::components::inputs::settings::providers::set_field(&mut app.config, app.forms[tab_idx].focus, &val);
                        }
                        SettingsTab::Agents => {
                            crate::presentation::components::inputs::settings::agents::set_field(&mut app.config.agents, app.forms[tab_idx].focus, &val);
                        }
                        SettingsTab::Tools => {
                            crate::presentation::components::inputs::settings::tools::set_field(&mut app.config, app.forms[tab_idx].focus, &val);
                        }
                        SettingsTab::DataSources => {
                            crate::presentation::components::inputs::settings::data_sources::set_field(&mut app.config, app.forms[tab_idx].focus, &val);
                        }
                        SettingsTab::Display => {
                            crate::presentation::components::inputs::settings::display::set_field(&mut app.config.display, app.forms[tab_idx].focus, &val);
                            if app.forms[tab_idx].focus == 0
                                && let Some(palette) = crate::presentation::theme::for_name(&val)
                            {
                                crate::presentation::theme::replace(palette);
                            }
                        }
                        SettingsTab::Advanced => {
                            crate::presentation::components::inputs::settings::advanced::set_field(&mut app.config.advanced, app.forms[tab_idx].focus, &val);
                        }
                    }
                    app.forms[tab_idx].dirty = true;
                }
            }
            ClickAction::ActivateClarifier => {
                app.query_input.active = false;
                app.clarifier_focused = true;
            }
            ClickAction::AddProvider => {
                use crate::config::{ProviderConfig, ProviderType};
                app.config.providers.push(ProviderConfig {
                    name: String::new(),
                    base_url: String::new(),
                    api_key: String::new(),
                    models: Vec::new(),
                    provider_type: ProviderType::OpenAICompatible,
                });
                app.forms[SettingsTab::Providers as usize].dirty = true;
            }
            ClickAction::RemoveProvider(idx) => {
                if *idx < app.config.providers.len() {
                    app.config.providers.swap_remove(*idx);
                    app.forms[SettingsTab::Providers as usize].dirty = true;
                }
            }
            ClickAction::AddSearchProvider => {
                use crate::config::{SearchProviderConfig, SearchProviderType};
                app.config.search.providers.push(SearchProviderConfig {
                    name: String::new(),
                    provider_type: SearchProviderType::Tavily,
                    api_key: String::new(),
                    max_results: None,
                    tavily: Default::default(),
                    firecrawl: Default::default(),
                    brave: Default::default(),
                    serper: Default::default(),
                });
                app.forms[SettingsTab::Tools as usize].dirty = true;
            }
            ClickAction::RemoveSearchProvider(idx) => {
                if *idx < app.config.search.providers.len() {
                    app.config.search.providers.swap_remove(*idx);
                    app.forms[SettingsTab::Tools as usize].dirty = true;
                }
            }
            ClickAction::ToggleSearchProvider(_idx) => {
                // No per-provider enabled flag yet — UI hint only.
            }
            ClickAction::ToggleArxiv => {
                app.config.search.papers.arxiv_enabled = !app.config.search.papers.arxiv_enabled;
                app.forms[SettingsTab::Tools as usize].dirty = true;
            }
            ClickAction::EditProviderModels(idx) => {
                app.active_popup = Some(ActivePopup::EditModels {
                    provider_idx: *idx,
                    focus_idx: 0,
                    edit_buffer: None,
                    edit_cursor: 0,
                    scroll_offset: 0,
                });
            }
            ClickAction::FetchProviderModels(idx) => {
                let idx = *idx;
                if idx < app.config.providers.len() {
                    let provider = &mut app.config.providers[idx];
                    let api_key = match provider.resolved_api_key() {
                        Ok(k) => k,
                        Err(e) => {
                            tracing::error!("Failed to resolve API key: {:?}", e);
                            provider.models = vec![crate::config::ProviderModel {
                                name: format!("Error: resolved_api_key: {e}"),
                                model_id: "error".to_string(),
                                description: String::new(),
                            }];
                            app.forms[SettingsTab::Providers as usize].dirty = true;
                            return;
                        }
                    };

                    let mut base_url = provider.base_url.trim().to_string();
                    if base_url.ends_with('/') {
                        base_url.pop();
                    }
                    let url = format!("{base_url}/models");

                    let (tx, rx) = std::sync::mpsc::channel();
                    std::thread::spawn(move || {
                        let client = reqwest::blocking::Client::new();
                        let mut req = client.get(&url);
                        if !api_key.is_empty() {
                            req = req.header("Authorization", format!("Bearer {api_key}"));
                        }
                        let res = req.send().map_err(|e| e.to_string()).and_then(|resp| {
                            #[derive(serde::Deserialize)]
                            struct ModelData {
                                id: String,
                            }
                            #[derive(serde::Deserialize)]
                            struct ModelsResponse {
                                data: Vec<ModelData>,
                            }
                            resp.json::<ModelsResponse>()
                                .map_err(|e| e.to_string())
                                .map(|parsed| parsed.data.into_iter().map(|m| m.id).collect::<Vec<String>>())
                        });
                        let _ = tx.send(res);
                    });

                    match rx.recv() {
                        Ok(Ok(model_ids)) => {
                            provider.models = model_ids
                                .into_iter()
                                .map(|id| crate::config::ProviderModel {
                                    name: id.clone(),
                                    model_id: id,
                                    description: String::new(),
                                })
                                .collect();
                        }
                        Ok(Err(e)) => {
                            tracing::error!("Failed to fetch/parse models: {}", e);
                            provider.models = vec![crate::config::ProviderModel {
                                name: format!("Error: {e}"),
                                model_id: "error".to_string(),
                                description: String::new(),
                            }];
                        }
                        Err(_) => {
                            provider.models = vec![crate::config::ProviderModel {
                                name: "Error: thread channel disconnected".to_string(),
                                model_id: "error".to_string(),
                                description: String::new(),
                            }];
                        }
                    }
                    app.forms[SettingsTab::Providers as usize].dirty = true;
                }
            }
            ClickAction::ConfigureSearchOptions(idx) => {
                app.active_popup = Some(ActivePopup::ConfigureSearch {
                    provider_idx: *idx,
                    focus_idx: 0,
                    edit_buffer: None,
                    edit_cursor: 0,
                });
            }
            ClickAction::ReindexRagIndex(idx) if *idx < app.config.data_sources.rag_indexes.len() => {
                app.config.data_sources.rag_indexes[*idx].status = "◉ indexing".to_string();
                app.forms[SettingsTab::DataSources as usize].dirty = true;
            }
            ClickAction::RemoveRagIndex(idx) if *idx < app.config.data_sources.rag_indexes.len() => {
                app.config.data_sources.rag_indexes.remove(*idx);
                app.forms[SettingsTab::DataSources as usize].dirty = true;
            }
            _ => {}
        }
        return;
    }
}

fn handle_scroll(app: &mut AppState, delta: i32) {
    // 1. If EditModels popup is active, scroll the popup list
    if let Some(ActivePopup::EditModels { provider_idx, scroll_offset, .. }) = &mut app.active_popup {
        let m = app.config.providers[*provider_idx].models.len();
        let popup_h = 18u16.min(app.term_rows);
        let inner_h = popup_h.saturating_sub(2);
        let chunks_0_h = inner_h.saturating_sub(2);
        let max_visible_models = ((chunks_0_h / 2) as usize).max(1);
        let max_offset = m.saturating_sub(max_visible_models);
        if delta > 0 {
            *scroll_offset = (*scroll_offset + 1).min(max_offset);
        } else {
            *scroll_offset = scroll_offset.saturating_sub(1);
        }
        return;
    }

    // 2. If a dropdown is open on the active settings tab, scroll the dropdown cursor
    let tab = app.router.settings_tab();
    let tab_idx = tab as usize;
    if app.forms[tab_idx].dropdown_open {
        let options = match tab {
            SettingsTab::Providers => crate::presentation::components::inputs::settings::providers::fields(&app.config)[app.forms[tab_idx].focus]
                .options
                .iter()
                .map(|s| s.to_string())
                .collect(),
            SettingsTab::Agents => crate::presentation::components::inputs::settings::agents::options_for(
                app.forms[tab_idx].focus,
                &app.config,
            ),
            SettingsTab::Tools => crate::presentation::components::inputs::settings::tools::fields(&app.config)[app.forms[tab_idx].focus]
                .options
                .iter()
                .map(|s| s.to_string())
                .collect(),
            SettingsTab::DataSources => crate::presentation::components::inputs::settings::data_sources::fields(&app.config)[app.forms[tab_idx].focus]
                .options
                .iter()
                .map(|s| s.to_string())
                .collect(),
            SettingsTab::Display => crate::presentation::components::inputs::settings::display::fields()[app.forms[tab_idx].focus]
                .options
                .iter()
                .map(|s| s.to_string())
                .collect(),
            SettingsTab::Advanced => crate::presentation::components::inputs::settings::advanced::fields()[app.forms[tab_idx].focus]
                .options
                .iter()
                .map(|s| s.to_string())
                .collect(),
        };
        let max = options.len();
        if max > 0 {
            if delta > 0 {
                if app.forms[tab_idx].dropdown_cursor + 1 < max {
                    app.forms[tab_idx].dropdown_cursor += 1;
                }
            } else {
                app.forms[tab_idx].dropdown_cursor = app.forms[tab_idx].dropdown_cursor.saturating_sub(1);
            }
        }
        return;
    }

    // 3. Otherwise scroll the background tab list
    use crate::presentation::handlers::settings::{scroll_list_len, scroll_visible_rows};
    let Some(visible) = scroll_visible_rows(app, tab) else {
        return;
    };
    let list_len = scroll_list_len(app, tab);
    if list_len == 0 {
        return;
    }
    let max_offset = list_len.saturating_sub(visible);
    let form = &mut app.forms[tab_idx];
    if delta > 0 {
        form.scroll_offset = (form.scroll_offset + 1).min(max_offset);
    } else {
        form.scroll_offset = form.scroll_offset.saturating_sub(1);
    }
}

fn handle_event(app: &mut AppState, event: Event) {
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
            #[allow(clippy::collapsible_if)]
            if let Some(ref mut rx) = app.config_reload_rx {
                if let Ok(new_config) = rx.try_recv() {
                    tracing::info!(target: "muon::config", "config reloaded from disk");
                    app.config = new_config;
                }
            }
        }
        Event::AgentEvent(AgentEvent::StageChanged(stage)) => {
            app.pipeline.set_stage(stage);
        }
        Event::AgentEvent(AgentEvent::Completed { report, sources }) => {
            app.last_report = Some(report);
            app.last_sources = sources;
            app.pipeline.finish();
            app.clarifier_focused = false;
        }
        Event::AgentEvent(AgentEvent::Log(entry)) => {
            if let Some(active) = app.sessions.active() {
                let _ = active;
            }
            let tag = entry.agent_tag;
            if matches!(tag, AgentTag::Sys) {
                tracing::debug!(target: "muon::sys", "{}", entry.message);
            }
            app.live_feed_entries.push(entry);
            if app.live_feed_entries.len() > 50 {
                app.live_feed_entries.remove(0);
            }
        }
        Event::AgentEvent(AgentEvent::ClarifierQuestion {
            question,
            responder,
        }) => {
            app.query_input.active = false;
            app.clarifier_pending = Some(ClarifierPending { question, responder });
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
            app.clarifier_focused = false;
            let entry = LogEntry {
                timestamp: chrono::Utc::now(),
                agent_tag: AgentTag::Sys,
                message: msg,
                level: LogLevel::Error,
            };
            app.live_feed_entries.push(entry);
            if app.live_feed_entries.len() > 50 {
                app.live_feed_entries.remove(0);
            }
        }
        Event::ConfigReloaded(new_config) => {
            tracing::info!(target: "muon::config", "config reloaded via event");
            app.config = *new_config;
        }
    }
}

async fn build_infrastructure(
    cfg: &MuonConfig,
    bridge: &BridgeChannels,
) -> Result<InfrastructureContext, crate::error::MuonError> {
    InfrastructureContext::new_live(cfg, bridge)
        .await
        .map_err(|e| {
            bridge.log(
                AgentTag::Sys,
                LogLevel::Error,
                format!("live backend failed: {e}"),
            );
            e
        })
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> crate::error::Result<()> {
    let config = MuonConfig::load();

    let mut config_reload_rx = {
        use futures::StreamExt;
        let mut stream = MuonConfig::watch();
        let (tx, rx) = mpsc::channel(4);
        tokio::spawn(async move {
            while let Some(cfg) = stream.next().await {
                if tx.send(cfg).await.is_err() {
                    break;
                }
            }
        });
        rx
    };
    let _ = config_reload_rx.try_recv();

    let mut app = AppState {
        router: ViewRouter::new(),
        running: true,
        tick_count: 0,
        config: config.clone(),
        forms: std::array::from_fn(|_| FormState::default()),
        query_input: QueryInput::default(),
        sessions: SessionService::new(),
        pipeline: PipelineState::default(),
        mouse_col: 0,
        mouse_row: 0,
        term_cols: 0,
        term_rows: 0,
        hit_registry: Vec::new(),
        clarifier_pending: None,
        clarifier_response: String::new(),
        plan_pending: None,
        agent_tx: None,
        infra: None,
        config_reload_rx: Some(config_reload_rx),
        active_popup: None,
        last_report: None,
        last_sources: Vec::new(),
        live_feed_entries: Vec::new(),
        live_feed_scroll: 0,
        last_clarifier_log: String::new(),
        clarifier_focused: false,
    };

    if let Some(palette) = crate::presentation::theme::for_name(&app.config.display.visual_theme) {
        crate::presentation::theme::replace(palette);
    }

    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<Event>();
    let (agent_tx, mut agent_rx) = mpsc::unbounded_channel::<AgentEvent>();
    let key_tx = event_tx.clone();

    tokio::spawn(async move {
        loop {
            match poll(Duration::from_millis(250)) {
                Ok(true) => match read() {
                    Ok(CrosstermEvent::Key(key)) => {
                        if key_tx.send(Event::Key(key)).is_err() {
                            break;
                        }
                    }
                    Ok(CrosstermEvent::Mouse(mouse)) => {
                        if key_tx.send(Event::Mouse(mouse)).is_err() {
                            break;
                        }
                    }
                    Ok(_) => {}
                    Err(_) => break,
                },
                Ok(false) => {
                    let _ = key_tx.send(Event::Tick);
                }
                Err(_) => break,
            }
        }
    });

    // Forward agent events into the main event loop.
    tokio::spawn(async move {
        while let Some(ev) = agent_rx.recv().await {
            if event_tx.send(Event::AgentEvent(ev)).is_err() {
                break;
            }
        }
    });

    let bridge_for_init = BridgeChannels::new(agent_tx.clone());
    let infra = build_infrastructure(&config, &bridge_for_init).await?;
    app.infra = Some(Arc::new(infra));
    app.agent_tx = Some(agent_tx);

    while app.running {
        terminal.draw(|f| render(f, &mut app))?;
        if let Some(event) = event_rx.recv().await {
            handle_event(&mut app, event);
        }
    }

    Ok(())
}

pub async fn run() -> crate::error::Result<()> {
    let mut terminal = setup_terminal()?;
    let result = run_loop(&mut terminal).await;
    restore_terminal(&mut terminal);
    result
}
