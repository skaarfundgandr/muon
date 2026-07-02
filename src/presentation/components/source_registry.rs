use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::presentation::theme::{BORDER, SUCCESS, TEXT_DARK, TEXT_DIM, TEXT_MAIN};

pub fn render(f: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .title(" DATA SOURCE REGISTRY ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(16), Constraint::Min(0)])
        .split(inner);

    let label = Line::from(Span::styled("Data Sources:", Style::default().fg(TEXT_DIM)));
    f.render_widget(Paragraph::new(label), chunks[0]);

    let sources = [
        ("Web Search", true),
        ("Paper Search", true),
        ("Enterprise", false),
        ("Knowledge Layer", false),
    ];

    let per = (chunks[1].width / sources.len() as u16).max(20);

    for (i, (name, on)) in sources.iter().enumerate() {
        let x = chunks[1].x + (i as u16) * per;
        let cell = Rect {
            x,
            y: chunks[1].y,
            width: per,
            height: chunks[1].height,
        };

        let pill_track_color = if *on { SUCCESS } else { TEXT_DARK };
        let knob_char = if *on { "●" } else { "○" };

        let pill_track = "████";
        let line = Line::from(vec![
            Span::styled(*name, Style::default().fg(TEXT_MAIN)),
            Span::raw("  "),
            Span::styled(pill_track, Style::default().fg(pill_track_color)),
            Span::styled(" ", Style::default().fg(pill_track_color)),
            Span::styled(knob_char, Style::default().fg(TEXT_MAIN)),
        ]);

        f.render_widget(Paragraph::new(line).alignment(Alignment::Left), cell);
    }
}
