use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::config::MuonConfig;
use crate::presentation::click::{is_hovering, ClickAction, ClickTarget};
use crate::presentation::components::header::HeaderConfig;
use crate::presentation::components::*;
use crate::presentation::components::settings::{advanced, agents, data_sources, display, providers, tools};
use crate::presentation::components::settings::dropdown_overlay::PendingDropdown;
use crate::presentation::form::FormState;
use crate::presentation::theme;
use crate::presentation::views::{SettingsTab, View};

#[allow(clippy::too_many_arguments)]
pub fn render(
    f: &mut ratatui::Frame,
    area: Rect,
    tab: SettingsTab,
    config: &MuonConfig,
    _form: &FormState,
    hit_registry: &mut Vec<ClickTarget>,
    mouse_col: u16,
    mouse_row: u16,
) {
    let bg = Block::default().style(Style::default().bg(theme::bg_main()));
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
    render_tab_bar(f, chunks[1], tab, hit_registry, mouse_col, mouse_row);

    let mut pending_dropdown: Option<PendingDropdown> = None;

    match tab {
        SettingsTab::Providers => {
            providers::render(f, chunks[2], config, _form, hit_registry, mouse_col, mouse_row, &mut pending_dropdown);
        }
        SettingsTab::Agents => {
            agents::render(f, chunks[2], config, _form, hit_registry, mouse_col, mouse_row, &mut pending_dropdown);
        }
        SettingsTab::Tools => {
            tools::render(f, chunks[2], config, _form, hit_registry, mouse_col, mouse_row, &mut pending_dropdown);
        }
        SettingsTab::DataSources => {
            data_sources::render(f, chunks[2], config, _form, hit_registry, mouse_col, mouse_row, &mut pending_dropdown);
        }
        SettingsTab::Display => {
            display::render(f, chunks[2], &config.display, _form, hit_registry, mouse_col, mouse_row, &mut pending_dropdown);
        }
        SettingsTab::Advanced => {
            advanced::render(f, chunks[2], &config.advanced, _form, hit_registry, mouse_col, mouse_row, &mut pending_dropdown);
        }
    }

    footer::render(f, chunks[3], View::Settings, hit_registry, mouse_col, mouse_row);

    if let Some(pending) = pending_dropdown {
        crate::presentation::components::settings::dropdown_overlay::render_dropdown_overlay(
            f,
            pending.below,
            &pending.field_label,
            &pending.options,
            _form,
            hit_registry,
            mouse_col,
            mouse_row,
        );
    }
}

fn render_tab_bar(f: &mut ratatui::Frame, area: Rect, active: SettingsTab, hit_registry: &mut Vec<ClickTarget>, mouse_col: u16, mouse_row: u16) {
    let mut spans: Vec<Span> = Vec::new();
    let mut col = area.x;
    let tabs = SettingsTab::ALL;

    for (i, tab) in tabs.iter().enumerate() {
        let is_active = *tab == active;
        let label = tab.label();

        // Compute this tab's rendered width (label + optional brackets + separator)
        let rendered_width = if is_active {
            // "[LABEL]" + 4-space separator (except last)
            (label.len() + 2 + if i < tabs.len() - 1 { 4 } else { 0 }) as u16
        } else {
            // "LABEL" + 4-space separator (except last)
            (label.len() + if i < tabs.len() - 1 { 4 } else { 0 }) as u16
        };

        let seg_rect = Rect::new(col, area.y, rendered_width, area.height);
        hit_registry.push(ClickTarget {
            rect: seg_rect,
            action: ClickAction::SwitchSettingsTab(*tab),
        });

        let hovered = is_hovering(seg_rect, mouse_col, mouse_row);
        let label_color = if is_active {
            theme::accent()
        } else if hovered {
            theme::border_hover()
        } else {
            theme::text_dim()
        };
        let label_style = if is_active {
            Style::default().fg(label_color).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(label_color)
        };

        if is_active {
            spans.push(Span::styled("[", label_style));
            spans.push(Span::styled(label, label_style));
            spans.push(Span::styled("]", label_style));
        } else {
            spans.push(Span::styled(label, label_style));
        }
        if i < tabs.len() - 1 {
            spans.push(Span::raw("    "));
        }

        col = col.saturating_add(rendered_width);
    }

    // Remove unused import for per_segment — we don't use the labels vec anymore either
    f.render_widget(
        Paragraph::new(Line::from(spans)).block(Block::default()),
        area,
    );
}
