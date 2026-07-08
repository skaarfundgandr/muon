#![allow(dead_code)]

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::presentation::theme;

use crate::domain::models::report::ResearchReport;

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

pub fn render(f: &mut ratatui::Frame, area: Rect, report: Option<&ResearchReport>) {
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

    if let Some(r) = report {
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!("Title: {}", r.title),
                Style::new().fg(theme::text_main()).add_modifier(Modifier::BOLD),
            ))),
            chunks[0],
        );

        let mut body_lines = Vec::new();
        body_lines.push(Line::from(Span::styled(
            &r.summary,
            Style::new().fg(theme::text_main()),
        )));

        for section in &r.sections {
            body_lines.push(Line::from(""));
            body_lines.push(Line::from(Span::styled(
                format!("## {}", section.heading),
                Style::new().fg(theme::cyan()).add_modifier(Modifier::BOLD),
            )));
            body_lines.push(Line::from(Span::styled(
                &section.body_markdown,
                Style::new().fg(theme::text_main()),
            )));
        }

        if !r.citations.is_empty() {
            body_lines.push(Line::from(""));
            body_lines.push(Line::from(Span::styled(
                "Citations:",
                Style::new().fg(theme::success()).add_modifier(Modifier::BOLD),
            )));
            for cite in &r.citations {
                body_lines.push(Line::from(vec![
                    Span::styled(format!(" [{}] ", cite.reference_number), Style::new().fg(theme::cyan())),
                    Span::styled(&cite.title, Style::new().fg(theme::text_main())),
                    Span::styled(format!(" - {}", cite.url), Style::new().fg(theme::text_dim())),
                ]));
            }
        }

        f.render_widget(
            Paragraph::new(body_lines).wrap(ratatui::widgets::Wrap { trim: true }),
            chunks[1],
        );
    } else {
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "—",
                Style::new().fg(theme::text_dim()),
            ))),
            chunks[0],
        );

        let body_lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                "No report generated yet. Run a query to see results.",
                Style::new().fg(theme::text_dim()),
            )),
        ];
        f.render_widget(
            Paragraph::new(body_lines).alignment(Alignment::Center),
            chunks[1],
        );
    }

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

    let (sources_analyzed, docs_read, citations_verified, overall_confidence) = if let Some(r) = report {
        (
            r.stats.total_sources.to_string(),
            r.stats.verified_sources.to_string(),
            r.citations.len().to_string(),
            format!("{}%", if r.stats.total_sources > 0 {
                (r.stats.verified_sources * 100 / r.stats.total_sources).min(100)
            } else {
                100
            })
        )
    } else {
        ("—".to_string(), "—".to_string(), "—".to_string(), "—".to_string())
    };

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Sources Analyzed:    ", Style::new().fg(theme::text_dim())),
            Span::styled(
                sources_analyzed,
                Style::new().fg(theme::text_main()).add_modifier(Modifier::BOLD),
            ),
        ])),
        stats_rows[0],
    );
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Documents Deep-Read:  ", Style::new().fg(theme::text_dim())),
            Span::styled(
                docs_read,
                Style::new().fg(theme::text_main()).add_modifier(Modifier::BOLD),
            ),
        ])),
        stats_rows[1],
    );
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Citations Verified:   ", Style::new().fg(theme::text_dim())),
            Span::styled(
                citations_verified,
                Style::new().fg(theme::text_main()).add_modifier(Modifier::BOLD),
            ),
        ])),
        stats_rows[2],
    );
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Overall Confidence:  ", Style::new().fg(theme::text_dim())),
            Span::styled(overall_confidence, Style::new().fg(theme::text_main()).add_modifier(Modifier::BOLD)),
        ])),
        stats_rows[3],
    );

    let tag_line = if let Some(r) = report {
        Line::from(vec![
            Span::styled(format!("Elapsed: {}s | Tokens In/Out: {}/{}", r.stats.elapsed_secs, r.stats.tokens_in, r.stats.tokens_out), Style::new().fg(theme::text_dim())),
        ])
    } else {
        Line::from(vec![
            Span::styled("—", Style::new().fg(theme::text_dim())),
        ])
    };
    f.render_widget(Paragraph::new(tag_line), chunks[3]);
}
