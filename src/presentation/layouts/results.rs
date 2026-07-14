use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::application::pipeline::PipelineState;
use crate::presentation::click::{ClickAction, ClickTarget, is_hovering};
use crate::presentation::components::header::HeaderConfig;
use crate::presentation::components::*;
use crate::presentation::theme;
use crate::presentation::views::View;

struct ActionBtn {
    label: &'static str,
    action: Option<ClickAction>,
    crossed: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn render(
    f: &mut ratatui::Frame,
    area: Rect,
    pipeline: &PipelineState,
    last_report: Option<&crate::domain::models::report::ResearchReport>,
    last_sources: &[crate::domain::models::source::Source],
    hit_registry: &mut Vec<ClickTarget>,
    mouse_col: u16,
    mouse_row: u16,
    report_scroll: usize,
    source_scroll: usize,
    full_report_mode: bool,
) {
    let total_time_secs = pipeline.elapsed_secs();
    f.render_widget(
        Block::default().style(Style::default().bg(theme::bg_main())),
        area,
    );

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(area);

    let header_area = vertical[0];
    let body_area = vertical[1];
    let actions_area = vertical[2];
    let footer_area = vertical[3];

    header::render(
        f,
        header_area,
        HeaderConfig::for_view(View::Results, total_time_secs),
    );
    footer::render(
        f,
        footer_area,
        View::Results,
        hit_registry,
        mouse_col,
        mouse_row,
    );

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(body_area);

    let report_area = horizontal[0];
    let sources_area = horizontal[1];

    report_view::render(
        f,
        report_area,
        last_report,
        report_scroll,
        mouse_col,
        mouse_row,
        full_report_mode,
    );
    source_card::render(
        f,
        sources_area,
        last_sources,
        source_scroll,
        mouse_col,
        mouse_row,
        hit_registry,
    );

    let action_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::border()));
    let inner = action_block.inner(actions_area);
    f.render_widget(action_block, actions_area);

    let full_label = if full_report_mode {
        "[F6] Summary view"
    } else {
        "[F6] Full Report"
    };

    let buttons = [
        ActionBtn {
            label: "[F1] Export MD",
            action: Some(ClickAction::ExportMarkdown),
            crossed: false,
        },
        ActionBtn {
            label: "[F2] Export PDF",
            action: Some(ClickAction::ExportPdf),
            crossed: false,
        },
        ActionBtn {
            label: "[F3] Sync Obsidian",
            action: Some(ClickAction::SyncObsidian),
            crossed: false,
        },
        ActionBtn {
            label: "[F4] New Query",
            action: Some(ClickAction::NewQuery),
            crossed: false,
        },
        ActionBtn {
            label: "[F5] Refine Search",
            action: Some(ClickAction::RefineSearch),
            crossed: false,
        },
        ActionBtn {
            label: full_label,
            action: Some(ClickAction::FullReportView),
            crossed: false,
        },
    ];

    let constraints: Vec<Constraint> = buttons.iter().map(|_| Constraint::Ratio(1, 6)).collect();
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(inner);

    for (i, btn) in buttons.iter().enumerate() {
        let rect = chunks[i];
        let hovered = is_hovering(rect, mouse_col, mouse_row) && btn.action.is_some();

        if let Some(action) = &btn.action {
            hit_registry.push(ClickTarget {
                rect,
                action: action.clone(),
            });
        }

        let style = if btn.crossed {
            Style::default()
                .fg(theme::text_dim())
                .add_modifier(Modifier::CROSSED_OUT)
        } else if hovered {
            Style::default()
                .fg(theme::border_hover())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::text_main())
        };

        let para = if btn.crossed {
            Paragraph::new(Line::from(Span::styled(btn.label, style))).alignment(Alignment::Center)
        } else if let Some((key, rest)) = btn.label.split_once(' ') {
            let key_style = if hovered {
                Style::default()
                    .fg(theme::border_hover())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(theme::purple())
                    .add_modifier(Modifier::BOLD)
            };
            let rest_style = if hovered {
                Style::default()
                    .fg(theme::border_hover())
                    .add_modifier(Modifier::BOLD)
            } else if full_report_mode && matches!(btn.action, Some(ClickAction::FullReportView)) {
                Style::default()
                    .fg(theme::accent())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::text_main())
            };
            Paragraph::new(Line::from(vec![
                Span::styled(key, key_style),
                Span::styled(format!(" {rest}"), rest_style),
            ]))
            .alignment(Alignment::Center)
        } else {
            Paragraph::new(Line::from(Span::styled(btn.label, style))).alignment(Alignment::Center)
        };
        f.render_widget(para, rect);
    }
}
