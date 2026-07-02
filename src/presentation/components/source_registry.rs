use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::presentation::theme::{BORDER, SUCCESS, TEXT_DIM, TEXT_MAIN};

const SOURCES: &[(&str, bool)] = &[
    ("Web Search", true),
    ("Paper Search", true),
    ("Enterprise", false),
    ("Knowledge Layer", false),
];

pub fn render(f: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .title(" DATA SOURCE REGISTRY ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .split(inner);

    let lines: Vec<Line> = SOURCES
        .iter()
        .map(|(name, on)| {
            let box_style = if *on {
                Style::default().fg(SUCCESS)
            } else {
                Style::default().fg(TEXT_DIM)
            };
            let mark = if *on { "✓" } else { " " };
            Line::from(vec![
                Span::styled("[", box_style),
                Span::styled(mark, box_style),
                Span::styled("] ", box_style),
                Span::styled(*name, Style::default().fg(TEXT_MAIN)),
            ])
        })
        .collect();

    let para = Paragraph::new(lines).alignment(Alignment::Left);
    f.render_widget(para, chunks[0]);
}
