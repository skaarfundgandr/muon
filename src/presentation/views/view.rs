use crossterm::event::KeyCode;
use ratatui::layout::Rect;

use crate::application::pipeline::PipelineState;
use crate::config::MuonConfig;
use crate::presentation::click::ClickTarget;
use crate::presentation::components::query_input::QueryInput;
use crate::presentation::form::FormState;
use crate::session::SessionSummary;

use super::SettingsTab;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Welcome,
    Dashboard,
    Progress,
    Results,
    Settings,
}

/// Data needed by View::render(). Constructed once per frame in app.rs.
pub struct RenderParams<'a> {
    pub query_input: &'a QueryInput,
    pub sessions: &'a [SessionSummary],
    pub pipeline: &'a PipelineState,
    pub config: &'a MuonConfig,
    pub forms: &'a [FormState; 6],
    pub settings_tab: SettingsTab,
    pub hit_registry: &'a mut Vec<ClickTarget>,
    pub mouse_col: u16,
    pub mouse_row: u16,
    pub clarifier_question: Option<&'a str>,
    pub clarifier_response: &'a str,
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

    /// Dispatch rendering to the appropriate layout.
    pub fn render(
        self,
        f: &mut ratatui::Frame,
        area: Rect,
        params: &mut RenderParams,
    ) {
        match self {
            View::Welcome => {
                crate::presentation::layouts::welcome::render(f, area);
            }
            View::Dashboard => {
                crate::presentation::layouts::dashboard::render(
                    f,
                    area,
                    params.query_input,
                    params.sessions,
                    params.pipeline,
                    params.hit_registry,
                    params.mouse_col,
                    params.mouse_row,
                    params.clarifier_question,
                    params.clarifier_response,
                );
            }
            View::Progress => {
                crate::presentation::layouts::progress::render(f, area, params.pipeline, params.hit_registry, params.mouse_col, params.mouse_row);
            }
            View::Results => {
                crate::presentation::layouts::results::render(f, area, params.pipeline, params.hit_registry, params.mouse_col, params.mouse_row);
            }
            View::Settings => {
                let tab = params.settings_tab;
                let form = &params.forms[tab as usize];
                crate::presentation::layouts::settings::render(f, area, tab, params.config, form, params.hit_registry, params.mouse_col, params.mouse_row);
            }
        }
    }
}
