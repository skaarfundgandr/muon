use crate::presentation::theme;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(f: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .title(" AGENT TELEMETRY ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::border()));

    let lines = vec![
        Line::from(vec![
            Span::styled("Sessions:      ", Style::default().fg(theme::text_dim())),
            Span::styled("—", Style::default().fg(theme::text_main())),
        ]),
        Line::from(vec![
            Span::styled("Total Tokens:  ", Style::default().fg(theme::text_dim())),
            Span::styled("—", Style::default().fg(theme::text_main())),
        ]),
        Line::from(vec![
            Span::styled("Est. Cost:     ", Style::default().fg(theme::text_dim())),
            Span::styled("—", Style::default().fg(theme::text_dim())),
        ]),
        Line::from(vec![
            Span::styled("Framework:     ", Style::default().fg(theme::text_dim())),
            Span::styled("rig 0.39", Style::default().fg(theme::text_main())),
        ]),
    ];

    let paragraph = Paragraph::new(lines);
    f.render_widget(block.clone(), area);
    f.render_widget(paragraph, block.inner(area));
}
