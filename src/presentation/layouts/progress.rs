use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::application::pipeline::PipelineState;
use crate::presentation::click::ClickTarget;
use crate::presentation::components::header::HeaderConfig;
use crate::presentation::components::*;
use crate::presentation::theme;
use crate::presentation::views::View;

pub fn render(
    f: &mut ratatui::Frame,
    area: Rect,
    pipeline: &PipelineState,
    live_feed: &[crate::domain::models::log_entry::LogEntry],
    live_feed_scroll: usize,
    hit_registry: &mut Vec<ClickTarget>,
    mouse_col: u16,
    mouse_row: u16,
) {
    let bg = Block::default().style(Style::default().bg(theme::bg_main()));
    f.render_widget(bg, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(2),
            Constraint::Length(1),
        ])
        .split(area);

    header::render(
        f,
        chunks[0],
        HeaderConfig::for_view(View::Progress, pipeline.elapsed_secs()),
    );

    let stage_name = format!("{:?}", pipeline.stage);
    let progress_bar = {
        let filled = pipeline.current_step.min(pipeline.total_steps);
        let empty = pipeline.total_steps.saturating_sub(filled);
        format!(
            "[{}{}]",
            "█".repeat(filled as usize),
            "░".repeat(empty as usize)
        )
    };
    let status_line = Line::from(vec![
        Span::styled(
            format!("Stage: {} ", stage_name),
            Style::default()
                .fg(theme::accent())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            progress_bar,
            Style::default().fg(theme::text_main()),
        ),
        Span::styled(
            format!(" {}/{}", pipeline.current_step, pipeline.total_steps),
            Style::default().fg(theme::text_dim()),
        ),
    ]);
    f.render_widget(
        Paragraph::new(status_line),
        chunks[1],
    );

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[2]);

    pipeline_graph::render(f, body_chunks[0], pipeline);
    live_feed::render(f, body_chunks[1], live_feed, live_feed_scroll);
    resource_monitor::render(f, chunks[3]);
    footer::render(f, chunks[4], View::Progress, hit_registry, mouse_col, mouse_row);
}
