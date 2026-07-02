use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use crate::presentation::theme::{ACCENT, BORDER, SUCCESS, TEXT_DIM, TEXT_MAIN};

pub fn render(f: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(BORDER))
        .title(" CLARIFIER ");

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(inner);

    let lines = vec![
        Line::from(vec![
            Span::styled("Status: ", Style::new().fg(TEXT_DIM)),
            Span::styled(" ✓ 2 rounds complete", Style::new().fg(SUCCESS)),
            Span::styled("  Plan approved", Style::new().fg(ACCENT)),
        ]),
        Line::from(""),
        Line::from(Span::styled("Last clarification:", Style::new().fg(TEXT_DIM))),
        Line::from(vec![
            Span::styled("> ", Style::new().fg(ACCENT)),
            Span::styled("'Focus on Germany and Japan'", Style::new().fg(TEXT_MAIN)),
        ]),
        Line::from(""),
    ];

    let status = Paragraph::new(lines);
    f.render_widget(status, chunks[0]);

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(BORDER));

    let input_paragraph = Paragraph::new(Line::from(
        Span::styled("[Type response to clarify...]", Style::new().fg(TEXT_DIM)),
    ));

    let input_area = chunks[1];
    f.render_widget(input_block.clone(), input_area);
    let input_inner = input_block.inner(input_area);
    f.render_widget(input_paragraph, input_inner);
}
