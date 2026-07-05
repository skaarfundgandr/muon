use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::AppState;
use crate::presentation::components::settings::{advanced, agents, data_sources, display, tools};
use crate::presentation::form::FieldKind;
use crate::presentation::views::SettingsTab;

pub fn handle(app: &mut AppState, key: KeyEvent) -> bool {
    let tab = app.router.settings_tab();
    let tab_idx = tab as usize;
    let fields = match tab {
        SettingsTab::Agents => agents::fields(),
        SettingsTab::Tools => tools::fields(),
        SettingsTab::DataSources => data_sources::fields(),
        SettingsTab::Display => display::fields(),
        SettingsTab::Advanced => advanced::fields(),
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
        return true;
    }

    if app.forms[tab_idx].is_editing() {
        app.forms[tab_idx].handle_edit_key(key);
        if key.code == KeyCode::Enter
            && let Some(val) = app.forms[tab_idx].confirm_edit() {
                match tab {
                    SettingsTab::Agents => {
                        agents::set_field(&mut app.config.agents, app.forms[tab_idx].focus, &val);
                    }
                    SettingsTab::Tools => {
                        tools::set_field(&mut app.config.tools, app.forms[tab_idx].focus, &val);
                    }
                    SettingsTab::DataSources => {
                        data_sources::set_field(
                            &mut app.config.data_sources,
                            app.forms[tab_idx].focus,
                            &val,
                        );
                    }
                    SettingsTab::Display => {
                        display::set_field(&mut app.config.display, app.forms[tab_idx].focus, &val);
                    }
                    SettingsTab::Advanced => {
                        advanced::set_field(&mut app.config.advanced, app.forms[tab_idx].focus, &val);
                    }
                }
            }
        return true;
    }

    if app.forms[tab_idx].dropdown_open {
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
                        agents::set_field(&mut app.config.agents, app.forms[tab_idx].focus, &val);
                    }
                    SettingsTab::Tools => {
                        tools::set_field(&mut app.config.tools, app.forms[tab_idx].focus, &val);
                    }
                    SettingsTab::DataSources => {
                        data_sources::set_field(
                            &mut app.config.data_sources,
                            app.forms[tab_idx].focus,
                            &val,
                        );
                    }
                    SettingsTab::Display => {
                        display::set_field(&mut app.config.display, app.forms[tab_idx].focus, &val);
                    }
                    SettingsTab::Advanced => {
                        advanced::set_field(
                            &mut app.config.advanced,
                            app.forms[tab_idx].focus,
                            &val,
                        );
                    }
                }
                app.forms[tab_idx].dirty = true;
            }
            KeyCode::Esc => {
                app.forms[tab_idx].dropdown_open = false;
            }
            _ => {}
        }
        return true;
    }

    // Normal navigation mode
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
                            SettingsTab::Agents => {
                                agents::get_field(&app.config.agents, focused)
                            }
                            SettingsTab::Tools => tools::get_field(&app.config.tools, focused),
                            SettingsTab::DataSources => {
                                data_sources::get_field(&app.config.data_sources, focused)
                            }
                            SettingsTab::Display => {
                                display::get_field(&app.config.display, focused)
                            }
                            SettingsTab::Advanced => {
                                advanced::get_field(&app.config.advanced, focused)
                            }
                        };
                        app.forms[tab_idx].begin_edit(&val);
                    }
                    FieldKind::Dropdown => {
                        if app.forms[tab_idx].dropdown_open {
                            app.forms[tab_idx].dropdown_open = false;
                        } else {
                            app.forms[tab_idx].open_dropdown();
                        }
                    }
                    FieldKind::Checkbox => {
                        match tab {
                            SettingsTab::Agents => {
                                agents::toggle_field(&mut app.config.agents, focused);
                            }
                            SettingsTab::Tools => {
                                tools::toggle_field(&mut app.config.tools, focused);
                            }
                            SettingsTab::DataSources => {
                                data_sources::toggle_field(
                                    &mut app.config.data_sources,
                                    focused,
                                );
                            }
                            SettingsTab::Display => {
                                display::toggle_field(&mut app.config.display, focused);
                            }
                            SettingsTab::Advanced => {
                                advanced::toggle_field(&mut app.config.advanced, focused);
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
    true
}
