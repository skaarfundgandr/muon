use chrono::Utc;

use crate::application::session::{SessionSummary, format_relative_time};
use crate::presentation::click::{ClickAction, ClickTarget, is_hovering};
use crate::presentation::theme;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

/// Bracket-style delete control — same convention as `[Remove]`, `[Approve]`, etc.
const DEL_LABEL: &str = "[x]";
const DEL_W: u16 = 3;

pub fn render(
    f: &mut ratatui::Frame,
    area: Rect,
    sessions: &[SessionSummary],
    scroll: usize,
    hit_registry: &mut Vec<ClickTarget>,
    mouse_col: u16,
    mouse_row: u16,
) {
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
    let slot_h: u16 = 3;
    let visible_count = ((inner.height / slot_h) as usize).max(1);
    let max_scroll = sessions.len().saturating_sub(visible_count);
    let scroll = scroll.min(max_scroll);

    let window: Vec<(usize, &SessionSummary)> = sessions
        .iter()
        .enumerate()
        .skip(scroll)
        .take(visible_count)
        .collect();

    for (local_i, (abs_i, session)) in window.iter().enumerate() {
        let dot_style = if session.is_active {
            Style::new()
                .fg(theme::success())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(theme::text_dim())
        };

        let time_str = format_relative_time(now, session.created_at);

        let offset = (local_i as u16).saturating_mul(slot_h);
        let slot_y = inner.y + offset;
        let target_h = slot_h.min(inner.height.saturating_sub(offset));
        if target_h == 0 {
            continue;
        }

        let slot_rect = Rect {
            x: inner.x,
            y: slot_y,
            width: inner.width,
            height: target_h,
        };
        let hovered = is_hovering(slot_rect, mouse_col, mouse_row);

        let show_del = inner.width > DEL_W;
        let del_rect = Rect {
            x: inner.x + inner.width.saturating_sub(DEL_W),
            y: slot_y,
            width: DEL_W,
            height: 1,
        };
        let del_hovered = show_del && is_hovering(del_rect, mouse_col, mouse_row);

        // Title content abuts the right-aligned delete button (no unpainted gap).
        let content_w = if show_del {
            inner.width.saturating_sub(DEL_W)
        } else {
            inner.width
        };

        // Paint full-width hover bg first so the strip under short titles + next to [x]
        // is darkened continuously across the title row.
        if hovered && offset < inner.height {
            f.render_widget(
                Paragraph::new("").style(Style::default().bg(theme::bg_dark())),
                Rect {
                    x: inner.x,
                    y: slot_y,
                    width: inner.width,
                    height: 1,
                },
            );
        }

        let title_style_main = if hovered {
            Style::new().fg(theme::text_main()).bg(theme::bg_dark())
        } else {
            Style::new().fg(theme::text_main())
        };
        let title_style_dim = if hovered {
            Style::new().fg(theme::text_dim()).bg(theme::bg_dark())
        } else {
            Style::new().fg(theme::text_dim())
        };
        let title_line = Line::from(vec![
            Span::styled(
                "●",
                if hovered {
                    dot_style.bg(theme::bg_dark())
                } else {
                    dot_style
                },
            ),
            Span::styled(format!(" {} ", session.title), title_style_main),
            Span::styled(time_str, title_style_dim),
        ]);

        if offset < inner.height && content_w > 0 {
            let title_rect = Rect {
                x: inner.x,
                y: slot_y,
                width: content_w,
                height: 1,
            };
            if hovered {
                f.render_widget(
                    Paragraph::new(title_line).style(Style::default().bg(theme::bg_dark())),
                    title_rect,
                );
            } else {
                f.render_widget(title_line, title_rect);
            }
        }

        if show_del {
            // Match [Remove]: error fg; bold when hovered (same as providers Remove).
            let del_style = if del_hovered {
                Style::new().fg(theme::error()).add_modifier(Modifier::BOLD)
            } else if hovered {
                Style::new().fg(theme::error())
            } else {
                Style::new().fg(theme::text_dim())
            };
            let del_bg = if hovered || del_hovered {
                Style::default().bg(theme::bg_dark())
            } else {
                Style::default()
            };
            f.render_widget(
                Paragraph::new(Span::styled(DEL_LABEL, del_style)).style(del_bg),
                del_rect,
            );
        }

        if offset + 1 < inner.height {
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
            let row_rect = Rect {
                x: inner.x,
                y: slot_y + 1,
                width: inner.width,
                height: 1,
            };
            if hovered {
                f.render_widget(
                    Paragraph::new(query_line).style(Style::default().bg(theme::bg_dark())),
                    row_rect,
                );
            } else {
                f.render_widget(query_line, row_rect);
            }
        }

        // Select covers the row except the delete button (pushed first so reverse-iter
        // still prefers Delete when they would overlap).
        let select_w = if show_del {
            inner.width.saturating_sub(DEL_W)
        } else {
            inner.width
        };
        if select_w > 0 {
            hit_registry.push(ClickTarget {
                rect: Rect {
                    x: inner.x,
                    y: slot_y,
                    width: select_w,
                    height: target_h,
                },
                action: ClickAction::SelectSession(*abs_i),
            });
        }
        if show_del {
            hit_registry.push(ClickTarget {
                rect: del_rect,
                action: ClickAction::DeleteSession(*abs_i),
            });
        }
    }
}
