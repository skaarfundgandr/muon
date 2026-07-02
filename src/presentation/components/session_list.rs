use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use crate::presentation::theme::{BORDER, SUCCESS, TEXT_DIM, TEXT_MAIN};

pub fn render(f: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .title(" RECENT SESSIONS ")
        .borders(Borders::ALL)
        .border_style(Style::new().fg(BORDER));

    let mut lines: Vec<Line> = Vec::new();

    let entries = [
        ("●", "Economic Impacts G7", "2m ago", "Economic impacts of renewable energy transition...", true),
        ("●", "Quantum Computing", "1h ago", "Quantum entanglement and future computing paradigms", false),
        ("●", "CRISPR Ethics 2026", "Yesterday", "Ethical considerations in CRISPR gene editing", false),
        ("●", "Solid State Battery", "3d ago", "Advances in solid-state battery technology", false),
    ];

    for (i, (dot, title, time, desc, is_active)) in entries.iter().enumerate() {
        let dot_style = if *is_active {
            Style::new().fg(SUCCESS).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(TEXT_DIM)
        };

        lines.push(Line::from(vec![
            Span::styled(*dot, dot_style),
            Span::styled(format!(" {} ", title), Style::new().fg(TEXT_MAIN)),
            Span::styled(*time, Style::new().fg(TEXT_DIM)),
        ]));

        lines.push(Line::from(vec![
            Span::styled(format!("   {}", desc), Style::new().fg(TEXT_DIM)),
        ]));

        if i < entries.len() - 1 {
            lines.push(Line::from(""));
        }
    }

    let paragraph = Paragraph::new(lines);
    f.render_widget(block.clone(), area);
    f.render_widget(paragraph, block.inner(area));
}
