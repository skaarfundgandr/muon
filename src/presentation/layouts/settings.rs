use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::Block;

use crate::presentation::components::header::HeaderConfig;
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
            Constraint::Length(1),
        ])
        .split(area);

    crate::presentation::components::header::render(
        f,
        chunks[0],
        HeaderConfig::for_view(View::Settings, 0),
    );
    crate::presentation::components::settings_form::render(f, chunks[1]);
    crate::presentation::components::footer::render(f, chunks[2], View::Settings);
}
