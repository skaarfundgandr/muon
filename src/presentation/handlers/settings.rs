use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::AppState;
use crate::presentation::components::settings::{advanced, agents, data_sources, display, providers, tools};
use crate::presentation::form::{FieldKind, FormState};
use crate::presentation::views::SettingsTab;

pub fn handle(app: &mut AppState, key: KeyEvent) -> bool {
    let tab = app.router.settings_tab();
    let tab_idx = tab as usize;
    let fields = match tab {
        SettingsTab::Providers => providers::fields(),
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
                        SettingsTab::Providers => {
                            providers::set_field(&mut app.config, app.forms[tab_idx].focus, &val);
                        }
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
        let options: Vec<String> = match tab {
            SettingsTab::Providers => Vec::new(),
            SettingsTab::Agents => agents::options_for(app.forms[tab_idx].focus, &app.config),
            SettingsTab::Tools => tools::fields()[app.forms[tab_idx].focus]
                .options
                .iter()
                .map(|s| s.to_string())
                .collect(),
            SettingsTab::DataSources => data_sources::fields()[app.forms[tab_idx].focus]
                .options
                .iter()
                .map(|s| s.to_string())
                .collect(),
            SettingsTab::Display => display::fields()[app.forms[tab_idx].focus]
                .options
                .iter()
                .map(|s| s.to_string())
                .collect(),
            SettingsTab::Advanced => advanced::fields()[app.forms[tab_idx].focus]
                .options
                .iter()
                .map(|s| s.to_string())
                .collect(),
        };
        let max = options.len();
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if app.forms[tab_idx].dropdown_cursor > 0 {
                    app.forms[tab_idx].dropdown_cursor -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if app.forms[tab_idx].dropdown_cursor + 1 < max {
                    app.forms[tab_idx].dropdown_cursor += 1;
                }
            }
            KeyCode::Enter => {
                let idx = app.forms[tab_idx].dropdown_cursor;
                if idx >= max {
                    return true;
                }
                let val = options[idx].clone();
                app.forms[tab_idx].dropdown_open = false;
                match tab {
                    SettingsTab::Providers => {
                        providers::set_field(&mut app.config, app.forms[tab_idx].focus, &val);
                    }
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
    if key.code == KeyCode::Delete && tab == SettingsTab::Providers {
        let idx = app.forms[0].focus;
        if idx < app.config.providers.len() {
            app.config.providers.swap_remove(idx);
            app.forms[0].dirty = true;
        }
        return true;
    }
    if key.code == KeyCode::Delete && tab == SettingsTab::Tools {
        let idx = app.forms[2].focus;
        if idx < app.config.search.providers.len() {
            app.config.search.providers.swap_remove(idx);
            app.forms[2].dirty = true;
        }
        return true;
    }
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if let Some(visible) = scroll_visible_rows(app, tab) {
                let list_len = scroll_list_len(app, tab);
                let focus = app.forms[tab_idx].focus;
                if focus == 0 && app.forms[tab_idx].scroll_offset > 0 {
                    app.forms[tab_idx].scroll_offset -= 1;
                } else {
                    app.forms[tab_idx].focus_prev(field_count);
                    clamp_focus_to_visible(&mut app.forms[tab_idx], list_len, visible);
                }
            } else {
                app.forms[tab_idx].focus_prev(field_count);
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if let Some(visible) = scroll_visible_rows(app, tab) {
                let list_len = scroll_list_len(app, tab);
                let focus = app.forms[tab_idx].focus;
                if focus + 1 >= list_len && list_len > 0 {
                    let max_offset = list_len.saturating_sub(visible);
                    if app.forms[tab_idx].scroll_offset < max_offset {
                        app.forms[tab_idx].scroll_offset += 1;
                    } else {
                        app.forms[tab_idx].focus_next(field_count);
                    }
                } else {
                    app.forms[tab_idx].focus_next(field_count);
                    clamp_focus_to_visible(&mut app.forms[tab_idx], list_len, visible);
                }
            } else {
                app.forms[tab_idx].focus_next(field_count);
            }
        }
        KeyCode::PageUp => {
            if let Some(visible) = scroll_visible_rows(app, tab) {
                app.forms[tab_idx].scroll_offset =
                    app.forms[tab_idx].scroll_offset.saturating_sub(visible);
            }
        }
        KeyCode::PageDown => {
            if let Some(visible) = scroll_visible_rows(app, tab) {
                let list_len = scroll_list_len(app, tab);
                let max_offset = list_len.saturating_sub(visible);
                app.forms[tab_idx].scroll_offset =
                    (app.forms[tab_idx].scroll_offset + visible).min(max_offset);
            }
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if let Some(visible) = scroll_visible_rows(app, tab) {
                app.forms[tab_idx].scroll_offset =
                    app.forms[tab_idx].scroll_offset.saturating_sub(visible);
            }
        }
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if let Some(visible) = scroll_visible_rows(app, tab) {
                let list_len = scroll_list_len(app, tab);
                let max_offset = list_len.saturating_sub(visible);
                app.forms[tab_idx].scroll_offset =
                    (app.forms[tab_idx].scroll_offset + visible).min(max_offset);
            }
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
            app.router.set_settings_tab(SettingsTab::Providers);
            app.forms[0].reset_edit();
            app.forms[0].focus = 0;
        }
        KeyCode::Char('2') => {
            app.router.set_settings_tab(SettingsTab::Agents);
            app.forms[1].reset_edit();
            app.forms[1].focus = 0;
        }
        KeyCode::Char('3') => {
            app.router.set_settings_tab(SettingsTab::Tools);
            app.forms[2].reset_edit();
            app.forms[2].focus = 0;
        }
        KeyCode::Char('4') => {
            app.router.set_settings_tab(SettingsTab::DataSources);
            app.forms[3].reset_edit();
            app.forms[3].focus = 0;
        }
        KeyCode::Char('5') => {
            app.router.set_settings_tab(SettingsTab::Display);
            app.forms[4].reset_edit();
            app.forms[4].focus = 0;
        }
        KeyCode::Char('6') => {
            app.router.set_settings_tab(SettingsTab::Advanced);
            app.forms[5].reset_edit();
            app.forms[5].focus = 0;
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            let focused = app.forms[tab_idx].focus;
            if focused < field_count {
                match fields[focused].kind {
                    FieldKind::Text | FieldKind::Number => {
                        let val = match tab {
                            SettingsTab::Providers => {
                                providers::get_field(&app.config, focused)
                            }
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
                            SettingsTab::Providers => {
                                providers::toggle_field(&mut app.config, focused);
                            }
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

/// Returns the number of list rows visible in the scrollable list for the given
/// tab, or `None` if the tab does not use a scrollable list layout.
pub(crate) fn scroll_visible_rows(app: &AppState, tab: SettingsTab) -> Option<usize> {
    let (row_height, chrome_rows) = match tab {
        SettingsTab::Providers => (5u16, 3u16 + 3u16 + 1u16 + 2u16),
        SettingsTab::Tools => (4u16, 3u16 + 3u16 + 2u16 + 2u16),
        _ => return None,
    };
    let settings_area = app.term_rows.saturating_sub(4);
    let list_area = settings_area.saturating_sub(chrome_rows);
    Some(((list_area / row_height) as usize).max(1))
}

/// Returns the length of the scrollable list for the given tab.
pub(crate) fn scroll_list_len(app: &AppState, tab: SettingsTab) -> usize {
    match tab {
        SettingsTab::Providers => app.config.providers.len(),
        SettingsTab::Tools => app.config.search.providers.len(),
        _ => 0,
    }
}

/// Clamp focus index to remain within the currently visible window of the
/// scrollable list. If focus is below the visible range, it is pulled to the
/// last visible row. If above, it is pulled to the first visible row.
pub(crate) fn clamp_focus_to_visible(form: &mut FormState, list_len: usize, visible: usize) {
    if list_len == 0 || visible == 0 {
        return;
    }
    let window_end = (form.scroll_offset + visible).min(list_len);
    if form.focus < form.scroll_offset {
        form.focus = form.scroll_offset;
    } else if form.focus >= window_end {
        form.focus = window_end.saturating_sub(1);
    }
}
