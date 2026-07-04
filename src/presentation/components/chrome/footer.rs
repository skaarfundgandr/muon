use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::presentation::theme::{ACTIVE_STYLE, DIM_STYLE, HEADER_STYLE};
use crate::presentation::views::View;

pub struct FooterConfig {
    pub tabs: Vec<(&'static str, &'static str, Option<View>)>,
    pub right_hint: Vec<(&'static str, &'static str)>,
}

impl FooterConfig {
    pub fn default_tabs() -> Vec<(&'static str, &'static str, Option<View>)> {
        vec![
            ("1  ", "Dashboard", Some(View::Dashboard)),
            ("2  ", "Progress", Some(View::Progress)),
            ("3  ", "Results", Some(View::Results)),
            ("4  ", "Settings", Some(View::Settings)),
            ("? ", "Help", None),
        ]
    }

    pub fn for_view(view: View) -> Self {
        match view {
            View::Progress => Self {
                tabs: Self::default_tabs()
                    .into_iter()
                    .filter(|(k, _, _)| *k != "? ")
                    .collect(),
                right_hint: vec![
                    ("[Esc]", "Pause"),
                    ("[Tab]", "Navigate"),
                    ("[Ctrl+C]", "Abort"),
                ],
            },
            View::Results => Self {
                tabs: Self::default_tabs()
                    .into_iter()
                    .filter(|(k, _, _)| *k != "? ")
                    .collect(),
                right_hint: vec![("[Tab]", "Navigate"), ("[^C]", "Exit μon")],
            },
            View::Settings => Self {
                tabs: Self::default_tabs()
                    .into_iter()
                    .filter(|(k, _, _)| *k != "? ")
                    .collect(),
                right_hint: vec![
                    ("[Tab]", "switch"),
                    ("[↑↓]", "navigate"),
                    ("[Enter]", "edit"),
                    ("[Ctrl+S]", "save"),
                    ("[Esc]", "discard"),
                ],
            },
            _ => Self {
                tabs: Self::default_tabs(),
                right_hint: vec![
                    ("[Esc]", "Cancel"),
                    ("[Tab]", "Navigate"),
                    ("[^C]", "Exit μon"),
                ],
            },
        }
    }
}

pub fn render(f: &mut ratatui::Frame, area: Rect, active: View) {
    let block = Block::default().style(HEADER_STYLE);
    f.render_widget(block, area);

    let config = FooterConfig::for_view(active);

    let inner = area.inner(Margin::new(1, 0));
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    let mut spans: Vec<Span> = Vec::new();
    for (i, (key, label, view)) in config.tabs.iter().enumerate() {
        let is_active = view.is_some_and(|v| v == active);
        let style = if is_active { ACTIVE_STYLE } else { DIM_STYLE };
        spans.push(Span::styled(*key, style));
        spans.push(Span::styled(*label, style));
        if i < config.tabs.len() - 1 {
            spans.push(Span::styled(" | ", DIM_STYLE));
        }
    }

    let tabs_line = Line::from(spans);
    f.render_widget(
        Paragraph::new(tabs_line),
        Rect {
            x: chunks[0].x,
            y: chunks[0].y,
            width: chunks[0].width,
            height: 1,
        },
    );

    let mut right_spans: Vec<Span> = Vec::new();
    for (i, (key, label)) in config.right_hint.iter().enumerate() {
        if i > 0 {
            right_spans.push(Span::styled("  |  ", DIM_STYLE));
        }
        right_spans.push(Span::styled(*key, DIM_STYLE));
        right_spans.push(Span::styled(format!(" {}", label), DIM_STYLE));
    }

    let hints_line = Line::from(right_spans).alignment(Alignment::Right);
    f.render_widget(
        Paragraph::new(hints_line),
        Rect {
            x: chunks[1].x,
            y: chunks[1].y,
            width: chunks[1].width,
            height: 1,
        },
    );
}
