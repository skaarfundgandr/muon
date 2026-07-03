use std::time::Duration;

use crossterm::event::{Event as CrosstermEvent, poll, read};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tokio::sync::mpsc;

use crate::presentation::{SettingsTab, View, ViewRouter};

#[derive(Debug)]
pub struct AppState {
    router: ViewRouter,
    running: bool,
    tick_count: u64,
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
    match app.router.active() {
        crate::presentation::View::Welcome => {
            crate::presentation::layouts::welcome::render(frame, frame.area());
        }
        crate::presentation::View::Dashboard => {
            crate::presentation::layouts::dashboard::render(frame, frame.area());
        }
        crate::presentation::View::Progress => {
            let elapsed = app.tick_count * 250 / 1000;
            crate::presentation::layouts::progress::render(frame, frame.area(), elapsed);
        }
        crate::presentation::View::Results => {
            let total = app.tick_count * 250 / 1000;
            crate::presentation::layouts::results::render(frame, frame.area(), total);
        }
        crate::presentation::View::Settings => {
            let tab = app.router.settings_tab();
            crate::presentation::layouts::settings::render(frame, frame.area(), tab);
        }
    }
}

fn handle_event(app: &mut AppState, event: Event) {
    match event {
        Event::Key(key) => {
            if key.code == crossterm::event::KeyCode::Esc {
                app.running = false;
            } else if key.code == crossterm::event::KeyCode::Enter
                && app.router.active() == View::Welcome
            {
                app.router.transition(View::Dashboard);
            } else if app.router.active() == View::Settings {
                match key.code {
                    crossterm::event::KeyCode::Right | crossterm::event::KeyCode::Char('l') => {
                        app.router.set_settings_tab(app.router.settings_tab().next());
                    }
                    crossterm::event::KeyCode::Left | crossterm::event::KeyCode::Char('h') => {
                        app.router.set_settings_tab(app.router.settings_tab().prev());
                    }
                    crossterm::event::KeyCode::Char('1') => {
                        app.router.set_settings_tab(SettingsTab::Agents);
                    }
                    crossterm::event::KeyCode::Char('2') => {
                        app.router.set_settings_tab(SettingsTab::Tools);
                    }
                    crossterm::event::KeyCode::Char('3') => {
                        app.router.set_settings_tab(SettingsTab::DataSources);
                    }
                    crossterm::event::KeyCode::Char('4') => {
                        app.router.set_settings_tab(SettingsTab::Display);
                    }
                    crossterm::event::KeyCode::Char('5') => {
                        app.router.set_settings_tab(SettingsTab::Advanced);
                    }
                    _ => {
                        let _ = app.router.handle_key(key);
                    }
                }
            } else {
                app.router.handle_key(key);
            }
        }
        Event::Tick => {
            app.tick_count += 1;
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
