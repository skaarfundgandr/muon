use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use crate::presentation::theme::{BORDER, SUCCESS, TEXT_DIM, TEXT_MAIN};

pub fn render(f: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .title(" DATA SOURCE REGISTRY ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(4)])
        .split(inner);

    let label = Line::from(Span::styled("Data Sources:", Style::default().fg(TEXT_DIM)));
    f.render_widget(Paragraph::new(label), chunks[0]);

    let row_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    let names = vec![
        Line::from(Span::styled("Web Search", Style::default().fg(TEXT_MAIN))),
        Line::from(Span::styled("Paper Search", Style::default().fg(TEXT_MAIN))),
        Line::from(Span::styled("Enterprise", Style::default().fg(TEXT_MAIN))),
        Line::from(Span::styled("Knowledge Layer", Style::default().fg(TEXT_MAIN))),
    ];

    let on_style = Style::default().fg(SUCCESS);
    let off_style = Style::default().fg(TEXT_DIM);

    let switches = vec![
        Line::from(Span::styled("[ON]", on_style)),
        Line::from(Span::styled("[ON]", on_style)),
        Line::from(Span::styled("[OFF]", off_style)),
        Line::from(Span::styled("[OFF]", off_style)),
    ];

    f.render_widget(Paragraph::new(names), row_chunks[0]);
    f.render_widget(Paragraph::new(switches).alignment(Alignment::Right), row_chunks[1]);
}
