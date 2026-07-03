use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::application::pipeline::PipelineState;
use crate::presentation::components::header::HeaderConfig;
use crate::presentation::components::*;
use crate::presentation::theme::{BG_MAIN, BORDER, PURPLE, TEXT_DIM, TEXT_MAIN};
use crate::presentation::views::View;

pub fn render(f: &mut ratatui::Frame, area: Rect, pipeline: &PipelineState) {
    let total_time_secs = pipeline.elapsed_secs();
    f.render_widget(Block::default().style(Style::default().bg(BG_MAIN)), area);

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
    footer::render(f, footer_area, View::Results);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(body_area);

    let report_area = horizontal[0];
    let sources_area = horizontal[1];

    report_view::render(f, report_area);
    source_card::render(f, sources_area);

    let action_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER));

    let action_line = Line::from(vec![
        Span::styled(
            "[F1]",
            Style::default().fg(PURPLE).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Export MD", Style::default().fg(TEXT_MAIN)),
        Span::styled("  |  ", Style::default().fg(TEXT_DIM)),
        Span::styled(
            "[F2]",
            Style::default().fg(PURPLE).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " Export PDF",
            Style::default()
                .fg(TEXT_DIM)
                .add_modifier(Modifier::CROSSED_OUT),
        ),
        Span::styled(" (v0.2)", Style::default().fg(TEXT_DIM)),
        Span::styled("  |  ", Style::default().fg(TEXT_DIM)),
        Span::styled(
            "[F3]",
            Style::default().fg(PURPLE).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Sync Obsidian", Style::default().fg(TEXT_MAIN)),
        Span::styled("  |  ", Style::default().fg(TEXT_DIM)),
        Span::styled(
            "[F4]",
            Style::default().fg(PURPLE).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" New Query", Style::default().fg(TEXT_MAIN)),
        Span::styled("  |  ", Style::default().fg(TEXT_DIM)),
        Span::styled(
            "[F5]",
            Style::default().fg(PURPLE).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Refine Search", Style::default().fg(TEXT_MAIN)),
        Span::styled("  |  ", Style::default().fg(TEXT_DIM)),
        Span::styled(
            "[F6]",
            Style::default().fg(PURPLE).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Full Report view", Style::default().fg(TEXT_MAIN)),
    ]);

    let action_para = Paragraph::new(action_line).alignment(Alignment::Center);
    let inner = action_block.inner(actions_area);
    f.render_widget(action_block, actions_area);
    f.render_widget(action_para, inner);
}
