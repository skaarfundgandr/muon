use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use crate::presentation::theme::{BG_DARK, BG_MAIN, TEXT_DIM};

pub fn render(f: &mut ratatui::Frame, area: Rect) {
    let bg = ratatui::widgets::Block::default().style(Style::default().bg(BG_MAIN));
    f.render_widget(bg, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    crate::presentation::components::welcome_logo::render(f, chunks[0]);

    let footer = Paragraph::new(Line::from(ratatui::text::Span::styled(
        " Built with Rust • Powered by rig • Tokyo Night Theme ",
        Style::default()
            .fg(TEXT_DIM)
            .bg(BG_DARK)
            .add_modifier(Modifier::BOLD),
    )))
    .style(Style::default().bg(BG_DARK));

    f.render_widget(footer, chunks[1]);
}
