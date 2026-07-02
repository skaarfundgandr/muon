use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::presentation::theme::{ACTIVE_STYLE, DIM_STYLE, HEADER_STYLE};
use crate::presentation::views::View;

pub fn render(f: &mut ratatui::Frame, area: Rect, active: View) {
    let block = Block::default().style(HEADER_STYLE);
    f.render_widget(block, area);

    let inner = area.inner(ratatui::layout::Margin::new(1, 0));
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(inner);

    let tabs: &[(&str, &str, Option<View>)] = &[
        ("F1 ", "Dashboard", Some(View::Dashboard)),
        ("F2 ", "Progress", Some(View::Progress)),
        ("F3 ", "Results", Some(View::Results)),
        ("F4 ", "Settings", Some(View::Settings)),
        ("F5 ", "Help", None),
    ];

    let mut spans: Vec<Span> = Vec::new();
    for (i, (key, label, view)) in tabs.iter().enumerate() {
        let is_active = view.is_some_and(|v| v == active);
        let style = if is_active { ACTIVE_STYLE } else { DIM_STYLE };
        spans.push(Span::styled(*key, style));
        spans.push(Span::styled(*label, style));
        if i < tabs.len() - 1 {
            spans.push(Span::styled(" | ", DIM_STYLE));
        }
    }

    let tabs_line = Line::from(spans);
    let tabs_para = Paragraph::new(tabs_line);
    f.render_widget(tabs_para, Rect {
        x: chunks[0].x,
        y: chunks[0].y,
        width: chunks[0].width,
        height: 1,
    });

    let hints_line = Line::from(Span::styled(
        "[Esc] Cancel  |  [Tab] Navigate  |  [^C] Exit μon",
        DIM_STYLE,
    ))
    .alignment(Alignment::Right);
    let hints_para = Paragraph::new(hints_line);
    f.render_widget(hints_para, Rect {
        x: chunks[1].x,
        y: chunks[1].y,
        width: chunks[1].width,
        height: 1,
    });
}
