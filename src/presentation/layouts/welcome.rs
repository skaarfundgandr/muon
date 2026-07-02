use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::Block;
use crate::presentation::theme::BG_MAIN;

pub fn render(f: &mut ratatui::Frame, area: Rect) {
    f.render_widget(
        Block::default().style(Style::default().bg(BG_MAIN)),
        area,
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    crate::presentation::components::header::render(f, chunks[0]);
    crate::presentation::components::welcome_logo::render(f, chunks[1]);
    crate::presentation::components::footer::render(
        f,
        chunks[2],
        crate::presentation::views::View::Welcome,
    );
}
