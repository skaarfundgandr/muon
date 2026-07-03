use crate::presentation::theme::{BORDER, SUCCESS, TEXT_DIM, TEXT_MAIN};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(f: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .title(" AGENT TELEMETRY ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER));

    let lines = vec![
        Line::from(vec![
            Span::styled("Sessions:      ", Style::default().fg(TEXT_DIM)),
            Span::styled("12", Style::default().fg(TEXT_MAIN)),
        ]),
        Line::from(vec![
            Span::styled("Total Tokens:  ", Style::default().fg(TEXT_DIM)),
            Span::styled("847,291", Style::default().fg(TEXT_MAIN)),
        ]),
        Line::from(vec![
            Span::styled("Est. Cost:     ", Style::default().fg(TEXT_DIM)),
            Span::styled("$1.84", Style::default().fg(SUCCESS)),
        ]),
        Line::from(vec![
            Span::styled("Framework:     ", Style::default().fg(TEXT_DIM)),
            Span::styled("rig 0.39", Style::default().fg(TEXT_MAIN)),
        ]),
    ];

    let paragraph = Paragraph::new(lines);
    f.render_widget(block.clone(), area);
    f.render_widget(paragraph, block.inner(area));
}
