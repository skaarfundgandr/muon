use crossterm::event::{KeyCode, KeyEvent};

use crate::app::AppState;
use crate::presentation::views::View;

pub fn handle(app: &mut AppState, key: KeyEvent) {
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
                    app.spawn_pipeline(&query);
                }
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
}
