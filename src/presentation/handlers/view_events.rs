use crossterm::event::{KeyCode, KeyEvent};

use crate::presentation::AppState;
use crate::presentation::views::View;

/// Handle keys for non-settings, non-dashboard views (Welcome, Progress, Results).
/// Returns true if the key was consumed.
pub fn handle(app: &mut AppState, key: KeyEvent) -> bool {
    let view = app.router.active();

    if key.code == KeyCode::Esc {
        if view == View::Progress && app.is_pipeline_busy() {
            app.abort_pipeline();
            return true;
        }
        app.running = false;
        return true;
    }

    if key.code == KeyCode::Enter && view == View::Welcome {
        app.router.transition(View::Dashboard);
        return true;
    }

    if view == View::Progress && app.active_popup.is_none() && app.clarifier_pending.is_none() {
        match key.code {
            KeyCode::Up => {
                app.live_feed_scroll = app.live_feed_scroll.saturating_add(1);
                return true;
            }
            KeyCode::Down => {
                app.live_feed_scroll = app.live_feed_scroll.saturating_sub(1);
                return true;
            }
            KeyCode::PageUp => {
                app.live_feed_scroll = app.live_feed_scroll.saturating_add(5);
                return true;
            }
            KeyCode::PageDown => {
                app.live_feed_scroll = app.live_feed_scroll.saturating_sub(5);
                return true;
            }
            _ => {}
        }
    }

    if view == View::Results {
        match key.code {
            KeyCode::Up => {
                app.report_scroll = app.report_scroll.saturating_add(1);
                return true;
            }
            KeyCode::Down => {
                app.report_scroll = app.report_scroll.saturating_sub(1);
                return true;
            }
            KeyCode::PageUp => {
                app.report_scroll = app.report_scroll.saturating_add(5);
                return true;
            }
            KeyCode::PageDown => {
                app.report_scroll = app.report_scroll.saturating_sub(5);
                return true;
            }
            KeyCode::Left => {
                app.source_scroll = app.source_scroll.saturating_add(1);
                return true;
            }
            KeyCode::Right => {
                app.source_scroll = app.source_scroll.saturating_sub(1);
                return true;
            }
            KeyCode::F(1) => {
                app.action_export_markdown();
                return true;
            }
            KeyCode::F(2) => {
                app.action_export_pdf();
                return true;
            }
            KeyCode::F(3) => {
                app.action_sync_obsidian();
                return true;
            }
            KeyCode::F(4) => {
                app.action_new_query();
                return true;
            }
            KeyCode::F(5) => {
                app.action_refine_search();
                return true;
            }
            KeyCode::F(6) => {
                app.action_toggle_full_report();
                return true;
            }
            _ => {}
        }
    }

    // Fallback: let the router handle f-keys, tab cycling
    app.router.handle_key(key)
}
