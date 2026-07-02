use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::presentation::theme::{ACCENT, BORDER, CYAN, PURPLE, SUCCESS, TEXT_DIM, TEXT_MAIN, WARNING};

struct SourceEntry {
    title: &'static str,
    snippet: &'static str,
    domain: &'static str,
    relevance: u16,
    verified: bool,
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
            SourceType::Web => ACCENT,
            SourceType::Paper => PURPLE,
            SourceType::Code => CYAN,
        }
    }
}

pub fn render(f: &mut ratatui::Frame<'_>, area: Rect) {
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER))
        .title(Span::styled(
            " VERIFIED SOURCES (8/8) ",
            Style::default().fg(PURPLE).add_modifier(Modifier::BOLD),
        ));

    let outer_inner = outer_block.inner(area);
    f.render_widget(outer_block, area);

    let header_row = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(outer_inner);

    let total_status = Line::from(vec![
        Span::styled("ALL CHECKS PASSED", Style::default().fg(SUCCESS).add_modifier(Modifier::BOLD)),
        Span::styled(" ✓", Style::default().fg(SUCCESS)),
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
            verified: true,
            source_type: SourceType::Web,
        },
        SourceEntry {
            title: "Japan Green Growth Strategy & Industrial Price Premiums",
            snippet: "High reliance on imported fossil fuels during renewable capacity scaling creates structural electricity premiums that challenge export industries in the Tokyo and Osaka manufacturing centers...",
            domain: "j-stage.jst.go.jp",
            relevance: 89,
            verified: true,
            source_type: SourceType::Paper,
        },
        SourceEntry {
            title: "Comparative Assessment of Feed-In Tariffs and Corporate PPAs",
            snippet: "Source code repository containing macroeconomic simulation parameters comparing Germany's feed-in tariff policies to Japan's corporate power purchase agreements...",
            domain: "github.com/econ-model/g7-energy",
            relevance: 85,
            verified: true,
            source_type: SourceType::Code,
        },
        SourceEntry {
            title: "Grid Constraints & Macroeconomic Performance projections",
            snippet: "...macroeconomic growth limitations due to power transmission capacity ceiling. Grid congestion costs are estimated to suppress German GDP growth potential by up to 0.15%...",
            domain: "energy-forecast-archive.net/g7...",
            relevance: 72,
            verified: false,
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
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(if entry.verified {
                Style::default().fg(BORDER)
            } else {
                Style::default().fg(WARNING)
            });

        let index_style = Style::default().fg(PURPLE).add_modifier(Modifier::BOLD);
        let index_span = Span::styled(format!("[{}] ", i + 1), index_style);
        let title_span = Span::styled(entry.title, Style::default().fg(TEXT_MAIN));

        let rel_color = if entry.relevance >= 85 {
            SUCCESS
        } else if entry.relevance >= 60 {
            WARNING
        } else {
            TEXT_DIM
        };

        let badge = if entry.verified {
            Span::styled("✓ VERIFIED", Style::default().fg(SUCCESS).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(
                "⚠ WARNING",
                Style::default().fg(WARNING).add_modifier(Modifier::BOLD),
            )
        };

        let bar_width = 10u16;
        let filled_count = (entry.relevance as u32 * bar_width as u32 / 100) as usize;
        let empty_count = bar_width as usize - filled_count;
        let filled: String = "█".repeat(filled_count);
        let empty: String = "░".repeat(empty_count);

        let type_span = Span::styled(entry.source_type.icon(), Style::default().fg(entry.source_type.color()));

        let header_line = Line::from(vec![index_span, title_span]);
        let meta_line = Line::from(vec![
            badge,
            Span::raw("  "),
            type_span,
            Span::styled(" • ", Style::default().fg(TEXT_DIM)),
            Span::styled(entry.domain, Style::default().fg(TEXT_DIM)),
            Span::raw("  "),
            Span::styled(format!("Relevance: {}% ", entry.relevance), Style::default().fg(rel_color)),
            Span::styled(filled, Style::default().fg(rel_color)),
            Span::styled(empty, Style::default().fg(TEXT_DIM)),
        ]);

        let warning_text = if !entry.verified {
            Line::from(Span::styled(
                "⚠ URL truncated — partial match on index server",
                Style::default().fg(WARNING).add_modifier(Modifier::BOLD),
            ))
        } else {
            Line::from("")
        };

        let snippet_line = Line::from(Span::styled(
            entry.snippet,
            Style::default().fg(TEXT_DIM),
        ));

        let inner = block.inner(chunks[i]);
        f.render_widget(block, chunks[i]);
        let lines = vec![header_line, meta_line];
        let snippet_offset = if !entry.verified { 1u16 } else { 0u16 };
        f.render_widget(
            Paragraph::new(lines),
            Rect {
                x: inner.x,
                y: inner.y,
                width: inner.width,
                height: 2,
            },
        );
        if !entry.verified {
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
