use crossterm::event::{KeyCode, KeyEvent};

use crate::app::AppState;

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
            }
            KeyCode::Esc => app.query_input.active = false,
            _ => {}
        }
    } else if app.clarifier_pending.is_some() {
        match key.code {
            KeyCode::Char(c) => app.clarifier_response.push(c),
            KeyCode::Backspace => {
                app.clarifier_response.pop();
            }
            KeyCode::Enter => {
                if let Some(pending) = app.clarifier_pending.take() {
                    let response = std::mem::take(&mut app.clarifier_response);
                    let _ = pending.responder.send(response);
                }
                app.clarifier_focused = false;
            }
            KeyCode::Esc => {
                app.clarifier_response.clear();
                if let Some(pending) = app.clarifier_pending.take() {
                    let _ = pending.responder.send(String::new());
                }
                app.clarifier_focused = false;
            }
            _ => {
                let _ = app.router.handle_key(key);
            }
        }
    } else {
        match key.code {
            KeyCode::Enter | KeyCode::Char('i') => {
                app.query_input.active = true;
            }
            KeyCode::Esc => {
                app.running = false;
            }
            _ => {
                let _ = app.router.handle_key(key);
            }
        }
    }
}
