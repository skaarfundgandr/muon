use futures::StreamExt;
use std::sync::Arc;
use std::time::Duration;

use crossterm::event::{poll, read, Event as CrosstermEvent};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tokio::sync::mpsc;

use crate::application::bridge::{AgentEvent, BridgeChannels};
use crate::application::pipeline::PipelineState;
use crate::config::MuonConfig;
use crate::domain::models::log_entry::{AgentTag, LogLevel};
use crate::infrastructure::context::InfrastructureContext;
use crate::presentation::form::FormState;
use crate::presentation::components::query_input::QueryInput;
use crate::presentation::views::ViewRouter;
use crate::application::session::SessionService;

use crate::presentation::handlers::events::handle_event;
use crate::presentation::render::render;
use crate::presentation::state::AppState;
use crate::presentation::terminal::{restore_terminal, setup_terminal};
use crate::presentation::types::Event;

async fn build_infrastructure(
    cfg: &MuonConfig,
    bridge: &BridgeChannels,
) -> Result<InfrastructureContext, crate::domain::error::MuonError> {
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
) -> crate::domain::error::Result<()> {
    let config = MuonConfig::load();

    let mut config_reload_rx = {
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
        report_scroll: 0,
        source_scroll: 0,
        full_report_mode: false,
        status_flash: None,
        clipboard: None,
        export_session_id: None,
        pipeline_handle: None,
        event_tx: None,
        pending_config: None,
    };

    if let Some(palette) = crate::presentation::theme::for_name(&app.config.display.visual_theme) {
        crate::presentation::theme::replace(palette);
    }

    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<Event>();
    let (agent_tx, mut agent_rx) = mpsc::unbounded_channel::<AgentEvent>();
    let key_tx = event_tx.clone();
    app.event_tx = Some(event_tx.clone());
    app.agent_tx = Some(agent_tx.clone());

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
    match infra.session_store.list().await {
        Ok(list) => {
            let mapped: Vec<crate::application::session::SessionSummary> =
                list.into_iter().map(Into::into).collect();
            app.sessions.replace_all(mapped);
        }
        Err(e) => {
            bridge_for_init.log(
                AgentTag::Sys,
                LogLevel::Warn,
                format!("failed to load sessions: {e}"),
            );
        }
    }
    app.infra = Some(Arc::new(infra));
    app.agent_tx = Some(agent_tx);
    if config.providers.is_empty() {
        app.status_flash = Some((
            std::time::Instant::now(),
            "No providers configured — open Settings → Providers (F4)".to_string(),
            crate::presentation::components::chrome::toast::ToastKind::Info,
        ));
    }

    while app.running {
        terminal.draw(|f| render(f, &mut app))?;
        if let Some(event) = event_rx.recv().await {
            handle_event(&mut app, event);
        }
    }

    Ok(())
}

pub async fn run() -> crate::domain::error::Result<()> {
    let observability = crate::infrastructure::observability::Observability::init("muon")?;
    let mut terminal = setup_terminal()?;
    let result = run_loop(&mut terminal).await;
    restore_terminal(&mut terminal);
    if let Err(e) = observability.shutdown().await {
        tracing::warn!(target: "muon::observability", "shutdown failed: {e}");
    }
    result
}
