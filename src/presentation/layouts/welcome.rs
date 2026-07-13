use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};

use crate::presentation::components::*;
use crate::presentation::theme;

pub fn render(f: &mut ratatui::Frame, area: Rect, config: &crate::application::config::MuonConfig) {
    let bg = Block::default().style(Style::default().bg(theme::bg_main()));
    f.render_widget(bg, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    welcome_logo::render(f, chunks[0]);

    let footer_block = Block::default()
        .style(Style::default().bg(theme::bg_dark()))
        .border_style(Style::default().fg(theme::border()));

    let footer_text = format!(
        " Built with Rust • Powered by rig • {} Theme ",
        config.display.visual_theme
    );
    let footer = Paragraph::new(Line::from(ratatui::text::Span::styled(
        footer_text,
        Style::default().fg(theme::text_dim()).bg(theme::bg_dark()),
    )))
    .style(Style::default().bg(theme::bg_dark()))
    .block(footer_block);

    f.render_widget(footer, chunks[1]);
}
