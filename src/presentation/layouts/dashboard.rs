use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::Block;
use crate::presentation::theme::BG_MAIN;
use crate::presentation::views::View;

pub fn render(f: &mut ratatui::Frame, area: Rect) {
    f.render_widget(
        Block::default().style(Style::default().bg(BG_MAIN)),
        area,
    );

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

    crate::presentation::components::header::render(f, header_area);
    crate::presentation::components::footer::render(f, footer_area, View::Dashboard);

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

    crate::presentation::components::session_list::render(f, sidebar_split[0]);
    crate::presentation::components::telemetry_panel::render(f, sidebar_split[1]);

    let main_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Min(6),
            Constraint::Min(0),
        ])
        .split(main_area);

    crate::presentation::components::query_input::render(f, main_split[0]);
    crate::presentation::components::pipeline_graph::render(f, main_split[1]);
    crate::presentation::components::clarifier_panel::render(f, main_split[2]);
    crate::presentation::components::source_registry::render(f, main_split[3]);
}
