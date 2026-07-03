use std::time::Duration;

use crossterm::event::{Event as CrosstermEvent, poll, read};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tokio::sync::mpsc;

use crate::application::pipeline::{PipelineStage, PipelineState};
use crate::presentation::components::query_input::QueryInput;
use crate::presentation::components::{settings_advanced, settings_agents, settings_data_sources, settings_display, settings_tools};
use crate::presentation::form::{FieldKind, FormState};
use crate::presentation::views::{SettingsTab, View, ViewRouter};
use crate::session::SessionService;
use crossterm::event::{KeyCode, KeyModifiers};

#[derive(Debug)]
pub struct AppState {
    router: ViewRouter,
    running: bool,
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
    match app.router.active() {
        crate::presentation::View::Welcome => {
            crate::presentation::layouts::welcome::render(frame, frame.area());
        }
        crate::presentation::View::Dashboard => {
            crate::presentation::layouts::dashboard::render(
                frame,
                frame.area(),
                &app.query_input,
                app.sessions.list(),
            );
        }
        crate::presentation::View::Progress => {
            crate::presentation::layouts::progress::render(
                frame,
                frame.area(),
                &app.pipeline,
            );
        }
        crate::presentation::View::Results => {
            crate::presentation::layouts::results::render(
                frame,
                frame.area(),
                &app.pipeline,
            );
        }
        crate::presentation::View::Settings => {
            let tab = app.router.settings_tab();
            let tab_idx = tab as usize;
            let form = &app.forms[tab_idx];
            crate::presentation::layouts::settings::render(
                frame,
                frame.area(),
                tab,
                &app.config,
                form,
            );
        }
    }
}

fn handle_event(app: &mut AppState, event: Event) {
    match event {
        Event::Key(key) => {
            let view = app.router.active();

            if view == View::Settings {
                let tab = app.router.settings_tab();
                let tab_idx = tab as usize;
                let fields = match tab {
                    SettingsTab::Agents => settings_agents::fields(),
                    SettingsTab::Tools => settings_tools::fields(),
                    SettingsTab::DataSources => settings_data_sources::fields(),
                    SettingsTab::Display => settings_display::fields(),
                    SettingsTab::Advanced => settings_advanced::fields(),
                };
                let field_count = fields.len();

                if key.code == KeyCode::Esc {
                    if app.forms[tab_idx].dropdown_open {
                        app.forms[tab_idx].dropdown_open = false;
                    } else if app.forms[tab_idx].is_editing() {
                        app.forms[tab_idx].cancel_edit();
                    } else {
                        app.running = false;
                    }
                } else if app.forms[tab_idx].is_editing() {
                    app.forms[tab_idx].handle_edit_key(key);
                    if key.code == KeyCode::Enter
                        && let Some(val) = app.forms[tab_idx].confirm_edit()
                    {
                        match tab {
                            SettingsTab::Agents => {
                                settings_agents::set_field(&mut app.config.agents, app.forms[tab_idx].focus, &val);
                            }
                            SettingsTab::Tools => {
                                settings_tools::set_field(&mut app.config.tools, app.forms[tab_idx].focus, &val);
                            }
                            SettingsTab::DataSources => {
                                settings_data_sources::set_field(&mut app.config.data_sources, app.forms[tab_idx].focus, &val);
                            }
                            SettingsTab::Display => {
                                settings_display::set_field(&mut app.config.display, app.forms[tab_idx].focus, &val);
                            }
                            SettingsTab::Advanced => {
                                settings_advanced::set_field(&mut app.config.advanced, app.forms[tab_idx].focus, &val);
                            }
                        }
                    }
                } else if app.forms[tab_idx].dropdown_open {
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            if app.forms[tab_idx].dropdown_cursor > 0 {
                                app.forms[tab_idx].dropdown_cursor -= 1;
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            let max = fields[app.forms[tab_idx].focus].options.len();
                            if app.forms[tab_idx].dropdown_cursor + 1 < max {
                                app.forms[tab_idx].dropdown_cursor += 1;
                            }
                        }
                        KeyCode::Enter => {
                            let idx = app.forms[tab_idx].dropdown_cursor;
                            let val = fields[app.forms[tab_idx].focus].options[idx].to_string();
                            app.forms[tab_idx].dropdown_open = false;
                            match tab {
                                SettingsTab::Agents => {
                                    settings_agents::set_field(&mut app.config.agents, app.forms[tab_idx].focus, &val);
                                }
                                SettingsTab::Tools => {
                                    settings_tools::set_field(&mut app.config.tools, app.forms[tab_idx].focus, &val);
                                }
                                SettingsTab::DataSources => {
                                    settings_data_sources::set_field(&mut app.config.data_sources, app.forms[tab_idx].focus, &val);
                                }
                                SettingsTab::Display => {
                                    settings_display::set_field(&mut app.config.display, app.forms[tab_idx].focus, &val);
                                }
                                SettingsTab::Advanced => {
                                    settings_advanced::set_field(&mut app.config.advanced, app.forms[tab_idx].focus, &val);
                                }
                            }
                            app.forms[tab_idx].dirty = true;
                        }
                        KeyCode::Esc => {
                            app.forms[tab_idx].dropdown_open = false;
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.forms[tab_idx].focus_prev(field_count);
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.forms[tab_idx].focus_next(field_count);
                        }
                        KeyCode::Right | KeyCode::Char('l') => {
                            let new_tab = app.router.settings_tab().next();
                            app.router.set_settings_tab(new_tab);
                            let new_idx = new_tab as usize;
                            app.forms[new_idx].reset_edit();
                            app.forms[new_idx].focus = 0;
                        }
                        KeyCode::Left | KeyCode::Char('h') => {
                            let new_tab = app.router.settings_tab().prev();
                            app.router.set_settings_tab(new_tab);
                            let new_idx = new_tab as usize;
                            app.forms[new_idx].reset_edit();
                            app.forms[new_idx].focus = 0;
                        }
                        KeyCode::Char('1') => {
                            app.router.set_settings_tab(SettingsTab::Agents);
                            app.forms[0].reset_edit();
                            app.forms[0].focus = 0;
                        }
                        KeyCode::Char('2') => {
                            app.router.set_settings_tab(SettingsTab::Tools);
                            app.forms[1].reset_edit();
                            app.forms[1].focus = 0;
                        }
                        KeyCode::Char('3') => {
                            app.router.set_settings_tab(SettingsTab::DataSources);
                            app.forms[2].reset_edit();
                            app.forms[2].focus = 0;
                        }
                        KeyCode::Char('4') => {
                            app.router.set_settings_tab(SettingsTab::Display);
                            app.forms[3].reset_edit();
                            app.forms[3].focus = 0;
                        }
                        KeyCode::Char('5') => {
                            app.router.set_settings_tab(SettingsTab::Advanced);
                            app.forms[4].reset_edit();
                            app.forms[4].focus = 0;
                        }
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            let focused = app.forms[tab_idx].focus;
                            if focused < field_count {
                                match fields[focused].kind {
                                    FieldKind::Text | FieldKind::Number => {
                                        let val = match tab {
                                            SettingsTab::Agents => settings_agents::get_field(&app.config.agents, focused),
                                            SettingsTab::Tools => settings_tools::get_field(&app.config.tools, focused),
                                            SettingsTab::DataSources => settings_data_sources::get_field(&app.config.data_sources, focused),
                                            SettingsTab::Display => settings_display::get_field(&app.config.display, focused),
                                            SettingsTab::Advanced => settings_advanced::get_field(&app.config.advanced, focused),
                                        };
                                        app.forms[tab_idx].begin_edit(&val);
                                    }
                                    FieldKind::Dropdown => {
                                        app.forms[tab_idx].open_dropdown();
                                    }
                                    FieldKind::Checkbox => {
                                        match tab {
                                            SettingsTab::Agents => {
                                                settings_agents::toggle_field(&mut app.config.agents, focused);
                                            }
                                            SettingsTab::Tools => {
                                                settings_tools::toggle_field(&mut app.config.tools, focused);
                                            }
                                            SettingsTab::DataSources => {
                                                settings_data_sources::toggle_field(&mut app.config.data_sources, focused);
                                            }
                                            SettingsTab::Display => {
                                                settings_display::toggle_field(&mut app.config.display, focused);
                                            }
                                            SettingsTab::Advanced => {
                                                settings_advanced::toggle_field(&mut app.config.advanced, focused);
                                            }
                                        }
                                        app.forms[tab_idx].dirty = true;
                                    }
                                    FieldKind::Button => {}
                                }
                            }
                        }
                        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.config.save();
                            for form in &mut app.forms {
                                form.dirty = false;
                            }
                        }
                        _ => {
                            let _ = app.router.handle_key(key);
                        }
                    }
                }
            } else {
                if key.code == KeyCode::Esc {
                    if view == View::Progress {
                        app.pipeline.cancel();
                        app.router.transition(View::Dashboard);
                    } else {
                        app.running = false;
                    }
                } else if key.code == KeyCode::Enter && view == View::Welcome {
                    app.router.transition(View::Dashboard);
                } else if view == View::Dashboard {
                    if app.query_input.active {
                        match key.code {
                            KeyCode::Char(c) => app.query_input.insert_char(c),
                            KeyCode::Backspace => app.query_input.backspace(),
                            KeyCode::Delete => app.query_input.delete(),
                            KeyCode::Left => app.query_input.cursor_left(),
                            KeyCode::Right => app.query_input.cursor_right(),
                            KeyCode::Home => app.query_input.cursor_home(),
                            KeyCode::End => app.query_input.cursor_end(),
                            KeyCode::Enter => {
                                let query = app.query_input.submit();
                                if !query.is_empty() {
                                    let _ = app.sessions.create(&query);
                                }
                                app.pipeline.start();
                                app.router.transition(View::Progress);
                            }
                            KeyCode::Esc => app.query_input.active = false,
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Enter | KeyCode::Char('i') => {
                                app.query_input.active = true;
                            }
                            _ => {
                                let _ = app.router.handle_key(key);
                            }
                        }
                    }
                } else {
                    let _ = app.router.handle_key(key);
                }
            }
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
