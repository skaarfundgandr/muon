use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::Block;
use crate::presentation::theme::BG_MAIN;
use crate::presentation::views::View;

pub fn render(f: &mut ratatui::Frame, area: Rect) {
    let bg = Block::default().style(Style::default().bg(BG_MAIN));
    f.render_widget(bg, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(2),
            Constraint::Length(1),
        ])
        .split(area);

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[1]);

    crate::presentation::components::header::render(f, chunks[0]);
    crate::presentation::components::pipeline_graph::render(f, body_chunks[0]);
    crate::presentation::components::live_feed::render(f, body_chunks[1]);
    crate::presentation::components::resource_monitor::render(f, chunks[2]);
    crate::presentation::components::footer::render(f, chunks[3], View::Progress);
}
