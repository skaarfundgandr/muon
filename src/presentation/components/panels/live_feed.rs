use crate::presentation::theme::{
    ACCENT, BORDER, CYAN, PURPLE, SUCCESS, TEXT_DIM, TEXT_MAIN, WARNING,
};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(f: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .title(" >_ LIVE FEED ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER));

    let dim = Style::default().fg(TEXT_DIM);
    let main = Style::default().fg(TEXT_MAIN);

    let lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("[14:23:01] ", dim),
            Span::styled("[INTENT] ", Style::default().fg(PURPLE)),
            Span::styled("research → deep", main),
        ]),
        Line::from(vec![
            Span::styled("[14:23:02] ", dim),
            Span::styled("[CLARIFY] ", Style::default().fg(CYAN)),
            Span::styled("'Focus on Germany or all G7?'", main),
        ]),
        Line::from(vec![
            Span::styled("[14:23:05] ", dim),
            Span::styled("[CLARIFY] ", Style::default().fg(CYAN)),
            Span::styled("User: 'Germany and Japan'", main),
        ]),
        Line::from(vec![
            Span::styled("[14:23:08] ", dim),
            Span::styled("[PLAN] ", Style::default().fg(ACCENT)),
            Span::styled("4-section outline created", main),
        ]),
        Line::from(vec![
            Span::styled("[14:23:10] ", dim),
            Span::styled("[PLAN] ", Style::default().fg(ACCENT)),
            Span::styled("Plan approved", main),
        ]),
        Line::from(vec![
            Span::styled("[14:23:12] ", dim),
            Span::styled("[SEARCH] ", Style::default().fg(SUCCESS)),
            Span::styled("'renewable energy GDP Germany'", main),
        ]),
        Line::from(vec![
            Span::styled("[14:23:15] ", dim),
            Span::styled("[SEARCH] ", Style::default().fg(SUCCESS)),
            Span::styled("12 results, filtering > 0.7", main),
        ]),
        Line::from(vec![
            Span::styled("[14:23:18] ", dim),
            Span::styled("[EXTRACT] ", Style::default().fg(WARNING)),
            Span::styled("IEA-Germany-Report.pdf — 3 key data points", main),
        ]),
        Line::from(vec![
            Span::styled("[14:23:22] ", dim),
            Span::styled("[ORCHESTRATE] ", Style::default().fg(ACCENT)),
            Span::styled("Assigning Researcher Round 2 queries", main),
        ]),
        Line::from(vec![
            Span::styled("[14:23:25] ", dim),
            Span::styled("[VERIFY] ", Style::default().fg(SUCCESS)),
            Span::styled("Citation [3] exact match confirmed", main),
        ]),
        Line::from(vec![
            Span::styled("[14:23:28] ", dim),
            Span::styled("[SYS] ", Style::default().fg(TEXT_DIM)),
            Span::styled("Allocating thread to Agent-Researcher-2", main),
        ]),
    ];

    let paragraph = Paragraph::new(lines);
    f.render_widget(block.clone(), area);
    f.render_widget(paragraph, block.inner(area));
}
