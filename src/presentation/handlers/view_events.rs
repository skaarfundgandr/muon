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
            app.router.transition(View::Dashboard);
        } else {
            app.running = false;
        }
        return true;
    }

    if key.code == KeyCode::Enter && view == View::Welcome {
        app.router.transition(View::Dashboard);
        return true;
    }

    // Fallback: let the router handle f-keys, tab cycling
    app.router.handle_key(key)
}
