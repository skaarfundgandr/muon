use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::presentation::theme;

fn citation_line<'a>(text: &'a str, citations: &[&'a str]) -> Line<'a> {
    let mut spans = Vec::new();
    let mut rest = text;
    for cite in citations {
        if let Some(idx) = rest.find(cite) {
            if idx > 0 {
                spans.push(Span::styled(&rest[..idx], Style::new().fg(theme::text_main())));
            }
            spans.push(Span::styled(*cite, Style::new().fg(theme::cyan())));
            rest = &rest[idx + cite.len()..];
        }
    }
    if !rest.is_empty() {
        spans.push(Span::styled(rest, Style::new().fg(theme::text_main())));
    }
    Line::from(spans)
}

pub fn render(f: &mut ratatui::Frame, area: Rect) {
    let body1 = "Germany's renewable transition, or Energiewende, has expanded green energy capacity to over 52% of domestic generation as of late 2024, driving regional job growth but causing structural cost shifts in industrial manufacturing sectors [1]. In contrast, Japan's green growth strategies prioritize offshore wind and grid improvements to combat high electricity price premiums that drag down industrial export competitiveness [2].";

    let body2 = "Comparative macroeconomic modeling indicates Germany's feed-in tariff policies yielded high initial deployment velocity but lower cost efficiency than Japan's corporate-backed PPAs [3]. Moving forward, both countries face critical bottlenecks in grid storage infrastructure and regulatory limits on inter-regional transmission loops, capping immediate GDP growth opportunities to 1.2-1.5% annually [4].";

    let block = Block::new()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme::border()))
        .title(Span::styled(
            " RESEARCH REPORT SUMMARY ",
            Style::new().fg(theme::border()),
        ));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(7),
            Constraint::Length(2),
        ])
        .split(inner);

    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "Economic Impacts of Renewable Energy in Germany & Japan",
            Style::new().fg(theme::accent()).add_modifier(Modifier::BOLD),
        ))),
        chunks[0],
    );

    let body_lines: Vec<Line> = vec![
        citation_line(body1, &["[1]", "[2]"]),
        Line::from(""),
        citation_line(body2, &["[3]", "[4]"]),
    ];
    f.render_widget(Paragraph::new(body_lines), chunks[1]);

    let stats_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme::border()))
        .title(Span::styled(" STATS ", Style::new().fg(theme::border())));
    let stats_inner = stats_block.inner(chunks[2]);
    f.render_widget(stats_block, chunks[2]);

    let stats_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(stats_inner);

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Sources Analyzed:    ", Style::new().fg(theme::text_dim())),
            Span::styled(
                "47",
                Style::new().fg(theme::text_main()).add_modifier(Modifier::BOLD),
            ),
        ])),
        stats_rows[0],
    );
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Documents Deep-Read:  ", Style::new().fg(theme::text_dim())),
            Span::styled(
                "12",
                Style::new().fg(theme::text_main()).add_modifier(Modifier::BOLD),
            ),
        ])),
        stats_rows[1],
    );
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Citations Verified:   ", Style::new().fg(theme::text_dim())),
            Span::styled(
                "8 / 8 (100% ✓)",
                Style::new().fg(theme::success()).add_modifier(Modifier::BOLD),
            ),
        ])),
        stats_rows[2],
    );
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Overall Confidence:  ", Style::new().fg(theme::text_dim())),
            Span::styled("87%", Style::new().fg(theme::cyan()).add_modifier(Modifier::BOLD)),
        ])),
        stats_rows[3],
    );

    let tag_line = Line::from(vec![
        Span::styled("#renewable", Style::new().fg(theme::purple())),
        Span::raw("  "),
        Span::styled("#energy", Style::new().fg(theme::purple())),
        Span::raw("  "),
        Span::styled("#germany", Style::new().fg(theme::purple())),
        Span::raw("  "),
        Span::styled("#japan", Style::new().fg(theme::purple())),
        Span::raw("  "),
        Span::styled("#gdp", Style::new().fg(theme::purple())),
    ]);
    f.render_widget(Paragraph::new(tag_line), chunks[3]);
}
