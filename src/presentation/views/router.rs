use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::View;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum SettingsTab {
    Providers,
    Agents,
    Tools,
    DataSources,
    Display,
    Advanced,
}

impl SettingsTab {
    pub const ALL: [SettingsTab; 6] = [
        SettingsTab::Providers,
        SettingsTab::Agents,
        SettingsTab::Tools,
        SettingsTab::DataSources,
        SettingsTab::Display,
        SettingsTab::Advanced,
    ];

    pub fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|t| *t == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    pub fn prev(self) -> Self {
        let idx = Self::ALL.iter().position(|t| *t == self).unwrap_or(0);
        Self::ALL[(idx + Self::ALL.len() - 1) % Self::ALL.len()]
    }

    pub fn label(&self) -> &'static str {
        match self {
            SettingsTab::Providers => "Providers",
            SettingsTab::Agents => "Agents",
            SettingsTab::Tools => "Tools",
            SettingsTab::DataSources => "Data Sources",
            SettingsTab::Display => "Display",
            SettingsTab::Advanced => "Advanced",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ViewRouter {
    active_view: View,
    settings_tab: SettingsTab,
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
            settings_tab: SettingsTab::Providers,
        }
    }

    pub fn active(&self) -> View {
        self.active_view
    }

    pub fn settings_tab(&self) -> SettingsTab {
        self.settings_tab
    }

    pub fn transition(&mut self, view: View) {
        self.active_view = view;
    }

    pub fn set_settings_tab(&mut self, tab: SettingsTab) {
        self.settings_tab = tab;
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.code == KeyCode::Tab && !key.modifiers.contains(KeyModifiers::SHIFT) {
            let next = self.active_view.next();
            self.transition(next);
            return true;
        }
        match View::from_fkey(key.code) {
            Some(view) => {
                self.transition(view);
                true
            }
            None => false,
        }
    }
}
