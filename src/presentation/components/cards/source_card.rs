use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::presentation::theme;

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum Verification {
    Exact,
    Prefix,
    ChildPath,
    QuerySubset,
    Removed,
}

impl Verification {
    fn badge(&self) -> (&'static str, ratatui::style::Color) {
        match self {
            Verification::Exact => ("✓ EXACT", theme::success()),
            Verification::Prefix => ("~ PREFIX", theme::cyan()),
            Verification::ChildPath => ("↗ CHILD-PATH", theme::accent()),
            Verification::QuerySubset => ("⊞ QUERY-SUBSET", theme::purple()),
            Verification::Removed => ("⚠ REMOVED", theme::error()),
        }
    }
}

struct SourceEntry {
    title: &'static str,
    snippet: &'static str,
    domain: &'static str,
    relevance: u16,
    verification: Verification,
    source_type: SourceType,
}

#[derive(Clone, Copy)]
enum SourceType {
    Web,
    Paper,
    Code,
}

impl SourceType {
    fn icon(&self) -> &'static str {
        match self {
            SourceType::Web => "🌐 web",
            SourceType::Paper => "📄 paper",
            SourceType::Code => "💻 code",
        }
    }

    fn color(&self) -> ratatui::style::Color {
        match self {
            SourceType::Web => theme::accent(),
            SourceType::Paper => theme::purple(),
            SourceType::Code => theme::cyan(),
        }
    }
}

pub fn render(f: &mut ratatui::Frame<'_>, area: Rect) {
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::border()))
        .title(Span::styled(
            " VERIFIED SOURCES (8/8) ",
            Style::default().fg(theme::purple()).add_modifier(Modifier::BOLD),
        ));

    let outer_inner = outer_block.inner(area);
    f.render_widget(outer_block, area);

    let header_row = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(outer_inner);

    let total_status = Line::from(vec![
        Span::styled(
            "ALL CHECKS PASSED",
            Style::default().fg(theme::success()).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ✓", Style::default().fg(theme::success())),
    ]);
    f.render_widget(
        Paragraph::new(total_status),
        Rect {
            x: header_row[0].x,
            y: header_row[0].y,
            width: header_row[0].width,
            height: 1,
        },
    );

    let items = [
        SourceEntry {
            title: "IEA Germany 2024 Energy Policy Review",
            snippet: "Germany's renewable energy deployment reached record highs in 2024, contributing to structural modifications in price indexing across industrial sectors, albeit with temporary grid load bottlenecks...",
            domain: "iea.org",
            relevance: 96,
            verification: Verification::Exact,
            source_type: SourceType::Web,
        },
        SourceEntry {
            title: "Japan Green Growth Strategy & Industrial Price Premiums",
            snippet: "High reliance on imported fossil fuels during renewable capacity scaling creates structural electricity premiums that challenge export industries in the Tokyo and Osaka manufacturing centers...",
            domain: "j-stage.jst.go.jp",
            relevance: 89,
            verification: Verification::Prefix,
            source_type: SourceType::Paper,
        },
        SourceEntry {
            title: "Comparative Assessment of Feed-In Tariffs and Corporate PPAs",
            snippet: "Source code repository containing macroeconomic simulation parameters comparing Germany's feed-in tariff policies to Japan's corporate power purchase agreements...",
            domain: "github.com/econ-model/g7-energy",
            relevance: 85,
            verification: Verification::ChildPath,
            source_type: SourceType::Code,
        },
        SourceEntry {
            title: "Grid Constraints & Macroeconomic Performance projections",
            snippet: "...macroeconomic growth limitations due to power transmission capacity ceiling. Grid congestion costs are estimated to suppress German GDP growth potential by up to 0.15%...",
            domain: "energy-forecast-archive.net/g7...",
            relevance: 72,
            verification: Verification::Removed,
            source_type: SourceType::Web,
        },
    ];

    let body_area = header_row[1];
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(body_area);

    for (i, entry) in items.iter().enumerate() {
        let is_removed = matches!(entry.verification, Verification::Removed);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(if is_removed {
                Style::default().fg(theme::warning())
            } else {
                Style::default().fg(theme::border())
            });

        let index_style = Style::default().fg(theme::purple()).add_modifier(Modifier::BOLD);
        let index_span = Span::styled(format!("[{}] ", i + 1), index_style);
        let title_span = Span::styled(entry.title, Style::default().fg(theme::text_main()));

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
            Span::styled(entry.domain, Style::default().fg(theme::text_dim())),
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
                Style::default().fg(theme::warning()).add_modifier(Modifier::BOLD),
            ))
        } else {
            Line::from("")
        };

        let snippet_line = Line::from(Span::styled(entry.snippet, Style::default().fg(theme::text_dim())));

        let inner = block.inner(chunks[i]);
        f.render_widget(block, chunks[i]);
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
