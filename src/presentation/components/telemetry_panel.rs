use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use crate::presentation::theme::{ACCENT, BORDER, SUCCESS, TEXT_DIM, TEXT_MAIN};

pub fn render(f: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .title(" AGENT TELEMETRY ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER));

    let lines = vec![
        Line::from(vec![
            Span::styled("Active Router: ", Style::default().fg(TEXT_DIM)),
            Span::styled("glm-5.2", Style::default().fg(ACCENT)),
        ]),
        Line::from(vec![
            Span::styled("Session Tokens: ", Style::default().fg(TEXT_DIM)),
            Span::styled("12,847", Style::default().fg(TEXT_MAIN)),
        ]),
        Line::from(vec![
            Span::styled("Est. Cost: ", Style::default().fg(TEXT_DIM)),
            Span::styled("$0.034", Style::default().fg(SUCCESS)),
        ]),
        Line::from(vec![
            Span::styled("Framework: ", Style::default().fg(TEXT_DIM)),
            Span::styled("rig v0.38", Style::default().fg(TEXT_MAIN)),
        ]),
    ];

    let paragraph = Paragraph::new(lines);
    f.render_widget(block.clone(), area);
    f.render_widget(paragraph, block.inner(area));
}
