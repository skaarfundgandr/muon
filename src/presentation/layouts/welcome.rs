use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};

use crate::presentation::components::*;
use crate::presentation::theme::{BG_DARK, BG_MAIN, BORDER, TEXT_DIM};

pub fn render(f: &mut ratatui::Frame, area: Rect) {
    let bg = Block::default().style(Style::default().bg(BG_MAIN));
    f.render_widget(bg, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    welcome_logo::render(f, chunks[0]);

    let footer_block = Block::default()
        .style(Style::default().bg(BG_DARK))
        .border_style(Style::default().fg(BORDER));

    let footer = Paragraph::new(Line::from(ratatui::text::Span::styled(
        " Built with Rust • Powered by rig • Tokyo Night Theme ",
        Style::default().fg(TEXT_DIM).bg(BG_DARK),
    )))
    .style(Style::default().bg(BG_DARK))
    .block(footer_block);

    f.render_widget(footer, chunks[1]);
}
