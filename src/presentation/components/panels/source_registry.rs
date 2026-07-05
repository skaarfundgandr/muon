use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::presentation::theme;

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
        .border_style(Style::default().fg(theme::border()));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(inner);

    for (i, (name, on)) in SOURCES.iter().enumerate() {
        let dot = if *on { "●" } else { "○" };
        let color = if *on { theme::success() } else { theme::text_dim() };
        let line = Line::from(vec![
            Span::styled(*name, Style::default().fg(theme::text_main())),
            Span::raw(" "),
            Span::styled(dot, Style::default().fg(color)),
        ]);
        f.render_widget(Paragraph::new(line), chunks[i]);
    }
}
