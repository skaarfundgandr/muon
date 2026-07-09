use crate::presentation::theme;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(
    f: &mut ratatui::Frame,
    area: Rect,
    sessions_count: usize,
    tokens_in: u64,
    tokens_out: u64,
) {
    let block = Block::default()
        .title(" AGENT TELEMETRY ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::border()));

    let total_tokens = tokens_in.saturating_add(tokens_out);
    let sessions_str = sessions_count.to_string();
    let tokens_str = if total_tokens == 0 && tokens_in == 0 && tokens_out == 0 {
        "0".to_string()
    } else {
        format!("{total_tokens} ({tokens_in}↑/{tokens_out}↓)")
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("Sessions:      ", Style::default().fg(theme::text_dim())),
            Span::styled(sessions_str, Style::default().fg(theme::text_main())),
        ]),
        Line::from(vec![
            Span::styled("Total Tokens:  ", Style::default().fg(theme::text_dim())),
            Span::styled(tokens_str, Style::default().fg(theme::text_main())),
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
