#![allow(dead_code)]

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::domain::models::source::Source;
use crate::presentation::click::{ClickAction, ClickTarget, is_hovering};
use crate::presentation::theme;

#[derive(Clone, Copy)]
enum Verification {
    Exact,
    Prefix,
    ChildPath,
    QuerySubset,
    Unverified,
    Removed,
}

impl Verification {
    fn badge(&self) -> (&'static str, ratatui::style::Color) {
        match self {
            Verification::Exact => ("✓ EXACT", theme::success()),
            Verification::Prefix => ("~ PREFIX", theme::cyan()),
            Verification::ChildPath => ("↗ CHILD-PATH", theme::accent()),
            Verification::QuerySubset => ("⊞ QUERY-SUBSET", theme::purple()),
            Verification::Unverified => ("○ UNVERIFIED", theme::text_dim()),
            Verification::Removed => ("⚠ REMOVED", theme::error()),
        }
    }
}

struct SourceEntry {
    title: String,
    snippet: String,
    domain: String,
    url: String,
    relevance: u16,
    verification: Verification,
    source_type: SourceType,
}

#[derive(Clone, Copy)]
enum SourceType {
    Web,
    Paper,
}

impl SourceType {
    fn icon(&self) -> &'static str {
        match self {
            SourceType::Web => "🌐 web",
            SourceType::Paper => "📄 paper",
        }
    }

    fn color(&self) -> ratatui::style::Color {
        match self {
            SourceType::Web => theme::accent(),
            SourceType::Paper => theme::purple(),
        }
    }
}

fn get_domain(url: &str) -> &str {
    let mut d = url;
    if let Some(idx) = d.find("://") {
        d = &d[idx + 3..];
    }
    if let Some(idx) = d.find('/') {
        d = &d[..idx];
    }
    d
}

const CARD_MIN_HEIGHT: u16 = 5;

#[allow(clippy::too_many_arguments)]
pub fn render(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    sources: &[Source],
    scroll_offset: usize,
    mouse_col: u16,
    mouse_row: u16,
    hit_registry: &mut Vec<ClickTarget>,
) {
    let items: Vec<SourceEntry> = sources
        .iter()
        .map(|s| {
            let domain = get_domain(&s.url).to_string();
            let relevance = if s.relevance <= 1.0 {
                (s.relevance * 100.0) as u16
            } else {
                s.relevance as u16
            };
            let verification = match s.verification_status {
                crate::domain::models::source::VerificationStatus::Exact => Verification::Exact,
                crate::domain::models::source::VerificationStatus::Prefix => Verification::Prefix,
                crate::domain::models::source::VerificationStatus::ChildPath => {
                    Verification::ChildPath
                }
                crate::domain::models::source::VerificationStatus::QuerySubset => {
                    Verification::QuerySubset
                }
                crate::domain::models::source::VerificationStatus::Removed => Verification::Removed,
                crate::domain::models::source::VerificationStatus::Unverified => {
                    Verification::Unverified
                }
            };
            let source_type = match s.source_type {
                crate::domain::models::source::SourceType::Web => SourceType::Web,
                crate::domain::models::source::SourceType::Paper => SourceType::Paper,
                crate::domain::models::source::SourceType::Knowledge => SourceType::Web,
            };
            SourceEntry {
                title: if s.title.is_empty() {
                    s.url.clone()
                } else {
                    s.title.clone()
                },
                snippet: s.snippet.clone(),
                domain,
                url: s.url.clone(),
                relevance,
                verification,
                source_type,
            }
        })
        .collect();

    let panel_hover = is_hovering(area, mouse_col, mouse_row);
    let panel_border = if panel_hover {
        theme::border_hover()
    } else {
        theme::border()
    };

    let title_text = format!(" VERIFIED SOURCES ({}/{}) ", items.len(), items.len());
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(panel_border))
        .title(Span::styled(
            title_text,
            Style::default()
                .fg(theme::purple())
                .add_modifier(Modifier::BOLD),
        ));

    let outer_inner = outer_block.inner(area);
    f.render_widget(outer_block, area);

    let header_row = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(outer_inner);

    let removed_n = items
        .iter()
        .filter(|e| matches!(e.verification, Verification::Removed))
        .count();
    let unverified_n = items
        .iter()
        .filter(|e| matches!(e.verification, Verification::Unverified))
        .count();
    let total_status = if items.is_empty() {
        Line::from(Span::styled(
            "No sources",
            Style::default().fg(theme::text_dim()),
        ))
    } else if removed_n == 0 && unverified_n == 0 {
        Line::from(vec![
            Span::styled(
                "ALL CHECKS PASSED",
                Style::default()
                    .fg(theme::success())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ✓", Style::default().fg(theme::success())),
            Span::styled(
                "  (click card to copy URL)",
                Style::default().fg(theme::text_dim()),
            ),
        ])
    } else if removed_n > 0 {
        Line::from(vec![
            Span::styled(
                format!("{removed_n} REMOVED"),
                Style::default()
                    .fg(theme::error())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  (click card to copy URL)",
                Style::default().fg(theme::text_dim()),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled(
                format!("{unverified_n} UNVERIFIED"),
                Style::default().fg(theme::text_dim()),
            ),
            Span::styled(
                "  (click card to copy URL)",
                Style::default().fg(theme::text_dim()),
            ),
        ])
    };
    f.render_widget(
        Paragraph::new(total_status),
        Rect {
            x: header_row[0].x,
            y: header_row[0].y,
            width: header_row[0].width,
            height: 1,
        },
    );

    let body_area = header_row[1];

    if items.is_empty() {
        let empty_p = Paragraph::new("No sources yet")
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme::text_dim()));
        f.render_widget(empty_p, body_area);
        return;
    }

    let visible_count = ((body_area.height / CARD_MIN_HEIGHT) as usize).max(1);
    let max_scroll = items.len().saturating_sub(visible_count);
    let scroll = scroll_offset.min(max_scroll);
    let window: Vec<(usize, &SourceEntry)> = items
        .iter()
        .enumerate()
        .skip(scroll)
        .take(visible_count)
        .collect();

    if window.is_empty() {
        return;
    }

    let constraints: Vec<Constraint> = window
        .iter()
        .map(|_| Constraint::Length(CARD_MIN_HEIGHT))
        .collect();
    let mut layout_constraints = constraints;
    layout_constraints.push(Constraint::Min(0));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(layout_constraints)
        .split(body_area);

    for (slot, (global_idx, entry)) in window.iter().enumerate() {
        let card_area = chunks[slot];
        let card_hover = is_hovering(card_area, mouse_col, mouse_row);
        let is_removed = matches!(entry.verification, Verification::Removed);

        let border_color = if card_hover {
            theme::border_hover()
        } else if is_removed {
            theme::warning()
        } else {
            theme::border()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        hit_registry.push(ClickTarget {
            rect: card_area,
            action: ClickAction::CopySourceUrl(entry.url.clone()),
        });

        let index_style = Style::default()
            .fg(theme::purple())
            .add_modifier(Modifier::BOLD);
        let index_span = Span::styled(format!("[{}] ", global_idx + 1), index_style);
        let title_span = Span::styled(&entry.title, Style::default().fg(theme::text_main()));

        let rel_color = if entry.relevance >= 85 {
            theme::success()
        } else if entry.relevance >= 60 {
            theme::warning()
        } else {
            theme::text_dim()
        };

        let (badge_text, badge_color) = entry.verification.badge();
        let badge = Span::styled(
            badge_text,
            Style::default()
                .fg(badge_color)
                .add_modifier(Modifier::BOLD),
        );

        let bar_width = 10u16;
        let filled_count = (entry.relevance as u32 * bar_width as u32 / 100) as usize;
        let empty_count = bar_width as usize - filled_count;
        let filled: String = "█".repeat(filled_count);
        let empty: String = "░".repeat(empty_count);

        let type_span = Span::styled(
            entry.source_type.icon(),
            Style::default().fg(entry.source_type.color()),
        );

        let header_line = Line::from(vec![index_span, title_span]);
        let meta_line = Line::from(vec![
            badge,
            Span::raw("  "),
            type_span,
            Span::styled(" • ", Style::default().fg(theme::text_dim())),
            Span::styled(&entry.domain, Style::default().fg(theme::text_dim())),
            Span::raw("  "),
            Span::styled(
                format!("Relevance: {}% ", entry.relevance),
                Style::default().fg(rel_color),
            ),
            Span::styled(filled, Style::default().fg(rel_color)),
            Span::styled(empty, Style::default().fg(theme::text_dim())),
        ]);

        let warning_text = if is_removed {
            Line::from(Span::styled(
                "⚠ URL truncated — partial match on index server",
                Style::default()
                    .fg(theme::warning())
                    .add_modifier(Modifier::BOLD),
            ))
        } else {
            Line::from("")
        };

        let snippet_line = Line::from(Span::styled(
            &entry.snippet,
            Style::default().fg(theme::text_dim()),
        ));

        let inner = block.inner(card_area);
        f.render_widget(block, card_area);
        let lines = vec![header_line, meta_line];
        let snippet_offset = if is_removed { 1u16 } else { 0u16 };
        f.render_widget(
            Paragraph::new(lines),
            Rect {
                x: inner.x,
                y: inner.y,
                width: inner.width,
                height: 2,
            },
        );
        if is_removed {
            f.render_widget(
                Paragraph::new(warning_text),
                Rect {
                    x: inner.x,
                    y: inner.y + 2,
                    width: inner.width,
                    height: 1,
                },
            );
        }
        f.render_widget(
            Paragraph::new(snippet_line),
            Rect {
                x: inner.x,
                y: inner.y + 2 + snippet_offset,
                width: inner.width,
                height: 1,
            },
        );
    }
}
