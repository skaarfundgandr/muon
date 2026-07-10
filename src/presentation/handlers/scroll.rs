use crate::presentation::views::{SettingsTab, View};

use crate::presentation::state::AppState;
use crate::presentation::types::ActivePopup;

pub(crate) fn handle_scroll(app: &mut AppState, delta: i32) {
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

    if app.active_popup.is_some() {
        return;
    }

    let view = app.router.active();

    if view == View::Settings {
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
        return;
    }

    if view == View::Progress && app.clarifier_pending.is_none() {
        if delta > 0 {
            app.live_feed_scroll = app.live_feed_scroll.saturating_sub(1);
        } else {
            app.live_feed_scroll = app.live_feed_scroll.saturating_add(1);
        }
        return;
    }

    if view == View::Results {
        let sources_zone = app.term_cols.saturating_mul(40) / 100;
        let in_sources = app.mouse_col >= sources_zone;
        if in_sources {
            if delta > 0 {
                app.source_scroll = app.source_scroll.saturating_add(1);
            } else {
                app.source_scroll = app.source_scroll.saturating_sub(1);
            }
        } else if delta > 0 {
            app.report_scroll = app.report_scroll.saturating_add(1);
        } else {
            app.report_scroll = app.report_scroll.saturating_sub(1);
        }
    }
}

