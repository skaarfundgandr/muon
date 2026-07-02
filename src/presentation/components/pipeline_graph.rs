use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use crate::presentation::theme::{ACCENT, BORDER, SUCCESS, TEXT_DIM, TEXT_MAIN};

pub fn render(f: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(BORDER))
        .title(Span::styled(
            " PIPELINE ROUTING ",
            Style::new()
                .fg(TEXT_MAIN)
                .add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Length(3),
            Constraint::Percentage(33),
            Constraint::Length(3),
            Constraint::Percentage(33),
        ])
        .split(inner);

    let nodes: [(&str, &str, Style); 3] = [
        (
            "Intent Classifier",
            "✓ Complete",
            Style::new().fg(SUCCESS),
        ),
        (
            "Research Pipeline",
            "◉ Active",
            Style::new().fg(ACCENT),
        ),
        (
            "Deep Researcher",
            "○ Pending",
            Style::new().fg(TEXT_DIM),
        ),
    ];

    for (i, (title, status, status_style)) in nodes.iter().enumerate() {
        let node_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::new().fg(BORDER));

        let paragraph = Paragraph::new(vec![
            Line::from(Span::styled(
                *title,
                Style::new()
                    .fg(TEXT_MAIN)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(*status, *status_style)),
        ])
        .alignment(Alignment::Center);

        f.render_widget(node_block.clone(), chunks[i * 2]);
        f.render_widget(paragraph, node_block.inner(chunks[i * 2]));
    }

    for i in [1, 3] {
        let arrow = Paragraph::new(Line::from(Span::styled(
            " → ",
            Style::new().fg(ACCENT),
        )))
        .alignment(Alignment::Center);
        f.render_widget(arrow, chunks[i]);
    }
}
