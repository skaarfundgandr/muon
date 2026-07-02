use crossterm::event::KeyCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Welcome,
    Dashboard,
    Progress,
    Results,
    Settings,
}

impl View {
    pub fn title(&self) -> &'static str {
        match self {
            View::Welcome => "Welcome",
            View::Dashboard => "Dashboard",
            View::Progress => "Research Progress",
            View::Results => "Research Results",
            View::Settings => "Settings",
        }
    }

    pub fn from_fkey(key: KeyCode) -> Option<Self> {
        match key {
            KeyCode::F(1) => Some(View::Dashboard),
            KeyCode::F(2) => Some(View::Progress),
            KeyCode::F(3) => Some(View::Results),
            KeyCode::F(4) => Some(View::Settings),
            _ => None,
        }
    }
}
