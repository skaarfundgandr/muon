use crate::presentation::click::is_hovering;
use crate::presentation::theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(f: &mut ratatui::Frame, area: Rect, question: Option<&str>, response: &str, mouse_col: u16, mouse_row: u16) -> Option<Rect> {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme::border()))
        .title(" CLARIFIER ");

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(inner);

    let mut lines: Vec<Line> = Vec::new();

    match question {
        Some(q) => {
            lines.push(Line::from(vec![
                Span::styled("Status: ", Style::new().fg(theme::text_dim())),
                Span::styled(" ◉ Awaiting your input", Style::new().fg(theme::accent())),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Clarification required:",
                Style::new().fg(theme::text_dim()),
            )));
            lines.push(Line::from(vec![
                Span::styled("> ", Style::new().fg(theme::accent())),
                Span::styled(q, Style::new().fg(theme::text_main())),
            ]));
        }
        None => {
            lines.push(Line::from(vec![
                Span::styled("Status: ", Style::new().fg(theme::text_dim())),
                Span::styled(" ✓ 2 rounds complete", Style::new().fg(theme::success())),
                Span::styled("  Plan approved", Style::new().fg(theme::accent())),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Last clarification:",
                Style::new().fg(theme::text_dim()),
            )));
            lines.push(Line::from(vec![
                Span::styled("> ", Style::new().fg(theme::accent())),
                Span::styled("'Focus on Germany and Japan'", Style::new().fg(theme::text_main())),
            ]));
            lines.push(Line::from(""));
        }
    }

    let status = Paragraph::new(lines);
    f.render_widget(status, chunks[0]);

    let input_area = chunks[1];
    let input_hovered = is_hovering(input_area, mouse_col, mouse_row);
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(if input_hovered { theme::border_hover() } else { theme::border() }));

    let input_line = if question.is_some() {
        Line::from(vec![
            Span::styled("> ", Style::new().fg(theme::accent()).add_modifier(ratatui::style::Modifier::BOLD)),
            Span::styled(response.to_string(), Style::new().fg(theme::text_main())),
            Span::styled(
                "\u{258E}",
                Style::new().fg(theme::accent()).add_modifier(ratatui::style::Modifier::BOLD),
            ),
        ])
    } else {
        Line::from(Span::styled(
            "[Type response to clarify...]",
            Style::new().fg(theme::text_dim()),
        ))
    };

    let input_paragraph = Paragraph::new(input_line);

    f.render_widget(input_block.clone(), input_area);
    let input_inner = input_block.inner(input_area);
    f.render_widget(input_paragraph, input_inner);

    if question.is_some() {
        Some(input_area)
    } else {
        None
    }
}
