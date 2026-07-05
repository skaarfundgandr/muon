use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::presentation::click::{is_hovering, ClickAction, ClickTarget};
use crate::presentation::theme;
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
                    ("[Esc]", "Exit"),
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

pub fn render(f: &mut ratatui::Frame, area: Rect, active: View, hit_registry: &mut Vec<ClickTarget>, mouse_col: u16, mouse_row: u16) {
    let block = Block::default().style(theme::header_style());
    f.render_widget(block, area);

    let config = FooterConfig::for_view(active);

    let inner = area.inner(Margin::new(1, 0));
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    let mut spans: Vec<Span> = Vec::new();
    let mut cursor: u16 = chunks[0].x;
    for (i, (key, label, view)) in config.tabs.iter().enumerate() {
        let is_active = view.is_some_and(|v| v == active);
        let key_w = key.chars().count() as u16;
        let label_w = label.chars().count() as u16;
        let seg_w = key_w + label_w;
        let seg_rect = Rect::new(cursor, chunks[0].y, seg_w, 1);
        let hovered = is_hovering(seg_rect, mouse_col, mouse_row);
        let style = if is_active {
            theme::active_style()
        } else if hovered {
            Style::default().fg(theme::border_hover())
        } else {
            theme::dim_style()
        };
        spans.push(Span::styled(*key, style));
        spans.push(Span::styled(*label, style));
        if i < config.tabs.len() - 1 {
            spans.push(Span::styled(" | ", theme::dim_style()));
        }
        if let Some(v) = view {
            let click_rect = Rect::new(cursor, chunks[0].y, seg_w, 1);
            hit_registry.push(ClickTarget {
                rect: click_rect,
                action: ClickAction::SwitchView(*v),
            });
        }
        cursor = cursor.saturating_add(seg_w + 3);
    }

    let tabs_rect = Rect {
        x: chunks[0].x,
        y: chunks[0].y,
        width: chunks[0].width,
        height: 1,
    };
    let tabs_line = Line::from(spans);
    f.render_widget(Paragraph::new(tabs_line), tabs_rect);

    let mut right_spans: Vec<Span> = Vec::new();
    for (i, (key, label)) in config.right_hint.iter().enumerate() {
        if i > 0 {
            right_spans.push(Span::styled("  |  ", theme::dim_style()));
        }
        right_spans.push(Span::styled(*key, theme::dim_style()));
        right_spans.push(Span::styled(format!(" {}", label), theme::dim_style()));
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
