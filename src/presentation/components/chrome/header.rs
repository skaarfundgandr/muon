use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::presentation::theme::{ACCENT, DIM_STYLE, SUCCESS, SUCCESS_STYLE, WARNING};
use crate::presentation::views::View;

pub struct HeaderConfig {
    pub badge: Option<&'static str>,
    pub extra_right: Vec<Span<'static>>,
}

impl HeaderConfig {
    pub fn for_settings(elapsed_secs: u64, dirty: bool) -> Self {
        let _ = elapsed_secs;
        let (text, color) = if dirty {
            ("UNSAVED CHANGES", WARNING)
        } else {
            ("SAVED", SUCCESS)
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
                    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                )],
            },
            View::Results => Self {
                badge: Some("RESEARCH COMPLETE"),
                extra_right: vec![Span::styled(
                    format!("TOTAL TIME: {}", format_elapsed(elapsed_secs)),
                    Style::default().fg(SUCCESS).add_modifier(Modifier::BOLD),
                )],
            },
            View::Settings => Self {
                badge: Some("CONFIGURATION CONSOLE"),
                extra_right: vec![Span::styled(
                    "UNSAVED CHANGES",
                    Style::default().fg(WARNING).add_modifier(Modifier::BOLD),
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
        Some("RESEARCH COMPLETE") => SUCCESS,
        Some("RESEARCHING [DEEP]") => ACCENT,
        Some("CONFIGURATION CONSOLE") => ACCENT,
        _ => ACCENT,
    };

    let mut left_spans: Vec<Span> = vec![Span::styled(
        "μon // Deep Research Agent",
        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
    )];

    if let Some(text) = config.badge {
        left_spans.push(Span::raw("  "));
        left_spans.push(Span::styled(
            format!(" {} ", text),
            Style::default()
                .fg(badge_color)
                .add_modifier(Modifier::BOLD),
        ));
        left_spans.push(Span::styled(" ●", SUCCESS_STYLE));
    } else {
        left_spans.push(Span::styled(" v0.1.0", DIM_STYLE));
        left_spans.push(Span::styled("  ●", SUCCESS_STYLE));
        left_spans.push(Span::styled(" CONNECTED", SUCCESS_STYLE));
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
        right_spans.push(Span::styled("  ", DIM_STYLE));
    }
    right_spans.push(Span::styled(format!("SYS: {}", sys_time), DIM_STYLE));
    right_spans.push(Span::styled("  ", DIM_STYLE));
    right_spans.push(Span::styled(ws_dir, DIM_STYLE));

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let left_para = Paragraph::new(Line::from(left_spans));
    let right_para = Paragraph::new(Line::from(right_spans));

    f.render_widget(left_para, chunks[0]);
    f.render_widget(right_para, chunks[1]);
}
