use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::presentation::theme;
use crate::presentation::views::View;

pub struct HeaderConfig {
    pub badge: Option<&'static str>,
    pub extra_right: Vec<Span<'static>>,
}

impl HeaderConfig {
    pub fn for_settings(elapsed_secs: u64, dirty: bool) -> Self {
        let _ = elapsed_secs;
        let (text, color) = if dirty {
            ("UNSAVED CHANGES", theme::warning())
        } else {
            ("SAVED", theme::success())
        };
        Self {
            badge: Some("CONFIGURATION CONSOLE"),
            extra_right: vec![Span::styled(
                text,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            )],
        }
    }

    pub fn for_view(view: View, elapsed_secs: u64) -> Self {
        match view {
            View::Dashboard => Self {
                badge: None,
                extra_right: vec![],
            },
            View::Progress => Self {
                badge: Some("RESEARCHING [DEEP]"),
                extra_right: vec![Span::styled(
                    format!("ELAPSED: {}", format_elapsed(elapsed_secs)),
                    Style::default().fg(theme::accent()).add_modifier(Modifier::BOLD),
                )],
            },
            View::Results => Self {
                badge: Some("RESEARCH COMPLETE"),
                extra_right: vec![Span::styled(
                    format!("TOTAL TIME: {}", format_elapsed(elapsed_secs)),
                    Style::default().fg(theme::success()).add_modifier(Modifier::BOLD),
                )],
            },
            View::Settings => Self {
                badge: Some("CONFIGURATION CONSOLE"),
                extra_right: vec![Span::styled(
                    "UNSAVED CHANGES",
                    Style::default().fg(theme::warning()).add_modifier(Modifier::BOLD),
                )],
            },
            View::Welcome => Self {
                badge: None,
                extra_right: vec![],
            },
        }
    }
}

pub fn format_elapsed(total_secs: u64) -> String {
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}

pub fn render(f: &mut ratatui::Frame, area: Rect, config: HeaderConfig) {
    let badge_color = match config.badge {
        Some("RESEARCH COMPLETE") => theme::success(),
        Some("RESEARCHING [DEEP]") => theme::accent(),
        Some("CONFIGURATION CONSOLE") => theme::accent(),
        _ => theme::accent(),
    };

    let mut left_spans: Vec<Span> = vec![Span::styled(
        "μon // Deep Research Agent",
        Style::default().fg(theme::accent()).add_modifier(Modifier::BOLD),
    )];

    if let Some(text) = config.badge {
        left_spans.push(Span::raw("  "));
        left_spans.push(Span::styled(
            format!(" {} ", text),
            Style::default()
                .fg(badge_color)
                .add_modifier(Modifier::BOLD),
        ));
        left_spans.push(Span::styled(" ●", theme::success_style()));
    } else {
        left_spans.push(Span::styled(" v0.1.0", theme::dim_style()));
        left_spans.push(Span::styled("  ●", theme::success_style()));
        left_spans.push(Span::styled(" CONNECTED", theme::success_style()));
    }

    let sys_time = chrono::Local::now().format("%H:%M:%S");
    let ws_dir = std::env::current_dir()
        .map(|p| format!("WS: {}", p.display()))
        .unwrap_or_else(|_| "WS: ~".to_string());

    let mut right_spans: Vec<Span> = Vec::new();
    for (i, span) in config.extra_right.iter().enumerate() {
        if i > 0 {
            right_spans.push(Span::raw("  "));
        }
        right_spans.push(span.clone());
    }
    if !config.extra_right.is_empty() {
        right_spans.push(Span::styled("  ", theme::dim_style()));
    }
    right_spans.push(Span::styled(format!("SYS: {}", sys_time), theme::dim_style()));
    right_spans.push(Span::styled("  ", theme::dim_style()));
    right_spans.push(Span::styled(ws_dir, theme::dim_style()));

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let left_para = Paragraph::new(Line::from(left_spans));
    let right_para = Paragraph::new(Line::from(right_spans));

    f.render_widget(left_para, chunks[0]);
    f.render_widget(right_para, chunks[1]);
}
