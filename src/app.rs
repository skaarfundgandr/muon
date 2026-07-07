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
use crate::domain::models::log_entry::{AgentTag, LogLevel};
use crate::infrastructure::context::InfrastructureContext;
use crate::presentation::click::{ClickAction, ClickTarget};
use crate::presentation::components::query_input::QueryInput;
use crate::presentation::form::FormState;
use crate::presentation::views::{RenderParams, SettingsTab, View, ViewRouter};
use crate::session::SessionService;

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
}

impl AppState {
    pub fn spawn_pipeline(&mut self, query: &str) {
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
            let _ = crate::application::pipeline_runner::run_pipeline(
                &query,
                &mut pipeline,
                &cfg,
                &infra,
                &bridge,
            )
            .await;
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
    });
}

fn handle_mouse_click(app: &mut AppState, col: u16, row: u16) {
    for target in app.hit_registry.iter().rev() {
        if !crate::presentation::click::is_hovering(target.rect, col, row) {
            continue;
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
                    SettingsTab::Tools => crate::presentation::components::inputs::settings::tools::get_field(&app.config.tools, *idx),
                    SettingsTab::DataSources => crate::presentation::components::inputs::settings::data_sources::get_field(&app.config.data_sources, *idx),
                    SettingsTab::Display => crate::presentation::components::inputs::settings::display::get_field(&app.config.display, *idx),
                    SettingsTab::Advanced => crate::presentation::components::inputs::settings::advanced::get_field(&app.config.advanced, *idx),
                };
                let kind = match tab {
                    SettingsTab::Providers => crate::presentation::components::inputs::settings::providers::fields()[*idx].kind,
                    SettingsTab::Agents => crate::presentation::components::inputs::settings::agents::fields()[*idx].kind,
                    SettingsTab::Tools => crate::presentation::components::inputs::settings::tools::fields()[*idx].kind,
                    SettingsTab::DataSources => crate::presentation::components::inputs::settings::data_sources::fields()[*idx].kind,
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
                            SettingsTab::Tools => crate::presentation::components::inputs::settings::tools::toggle_field(&mut app.config.tools, *idx),
                            SettingsTab::DataSources => crate::presentation::components::inputs::settings::data_sources::toggle_field(&mut app.config.data_sources, *idx),
                            SettingsTab::Display => crate::presentation::components::inputs::settings::display::toggle_field(&mut app.config.display, *idx),
                            SettingsTab::Advanced => crate::presentation::components::inputs::settings::advanced::toggle_field(&mut app.config.advanced, *idx),
                        }
                        app.forms[tab_idx].dirty = true;
                    }
                    crate::presentation::form::FieldKind::Button => {}
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
                    SettingsTab::Tools => crate::presentation::components::inputs::settings::tools::toggle_field(&mut app.config.tools, *idx),
                    SettingsTab::DataSources => crate::presentation::components::inputs::settings::data_sources::toggle_field(&mut app.config.data_sources, *idx),
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
            }
            ClickAction::SelectSession(idx) => {
                app.sessions.select(*idx);
            }
            ClickAction::SelectDropdownOption(idx) => {
                let tab = app.router.settings_tab();
                let tab_idx = tab as usize;
                let options: Vec<String> = match tab {
                    SettingsTab::Providers => crate::presentation::components::inputs::settings::providers::fields()[app.forms[tab_idx].focus]
                        .options
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    SettingsTab::Agents => crate::presentation::components::inputs::settings::agents::options_for(
                        app.forms[tab_idx].focus,
                        &app.config,
                    ),
                    SettingsTab::Tools => crate::presentation::components::inputs::settings::tools::fields()[app.forms[tab_idx].focus]
                        .options
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    SettingsTab::DataSources => crate::presentation::components::inputs::settings::data_sources::fields()[app.forms[tab_idx].focus]
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
                    app.forms[tab_idx].dropdown_open = false;
                    match tab {
                        SettingsTab::Providers => {
                            crate::presentation::components::inputs::settings::providers::set_field(&mut app.config, app.forms[tab_idx].focus, &val);
                        }
                        SettingsTab::Agents => {
                            crate::presentation::components::inputs::settings::agents::set_field(&mut app.config.agents, app.forms[tab_idx].focus, &val);
                        }
                        SettingsTab::Tools => {
                            crate::presentation::components::inputs::settings::tools::set_field(&mut app.config.tools, app.forms[tab_idx].focus, &val);
                        }
                        SettingsTab::DataSources => {
                            crate::presentation::components::inputs::settings::data_sources::set_field(&mut app.config.data_sources, app.forms[tab_idx].focus, &val);
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
            }
            ClickAction::AddProvider => {
                use crate::config::ProviderConfig;
                app.config.providers.push(ProviderConfig {
                    name: String::new(),
                    base_url: String::new(),
                    api_key: String::new(),
                    models: Vec::new(),
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
            ClickAction::EditProviderModels(_)
            | ClickAction::AddModel
            | ClickAction::RemoveModel(_)
            | ClickAction::ConfigureSearchOptions(_) => {
                // Models sub-form and options panel: stub for v0.1.
            }
        }
        return;
    }
}

fn handle_scroll(app: &mut AppState, delta: i32) {
    use crate::presentation::handlers::settings::{scroll_list_len, scroll_visible_rows};
    let tab = app.router.settings_tab();
    let tab_idx = tab as usize;
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
        Event::AgentEvent(AgentEvent::Log(entry)) => {
            if let Some(active) = app.sessions.active() {
                let _ = active;
            }
            let tag = entry.agent_tag;
            if matches!(tag, AgentTag::Sys) {
                tracing::debug!(target: "muon::sys", "{}", entry.message);
            }
        }
        Event::AgentEvent(AgentEvent::ClarifierQuestion {
            question,
            responder,
        }) => {
            app.query_input.active = false;
            app.clarifier_pending = Some(ClarifierPending { question, responder });
        }
        Event::AgentEvent(AgentEvent::PlanProposed { plan, responder }) => {
            app.plan_pending = Some(PlanPending { plan, responder });
        }
        Event::AgentEvent(AgentEvent::Final(text)) => {
            tracing::info!(target: "muon::final", "{}", text);
        }
        Event::AgentEvent(AgentEvent::Error(msg)) => {
            tracing::error!(target: "muon::agent", "{}", msg);
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
) -> InfrastructureContext {
    match InfrastructureContext::new_live(cfg, bridge).await {
        Ok(infra) => infra,
        Err(e) => {
            bridge.log(
                AgentTag::Sys,
                LogLevel::Warn,
                format!("live backend failed, falling back to mock: {e}"),
            );
            fallback_mock()
        }
    }
}

#[cfg(any(test, feature = "mock"))]
fn fallback_mock() -> InfrastructureContext {
    InfrastructureContext::mock()
}

#[cfg(not(any(test, feature = "mock")))]
fn fallback_mock() -> InfrastructureContext {
    panic!("live backend failed and mock feature is not enabled");
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
    let infra = build_infrastructure(&config, &bridge_for_init).await;
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
