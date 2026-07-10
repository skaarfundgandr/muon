use chrono::Utc;

use crate::presentation::click::{ClickAction, ClickTarget};
use crate::application::session::{SessionSummary, format_relative_time};
use crate::presentation::click::is_hovering;
use crate::presentation::theme;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(f: &mut ratatui::Frame, area: Rect, sessions: &[SessionSummary], hit_registry: &mut Vec<ClickTarget>, mouse_col: u16, mouse_row: u16) {
    let block = Block::default()
        .title(" RECENT SESSIONS ")
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme::border()));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    if sessions.is_empty() {
        f.render_widget(
            Paragraph::new(Line::from(vec![Span::styled(
                "No sessions yet. Press Enter to start researching.",
                Style::new().fg(theme::text_dim()),
            )])),
            inner,
        );
        return;
    }

    let now = Utc::now();
    // Each session occupies a 3-line slot: title, query, blank separator
    // (the last session omits the blank line but still claims the slot
    // height for click-target purposes).
    let slot_h: u16 = 3;

    for (i, session) in sessions.iter().enumerate() {
        let dot_style = if session.is_active {
            Style::new().fg(theme::success()).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(theme::text_dim())
        };

        let time_str = format_relative_time(now, session.created_at);

        let slot_y = inner.y + (i as u16).saturating_mul(slot_h);
        let slot_rect = Rect { x: inner.x, y: slot_y, width: inner.width, height: slot_h.min(inner.height.saturating_sub(slot_y)) };
        let hovered = is_hovering(slot_rect, mouse_col, mouse_row);

        let title_line = if hovered {
            Line::from(vec![
                Span::styled("●", dot_style),
                Span::styled(format!(" {} ", session.title), Style::new().fg(theme::text_main()).bg(theme::bg_dark())),
                Span::styled(time_str, Style::new().fg(theme::text_dim()).bg(theme::bg_dark())),
            ])
        } else {
            Line::from(vec![
                Span::styled("●", dot_style),
                Span::styled(format!(" {} ", session.title), Style::new().fg(theme::text_main())),
                Span::styled(time_str, Style::new().fg(theme::text_dim())),
            ])
        };
        let query_line = if hovered {
            Line::from(vec![Span::styled(
                format!("   {}", session.query),
                Style::new().fg(theme::text_dim()).bg(theme::bg_dark()),
            )])
        } else {
            Line::from(vec![Span::styled(
                format!("   {}", session.query),
                Style::new().fg(theme::text_dim()),
            )])
        };

        // Render these two lines at the correct vertical offset.
        let offset = (i as u16).saturating_mul(slot_h);
        if offset < inner.height {
            let row_rect = Rect { x: inner.x, y: inner.y + offset, width: inner.width, height: 1 };
            if hovered {
                f.render_widget(Paragraph::new(title_line).style(Style::default().bg(theme::bg_dark())), row_rect);
            } else {
                f.render_widget(title_line, row_rect);
            }
        }
        if offset + 1 < inner.height {
            let row_rect = Rect { x: inner.x, y: inner.y + offset + 1, width: inner.width, height: 1 };
            if hovered {
                f.render_widget(Paragraph::new(query_line).style(Style::default().bg(theme::bg_dark())), row_rect);
            } else {
                f.render_widget(query_line, row_rect);
            }
        }

        // Register a click target spanning this session's slot.
        let target_h = slot_h.min(inner.height.saturating_sub(offset));
        if target_h > 0 {
            hit_registry.push(ClickTarget {
                rect: Rect {
                    x: inner.x,
                    y: inner.y + offset,
                    width: inner.width,
                    height: target_h,
                },
                action: ClickAction::SelectSession(i),
            });
        }
    }
}
