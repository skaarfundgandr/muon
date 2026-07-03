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
            KeyCode::Char('1') => Some(View::Dashboard),
            KeyCode::Char('2') => Some(View::Progress),
            KeyCode::Char('3') => Some(View::Results),
            KeyCode::Char('4') => Some(View::Settings),
            _ => None,
        }
    }

    pub fn next(self) -> Self {
        match self {
            View::Welcome => View::Dashboard,
            View::Dashboard => View::Progress,
            View::Progress => View::Results,
            View::Results => View::Settings,
            View::Settings => View::Dashboard,
        }
    }
}
