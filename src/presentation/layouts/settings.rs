use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::config::MuonConfig;
use crate::presentation::click::{ClickAction, ClickTarget};
use crate::presentation::components::header::HeaderConfig;
use crate::presentation::components::*;
use crate::presentation::components::settings::{advanced, agents, data_sources, display, tools};
use crate::presentation::form::FormState;
use crate::presentation::theme::{ACCENT, BG_MAIN, TEXT_DIM};
use crate::presentation::views::{SettingsTab, View};

pub fn render(
    f: &mut ratatui::Frame,
    area: Rect,
    tab: SettingsTab,
    config: &MuonConfig,
    _form: &FormState,
    hit_registry: &mut Vec<ClickTarget>,
) {
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

    header::render(f, chunks[0], HeaderConfig::for_settings(0, _form.dirty));
    render_tab_bar(f, chunks[1], tab, hit_registry);

    match tab {
        SettingsTab::Agents => {
            agents::render(f, chunks[2], &config.agents, _form, hit_registry);
        }
        SettingsTab::Tools => {
            tools::render(f, chunks[2], &config.tools, _form, hit_registry);
        }
        SettingsTab::DataSources => {
            data_sources::render(f, chunks[2], &config.data_sources, _form, hit_registry);
        }
        SettingsTab::Display => {
            display::render(f, chunks[2], &config.display, _form, hit_registry);
        }
        SettingsTab::Advanced => {
            advanced::render(f, chunks[2], &config.advanced, _form, hit_registry);
        }
    }

    footer::render(f, chunks[3], View::Settings, hit_registry);
}

fn render_tab_bar(f: &mut ratatui::Frame, area: Rect, active: SettingsTab, hit_registry: &mut Vec<ClickTarget>) {
    let labels: Vec<&'static str> = SettingsTab::ALL.iter().map(|t| t.label()).collect();
    let per_segment = (area.width as usize).max(1) / labels.len().max(1);
    for (i, tab) in SettingsTab::ALL.iter().enumerate() {
        let seg_x = area.x.saturating_add((i as u16) * (per_segment as u16));
        let seg_w = per_segment as u16;
        let seg_rect = Rect::new(seg_x, area.y, seg_w, area.height);
        hit_registry.push(ClickTarget {
            rect: seg_rect,
            action: ClickAction::SwitchSettingsTab(*tab),
        });
    }
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
