use crate::presentation::theme;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(f: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .title(" >_ LIVE FEED ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::border()));

    let dim = Style::default().fg(theme::text_dim());
    let main = Style::default().fg(theme::text_main());

    let lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("[14:23:01] ", dim),
            Span::styled("[INTENT] ", Style::default().fg(theme::purple())),
            Span::styled("research → deep", main),
        ]),
        Line::from(vec![
            Span::styled("[14:23:02] ", dim),
            Span::styled("[CLARIFY] ", Style::default().fg(theme::cyan())),
            Span::styled("'Focus on Germany or all G7?'", main),
        ]),
        Line::from(vec![
            Span::styled("[14:23:05] ", dim),
            Span::styled("[CLARIFY] ", Style::default().fg(theme::cyan())),
            Span::styled("User: 'Germany and Japan'", main),
        ]),
        Line::from(vec![
            Span::styled("[14:23:08] ", dim),
            Span::styled("[PLAN] ", Style::default().fg(theme::accent())),
            Span::styled("4-section outline created", main),
        ]),
        Line::from(vec![
            Span::styled("[14:23:10] ", dim),
            Span::styled("[PLAN] ", Style::default().fg(theme::accent())),
            Span::styled("Plan approved", main),
        ]),
        Line::from(vec![
            Span::styled("[14:23:12] ", dim),
            Span::styled("[SEARCH] ", Style::default().fg(theme::success())),
            Span::styled("'renewable energy GDP Germany'", main),
        ]),
        Line::from(vec![
            Span::styled("[14:23:15] ", dim),
            Span::styled("[SEARCH] ", Style::default().fg(theme::success())),
            Span::styled("12 results, filtering > 0.7", main),
        ]),
        Line::from(vec![
            Span::styled("[14:23:18] ", dim),
            Span::styled("[EXTRACT] ", Style::default().fg(theme::warning())),
            Span::styled("IEA-Germany-Report.pdf — 3 key data points", main),
        ]),
        Line::from(vec![
            Span::styled("[14:23:22] ", dim),
            Span::styled("[ORCHESTRATE] ", Style::default().fg(theme::accent())),
            Span::styled("Assigning Researcher Round 2 queries", main),
        ]),
        Line::from(vec![
            Span::styled("[14:23:25] ", dim),
            Span::styled("[VERIFY] ", Style::default().fg(theme::success())),
            Span::styled("Citation [3] exact match confirmed", main),
        ]),
        Line::from(vec![
            Span::styled("[14:23:28] ", dim),
            Span::styled("[SYS] ", Style::default().fg(theme::text_dim())),
            Span::styled("Allocating thread to Agent-Researcher-2", main),
        ]),
    ];

    let paragraph = Paragraph::new(lines);
    f.render_widget(block.clone(), area);
    f.render_widget(paragraph, block.inner(area));
}
