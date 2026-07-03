use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::presentation::components::header::HeaderConfig;
use crate::presentation::components::*;
use crate::presentation::theme::{ACCENT, BG_MAIN, TEXT_DIM};
use crate::presentation::views::{SettingsTab, View};

pub fn render(f: &mut ratatui::Frame, area: Rect, tab: SettingsTab) {
    let bg = Block::default().style(Style::default().bg(BG_MAIN));
    f.render_widget(bg, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    header::render(f, chunks[0], HeaderConfig::for_view(View::Settings, 0));
    render_tab_bar(f, chunks[1], tab);

    match tab {
        SettingsTab::Agents => settings_agents::render(f, chunks[2]),
        SettingsTab::Tools => settings_tools::render(f, chunks[2]),
        SettingsTab::DataSources => settings_data_sources::render(f, chunks[2]),
        SettingsTab::Display => settings_display::render(f, chunks[2]),
        SettingsTab::Advanced => settings_advanced::render(f, chunks[2]),
    }

    footer::render(f, chunks[3], View::Settings);
}

fn render_tab_bar(f: &mut ratatui::Frame, area: Rect, active: SettingsTab) {
    let mut spans: Vec<Span> = Vec::new();
    for (i, tab) in SettingsTab::ALL.iter().enumerate() {
        let is_active = *tab == active;
        if is_active {
            spans.push(Span::styled(
                "[",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(
                tab.label(),
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(
                "]",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(tab.label(), Style::default().fg(TEXT_DIM)));
        }
        if i < SettingsTab::ALL.len() - 1 {
            spans.push(Span::raw("    "));
        }
    }
    f.render_widget(
        Paragraph::new(Line::from(spans)).block(Block::default()),
        area,
    );
}
