use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::Block;

use crate::application::pipeline::PipelineState;
use crate::presentation::click::{ClickAction, ClickTarget};
use crate::presentation::components::header::HeaderConfig;
use crate::presentation::components::*;
use crate::presentation::theme;
use crate::presentation::views::View;
use crate::session::SessionSummary;
use crate::presentation::components::query_input::QueryInput;

#[allow(clippy::too_many_arguments)]
pub fn render(
    f: &mut ratatui::Frame,
    area: Rect,
    query: &QueryInput,
    sessions: &[SessionSummary],
    pipeline: &PipelineState,
    hit_registry: &mut Vec<ClickTarget>,
    mouse_col: u16,
    mouse_row: u16,
    clarifier_question: Option<&str>,
    clarifier_response: &str,
) {
    f.render_widget(Block::default().style(Style::default().bg(theme::bg_main())), area);

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let header_area = vertical[0];
    let body_area = vertical[1];
    let footer_area = vertical[2];

    header::render(f, header_area, HeaderConfig::for_view(View::Dashboard, 0));
    footer::render(f, footer_area, View::Dashboard, hit_registry, mouse_col, mouse_row);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(body_area);

    let sidebar_area = horizontal[0];
    let main_area = horizontal[1];

    let sidebar_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(sidebar_area);

    session_list::render(f, sidebar_split[0], sessions, hit_registry, mouse_col, mouse_row);
    telemetry_panel::render(f, sidebar_split[1]);

    let main_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(8),
            Constraint::Min(3),
        ])
        .split(main_area);

    hit_registry.push(ClickTarget {
        rect: main_split[0],
        action: ClickAction::ActivateQueryInput,
    });
    query_input::render(f, main_split[0], query);
    let mut clarifier_input_rect: Option<ratatui::layout::Rect> = None;
    pipeline_graph::render_horizontal(f, main_split[1], Some(clarifier_panel::render), clarifier_question, clarifier_response, mouse_col, mouse_row, pipeline, &mut clarifier_input_rect);
    if let Some(rect) = clarifier_input_rect
        && clarifier_question.is_some()
    {
        hit_registry.push(ClickTarget {
            rect,
            action: ClickAction::ActivateClarifier,
        });
    }
    source_registry::render(f, main_split[2]);
}
