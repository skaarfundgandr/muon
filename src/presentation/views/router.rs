use crossterm::event::KeyEvent;

use super::View;

#[derive(Debug, Clone, Copy)]
pub struct ViewRouter {
    active_view: View,
}

impl Default for ViewRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewRouter {
    pub fn new() -> Self {
        Self {
            active_view: View::Welcome,
        }
    }

    pub fn active(&self) -> View {
        self.active_view
    }

    pub fn transition(&mut self, view: View) {
        self.active_view = view;
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match View::from_fkey(key.code) {
            Some(view) => {
                self.transition(view);
                true
            }
            None => false,
        }
    }
}
