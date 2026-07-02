use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use crate::presentation::theme::{ACCENT, BORDER, SUCCESS, TEXT_DIM, TEXT_MAIN, WARNING};

pub fn render(f: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(BORDER));

    let filled = 1;
    let empty = 9;
    let filled_bar: String = "█".repeat(filled);
    let empty_bar: String = "░".repeat(empty);

    let line = Line::from(vec![
        Span::styled("Tokens: ", Style::default().fg(TEXT_DIM)),
        Span::styled("12,847 / 128k ", Style::default().fg(TEXT_MAIN)),
        Span::styled(filled_bar, Style::default().fg(ACCENT)),
        Span::styled(empty_bar, Style::default().fg(TEXT_DIM)),
        Span::styled(" 10%", Style::default().fg(TEXT_DIM)),
        Span::styled("  |  ", Style::default().fg(TEXT_DIM)),
        Span::styled("Agents: ", Style::default().fg(TEXT_DIM)),
        Span::styled("3 ", Style::default().fg(TEXT_MAIN)),
        Span::styled("  |  ", Style::default().fg(TEXT_DIM)),
        Span::styled("Round: ", Style::default().fg(TEXT_DIM)),
        Span::styled("1/2 ", Style::default().fg(WARNING).add_modifier(Modifier::BOLD)),
        Span::styled("  |  ", Style::default().fg(TEXT_DIM)),
        Span::styled("Cost: ", Style::default().fg(TEXT_DIM)),
        Span::styled("$0.034", Style::default().fg(SUCCESS).add_modifier(Modifier::BOLD)),
    ]);

    let paragraph = Paragraph::new(line);
    f.render_widget(block.clone(), area);
    f.render_widget(paragraph, block.inner(area));
}
