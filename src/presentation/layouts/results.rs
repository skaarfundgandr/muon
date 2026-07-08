use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::application::pipeline::PipelineState;
use crate::presentation::click::ClickTarget;
use crate::presentation::components::header::HeaderConfig;
use crate::presentation::components::*;
use crate::presentation::theme;
use crate::presentation::views::View;

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
) {
    let total_time_secs = pipeline.elapsed_secs();
    f.render_widget(Block::default().style(Style::default().bg(theme::bg_main())), area);

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
    footer::render(f, footer_area, View::Results, hit_registry, mouse_col, mouse_row);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(body_area);

    let report_area = horizontal[0];
    let sources_area = horizontal[1];

    report_view::render(f, report_area, last_report);
    source_card::render(f, sources_area, last_sources);

    let action_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::border()));

    let action_line = Line::from(vec![
        Span::styled(
            "[F1]",
            Style::default().fg(theme::purple()).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Export MD", Style::default().fg(theme::text_main())),
        Span::styled("  |  ", Style::default().fg(theme::text_dim())),
        Span::styled(
            "[F2]",
            Style::default().fg(theme::purple()).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " Export PDF",
            Style::default()
                .fg(theme::text_dim())
                .add_modifier(Modifier::CROSSED_OUT),
        ),
        Span::styled(" (v0.2)", Style::default().fg(theme::text_dim())),
        Span::styled("  |  ", Style::default().fg(theme::text_dim())),
        Span::styled(
            "[F3]",
            Style::default().fg(theme::purple()).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Sync Obsidian", Style::default().fg(theme::text_main())),
        Span::styled("  |  ", Style::default().fg(theme::text_dim())),
        Span::styled(
            "[F4]",
            Style::default().fg(theme::purple()).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" New Query", Style::default().fg(theme::text_main())),
        Span::styled("  |  ", Style::default().fg(theme::text_dim())),
        Span::styled(
            "[F5]",
            Style::default().fg(theme::purple()).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Refine Search", Style::default().fg(theme::text_main())),
        Span::styled("  |  ", Style::default().fg(theme::text_dim())),
        Span::styled(
            "[F6]",
            Style::default().fg(theme::purple()).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Full Report view", Style::default().fg(theme::text_main())),
    ]);

    let action_para = Paragraph::new(action_line).alignment(Alignment::Center);
    let inner = action_block.inner(actions_area);
    f.render_widget(action_block, actions_area);
    f.render_widget(action_para, inner);
}
