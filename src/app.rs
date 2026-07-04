use std::time::Duration;

use crossterm::event::{poll, read, Event as CrosstermEvent};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tokio::sync::mpsc;

use crate::application::pipeline::{PipelineStage, PipelineState};
use crate::presentation::components::query_input::QueryInput;
use crate::presentation::form::FormState;
use crate::presentation::views::{RenderParams, View, ViewRouter};
use crate::session::SessionService;

#[derive(Debug)]
pub struct AppState {
    pub router: ViewRouter,
    pub running: bool,
    tick_count: u64,
    pub config: crate::config::MuonConfig,
    pub forms: [FormState; 5],
    pub query_input: QueryInput,
    pub sessions: SessionService,
    pub pipeline: PipelineState,
}

#[derive(Debug, Clone)]
pub enum Event {
    Key(crossterm::event::KeyEvent),
    Tick,
    AgentEvent(String),
}

fn setup_terminal() -> crate::error::Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) {
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
}

fn render(frame: &mut ratatui::Frame, app: &AppState) {
    let view = app.router.active();
    view.render(frame, frame.area(), &RenderParams {
        query_input: &app.query_input,
        sessions: app.sessions.list(),
        pipeline: &app.pipeline,
        config: &app.config,
        forms: &app.forms,
        settings_tab: app.router.settings_tab(),
    });
}

fn handle_event(app: &mut AppState, event: Event) {
    match event {
        Event::Key(key) => {
            crate::presentation::handlers::handle_key(app, key);
        }
        Event::Tick => {
            app.tick_count += 1;
            if app.pipeline.is_running() && app.tick_count.is_multiple_of(4) {
                app.pipeline.advance();
                if app.pipeline.stage == PipelineStage::Complete {
                    app.router.transition(View::Results);
                }
            }
        }
        Event::AgentEvent(_) => {}
    }
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> crate::error::Result<()> {
    let mut app = AppState {
        router: ViewRouter::new(),
        running: true,
        tick_count: 0,
        config: crate::config::MuonConfig::load(),
        forms: std::array::from_fn(|_| FormState::default()),
        query_input: QueryInput::default(),
        sessions: SessionService::new(),
        pipeline: PipelineState::default(),
    };

    let (tx, mut rx) = mpsc::unbounded_channel::<Event>();

    tokio::spawn(async move {
        loop {
            match poll(Duration::from_millis(250)) {
                Ok(true) => {
                    if let Ok(CrosstermEvent::Key(key)) = read()
                        && tx.send(Event::Key(key)).is_err()
                    {
                        break;
                    }
                }
                Ok(false) => {
                    let _ = tx.send(Event::Tick);
                }
                Err(_) => break,
            }
        }
    });

    while app.running {
        terminal.draw(|f| render(f, &app))?;
        if let Some(event) = rx.recv().await {
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
