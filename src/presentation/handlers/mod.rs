pub mod dashboard;
pub mod settings;
pub mod view_events;

use crossterm::event::KeyEvent;

use crate::app::AppState;
use crate::presentation::views::View;

pub fn handle_key(app: &mut AppState, key: KeyEvent) {
    let view = app.router.active();

    match view {
        View::Settings => {
            settings::handle(app, key);
        }
        View::Dashboard => {
            dashboard::handle(app, key);
        }
        _ => {
            view_events::handle(app, key);
        }
    }
}
