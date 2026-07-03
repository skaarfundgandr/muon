use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::Block;

use crate::presentation::components::header::HeaderConfig;
use crate::presentation::components::*;
use crate::presentation::theme::BG_MAIN;
use crate::presentation::views::View;
use crate::session::SessionSummary;
use crate::presentation::components::query_input::QueryInput;

pub fn render(
    f: &mut ratatui::Frame,
    area: Rect,
    query: &QueryInput,
    sessions: &[SessionSummary],
) {
    f.render_widget(Block::default().style(Style::default().bg(BG_MAIN)), area);

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let header_area = vertical[0];
    let body_area = vertical[1];
    let footer_area = vertical[2];

    header::render(f, header_area, HeaderConfig::for_view(View::Dashboard, 0));
    footer::render(f, footer_area, View::Dashboard);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(body_area);

    let sidebar_area = horizontal[0];
    let main_area = horizontal[1];

    let sidebar_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(sidebar_area);

    session_list::render(f, sidebar_split[0], sessions);
    telemetry_panel::render(f, sidebar_split[1]);

    let main_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(8),
            Constraint::Min(3),
        ])
        .split(main_area);

    query_input::render(f, main_split[0], query);
    pipeline_graph::render_horizontal(f, main_split[1], Some(clarifier_panel::render));
    source_registry::render(f, main_split[2]);
}
