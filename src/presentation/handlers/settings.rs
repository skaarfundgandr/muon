use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{AppState, ActivePopup};
use crate::presentation::components::settings::{advanced, agents, data_sources, display, providers, tools};
use crate::presentation::form::{FieldKind, FormState};
use crate::presentation::views::SettingsTab;

pub fn handle(app: &mut AppState, key: KeyEvent) -> bool {
    if app.active_popup.is_some() {
        return handle_popup_key(app, key);
    }

    let tab = app.router.settings_tab();
    let tab_idx = tab as usize;
    let fields = match tab {
        SettingsTab::Providers => providers::fields(&app.config),
        SettingsTab::Agents => agents::fields().to_vec(),
        SettingsTab::Tools => tools::fields(&app.config),
        SettingsTab::DataSources => data_sources::fields(&app.config),
        SettingsTab::Display => display::fields().to_vec(),
        SettingsTab::Advanced => advanced::fields().to_vec(),
    };
    let field_count = fields.len();
    if app.forms[tab_idx].focus >= field_count && field_count > 0 {
        app.forms[tab_idx].focus = field_count - 1;
    }

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
                            tools::set_field(&mut app.config, app.forms[tab_idx].focus, &val);
                        }
                        SettingsTab::DataSources => {
                            data_sources::set_field(&mut app.config, app.forms[tab_idx].focus, &val);
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
            SettingsTab::Providers => providers::fields(&app.config)[app.forms[tab_idx].focus]
                .options
                .iter()
                .map(|s| s.to_string())
                .collect(),
            SettingsTab::Agents => agents::options_for(app.forms[tab_idx].focus, &app.config),
            SettingsTab::Tools => tools::fields(&app.config)[app.forms[tab_idx].focus]
                .options
                .iter()
                .map(|s| s.to_string())
                .collect(),
            SettingsTab::DataSources => data_sources::fields(&app.config)[app.forms[tab_idx].focus]
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
                if val.starts_with("<no models") {
                    app.forms[tab_idx].dropdown_open = false;
                    return true;
                }
                app.forms[tab_idx].dropdown_open = false;
                match tab {
                    SettingsTab::Providers => {
                        providers::set_field(&mut app.config, app.forms[tab_idx].focus, &val);
                    }
                    SettingsTab::Agents => {
                        agents::set_field(&mut app.config.agents, app.forms[tab_idx].focus, &val);
                    }
                    SettingsTab::Tools => {
                        tools::set_field(&mut app.config, app.forms[tab_idx].focus, &val);
                    }
                    SettingsTab::DataSources => {
                        data_sources::set_field(&mut app.config, app.forms[tab_idx].focus, &val);
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
        let focus = app.forms[0].focus;
        let n = app.config.providers.len();
        if focus < 5 * n {
            let provider_idx = focus / 5;
            app.config.providers.swap_remove(provider_idx);
            app.forms[0].dirty = true;
            if app.forms[0].focus >= 5 * app.config.providers.len() && !app.config.providers.is_empty() {
                app.forms[0].focus = 5 * app.config.providers.len() - 5;
            }
        }
        return true;
    }
    if key.code == KeyCode::Delete && tab == SettingsTab::Tools {
        let focus = app.forms[2].focus;
        let n = app.config.search.providers.len();
        if focus < 5 * n {
            let provider_idx = focus / 5;
            app.config.search.providers.swap_remove(provider_idx);
            app.forms[2].dirty = true;
            if app.forms[2].focus >= 5 * app.config.search.providers.len() && !app.config.search.providers.is_empty() {
                app.forms[2].focus = 5 * app.config.search.providers.len() - 5;
            }
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
                            SettingsTab::Tools => tools::get_field(&app.config, focused),
                            SettingsTab::DataSources => {
                                data_sources::get_field(&app.config, focused)
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
                                tools::toggle_field(&mut app.config, focused);
                            }
                            SettingsTab::DataSources => {
                                data_sources::toggle_field(&mut app.config, focused);
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
                    FieldKind::Button => {
                        match tab {
                            SettingsTab::Providers => {
                                let n = app.config.providers.len();
                                if focused == 5 * n {
                                    app.config.providers.push(crate::config::ProviderConfig {
                                        name: String::new(),
                                        base_url: String::new(),
                                        api_key: String::new(),
                                        models: Vec::new(),
                                        provider_type: crate::config::ProviderType::OpenAICompatible,
                                    });
                                    app.forms[tab_idx].focus = 5 * app.config.providers.len() - 5;
                                    app.forms[tab_idx].dirty = true;
                                } else {
                                    let provider_idx = focused / 5;
                                    let sub_idx = focused % 5;
                                    if sub_idx == 4 {
                                        app.active_popup = Some(ActivePopup::EditModels {
                                            provider_idx,
                                            focus_idx: 0,
                                            edit_buffer: None,
                                            edit_cursor: 0,
                                            scroll_offset: 0,
                                        });
                                    }
                                }
                            }
                            SettingsTab::Tools => {
                                let n = app.config.search.providers.len();
                                if focused == 5 * n {
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
                                } else if focused < 5 * n {
                                    let provider_idx = focused / 5;
                                    let sub_idx = focused % 5;
                                    if sub_idx == 3 {
                                        app.active_popup = Some(ActivePopup::ConfigureSearch {
                                            provider_idx,
                                            focus_idx: 0,
                                            edit_buffer: None,
                                            edit_cursor: 0,
                                        });
                                    } else if sub_idx == 4 && provider_idx < app.config.search.providers.len() {
                                        app.config.search.providers.remove(provider_idx);
                                        app.forms[tab_idx].focus = if provider_idx > 0 { 5 * (provider_idx - 1) } else { 0 };
                                        app.forms[tab_idx].dirty = true;
                                    }
                                }
                            }
                            SettingsTab::DataSources if focused == 6 => {
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
                            _ => {}
                        }
                    }
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
        SettingsTab::Tools => (6u16, 10u16),
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

fn handle_popup_key(app: &mut AppState, key: KeyEvent) -> bool {
    let mut popup = match app.active_popup.take() {
        Some(p) => p,
        None => return false,
    };
    
    // 1. Esc: close popup or cancel edit
    if key.code == KeyCode::Esc {
        match &mut popup {
            ActivePopup::EditModels { edit_buffer, .. } | ActivePopup::ConfigureSearch { edit_buffer, .. } => {
                if edit_buffer.is_some() {
                    *edit_buffer = None;
                    app.active_popup = Some(popup);
                } else {
                    app.active_popup = None;
                }
            }
            ActivePopup::PlanApproval { .. } => {}
        }
        return true;
    }

    // 2. If in edit mode
    let is_editing = match &popup {
        ActivePopup::EditModels { edit_buffer, .. } => edit_buffer.is_some(),
        ActivePopup::ConfigureSearch { edit_buffer, .. } => edit_buffer.is_some(),
        ActivePopup::PlanApproval { .. } => false,
    };

    if is_editing {
        match &mut popup {
            ActivePopup::EditModels { edit_buffer, edit_cursor, focus_idx, provider_idx, .. } => {
                if let Some(buf) = edit_buffer {
                    match key.code {
                        KeyCode::Char(c) => {
                            buf.insert(*edit_cursor, c);
                            *edit_cursor += 1;
                        }
                        KeyCode::Backspace => {
                            if *edit_cursor > 0 {
                                *edit_cursor -= 1;
                                buf.remove(*edit_cursor);
                            }
                        }
                        KeyCode::Delete => {
                            if *edit_cursor < buf.len() {
                                buf.remove(*edit_cursor);
                            }
                        }
                        KeyCode::Left => {
                            if *edit_cursor > 0 {
                                *edit_cursor -= 1;
                            }
                        }
                        KeyCode::Right => {
                            if *edit_cursor < buf.len() {
                                *edit_cursor += 1;
                            }
                        }
                        KeyCode::Enter => {
                            // Confirm edit! Save back to config
                            let val = edit_buffer.take().unwrap_or_default();
                            *edit_cursor = 0;
                            let model_idx = *focus_idx / 3;
                            let sub_idx = *focus_idx % 3;
                            if model_idx < app.config.providers[*provider_idx].models.len() {
                                let m = &mut app.config.providers[*provider_idx].models[model_idx];
                                match sub_idx {
                                    0 => m.name = val,
                                    1 => m.model_id = val,
                                    _ => {}
                                }
                            }
                            app.forms[SettingsTab::Providers as usize].dirty = true;
                        }
                        _ => {}
                    }
                }
            }
            ActivePopup::ConfigureSearch { edit_buffer, edit_cursor, focus_idx, provider_idx } => {
                if let Some(buf) = edit_buffer {
                    match key.code {
                        KeyCode::Char(c) => {
                            buf.insert(*edit_cursor, c);
                            *edit_cursor += 1;
                        }
                        KeyCode::Backspace => {
                            if *edit_cursor > 0 {
                                *edit_cursor -= 1;
                                buf.remove(*edit_cursor);
                            }
                        }
                        KeyCode::Delete => {
                            if *edit_cursor < buf.len() {
                                buf.remove(*edit_cursor);
                            }
                        }
                        KeyCode::Left => {
                            if *edit_cursor > 0 {
                                *edit_cursor -= 1;
                            }
                        }
                        KeyCode::Right => {
                            if *edit_cursor < buf.len() {
                                *edit_cursor += 1;
                            }
                        }
                        KeyCode::Enter => {
                            // Confirm edit! Save back to config
                            let val = edit_buffer.take().unwrap_or_default();
                            *edit_cursor = 0;
                            let p = &mut app.config.search.providers[*provider_idx];
                            if *focus_idx == 0 {
                                p.name = val;
                            } else if *focus_idx == 1 {
                                p.api_key = val;
                            } else if *focus_idx == 2 {
                                p.max_results = val.parse().ok();
                            }
                            app.forms[SettingsTab::Tools as usize].dirty = true;
                        }
                        _ => {}
                    }
                }
            }
            ActivePopup::PlanApproval { .. } => {}
        }
        app.active_popup = Some(popup);
        return true;
    }

    // 3. Normal navigation mode in popup
    let max_focus = match &popup {
        ActivePopup::EditModels { provider_idx, .. } => {
            let m = app.config.providers[*provider_idx].models.len();
            3 * m + 2
        }
        ActivePopup::ConfigureSearch { .. } => {
            5
        }
        ActivePopup::PlanApproval { .. } => 0,
    };

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            match &mut popup {
                ActivePopup::EditModels { focus_idx, .. } | ActivePopup::ConfigureSearch { focus_idx, .. } => {
                    *focus_idx = (*focus_idx + max_focus - 1) % max_focus;
                }
                ActivePopup::PlanApproval { .. } => {}
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            match &mut popup {
                ActivePopup::EditModels { focus_idx, .. } | ActivePopup::ConfigureSearch { focus_idx, .. } => {
                    *focus_idx = (*focus_idx + 1) % max_focus;
                }
                ActivePopup::PlanApproval { .. } => {}
            }
        }
        KeyCode::Tab => {
            match &mut popup {
                ActivePopup::EditModels { focus_idx, .. } | ActivePopup::ConfigureSearch { focus_idx, .. } => {
                    *focus_idx = (*focus_idx + 1) % max_focus;
                }
                ActivePopup::PlanApproval { .. } => {}
            }
        }
        KeyCode::BackTab => {
            match &mut popup {
                ActivePopup::EditModels { focus_idx, .. } | ActivePopup::ConfigureSearch { focus_idx, .. } => {
                    *focus_idx = (*focus_idx + max_focus - 1) % max_focus;
                }
                ActivePopup::PlanApproval { .. } => {}
            }
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            match &mut popup {
                ActivePopup::EditModels { provider_idx, focus_idx, edit_buffer, edit_cursor, scroll_offset } => {
                    let m = app.config.providers[*provider_idx].models.len();
                    if *focus_idx < 3 * m {
                        let model_idx = *focus_idx / 3;
                        let sub_idx = *focus_idx % 3;
                        match sub_idx {
                            0 => {
                                let current = app.config.providers[*provider_idx].models[model_idx].name.clone();
                                *edit_buffer = Some(current.clone());
                                *edit_cursor = current.len();
                            }
                            1 => {
                                let current = app.config.providers[*provider_idx].models[model_idx].model_id.clone();
                                *edit_buffer = Some(current.clone());
                                *edit_cursor = current.len();
                            }
                            2 if model_idx < app.config.providers[*provider_idx].models.len() => {
                                // Remove model
                                app.config.providers[*provider_idx].models.remove(model_idx);
                                *focus_idx = 0;
                                *scroll_offset = 0;
                                app.forms[SettingsTab::Providers as usize].dirty = true;
                            }
                            _ => {}
                        }
                    } else if *focus_idx == 3 * m {
                        // [+ Add Model]
                        app.config.providers[*provider_idx].models.push(crate::config::ProviderModel {
                            name: String::new(),
                            model_id: String::new(),
                            description: String::new(),
                        });
                        let m_new = app.config.providers[*provider_idx].models.len();
                        *focus_idx = 3 * m_new - 3; // focus name of new model
                        
                        // Adjust scroll_offset to show new model at bottom
                        let popup_h = 18u16.min(app.term_rows);
                        let inner_h = popup_h.saturating_sub(2);
                        let chunks_0_h = inner_h.saturating_sub(2);
                        let max_visible_models = ((chunks_0_h / 2) as usize).max(1);
                        if m_new > max_visible_models {
                            *scroll_offset = m_new - max_visible_models;
                        } else {
                            *scroll_offset = 0;
                        }
                        
                        app.forms[SettingsTab::Providers as usize].dirty = true;
                    } else if *focus_idx == 3 * m + 1 {
                        // [Close]
                        app.active_popup = None;
                        return true;
                    }
                }
                ActivePopup::ConfigureSearch { provider_idx, focus_idx, edit_buffer, edit_cursor } => {
                    if *focus_idx == 0 {
                        let current = app.config.search.providers[*provider_idx].name.clone();
                        *edit_buffer = Some(current.clone());
                        *edit_cursor = current.len();
                    } else if *focus_idx == 1 {
                        let current = app.config.search.providers[*provider_idx].api_key.clone();
                        *edit_buffer = Some(current.clone());
                        *edit_cursor = current.len();
                    } else if *focus_idx == 2 {
                        let current = app.config.search.providers[*provider_idx].max_results.map(|x| x.to_string()).unwrap_or_default();
                        *edit_buffer = Some(current.clone());
                        *edit_cursor = current.len();
                    } else if *focus_idx == 3 {
                        // [Save & Close]
                        app.active_popup = None;
                        return true;
                    } else if *focus_idx == 4 {
                        // [Cancel]
                        app.active_popup = None;
                        return true;
                    }
                }
                ActivePopup::PlanApproval { .. } => {}
            }
        }
        _ => {}
    }

    // Keep focus visible for EditModels popup
    if let ActivePopup::EditModels { provider_idx, focus_idx, scroll_offset, .. } = &mut popup {
        let m = app.config.providers[*provider_idx].models.len();
        let popup_h = 18u16.min(app.term_rows);
        let inner_h = popup_h.saturating_sub(2);
        let chunks_0_h = inner_h.saturating_sub(2);
        let max_visible_models = ((chunks_0_h / 2) as usize).max(1);

        if *focus_idx < 3 * m {
            let focused_model = *focus_idx / 3;
            if focused_model < *scroll_offset {
                *scroll_offset = focused_model;
            } else if focused_model >= *scroll_offset + max_visible_models {
                *scroll_offset = focused_model + 1 - max_visible_models;
            }
        } else {
            if m > max_visible_models {
                *scroll_offset = m - max_visible_models;
            } else {
                *scroll_offset = 0;
            }
        }
    }

    app.active_popup = Some(popup);
    true
}
