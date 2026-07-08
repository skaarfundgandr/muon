use crossterm::event::{KeyCode, KeyEvent};

use crate::app::AppState;
use crate::presentation::views::View;

/// Handle keys for non-settings, non-dashboard views (Welcome, Progress, Results).
/// Returns true if the key was consumed.
pub fn handle(app: &mut AppState, key: KeyEvent) -> bool {
    let view = app.router.active();

    if key.code == KeyCode::Esc {
        if view == View::Progress {
            app.pipeline.cancel();
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

    // Fallback: let the router handle f-keys, tab cycling
    app.router.handle_key(key)
}
