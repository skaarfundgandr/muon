use chrono::Utc;

use crate::session::{SessionSummary, format_relative_time};
use crate::presentation::theme::{BORDER, SUCCESS, TEXT_DIM, TEXT_MAIN};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(f: &mut ratatui::Frame, area: Rect, sessions: &[SessionSummary]) {
    let block = Block::default()
        .title(" RECENT SESSIONS ")
        .borders(Borders::ALL)
        .border_style(Style::new().fg(BORDER));

    let mut lines: Vec<Line> = Vec::new();

    if sessions.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "No sessions yet. Press Enter to start researching.",
            Style::new().fg(TEXT_DIM),
        )]));
    } else {
        let now = Utc::now();
        for (i, session) in sessions.iter().enumerate() {
            let dot_style = if session.is_active {
                Style::new().fg(SUCCESS).add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(TEXT_DIM)
            };

            let time_str = format_relative_time(now, session.created_at);

            lines.push(Line::from(vec![
                Span::styled("●", dot_style),
                Span::styled(
                    format!(" {} ", session.title),
                    Style::new().fg(TEXT_MAIN),
                ),
                Span::styled(time_str, Style::new().fg(TEXT_DIM)),
            ]));

            lines.push(Line::from(vec![Span::styled(
                format!("   {}", session.query),
                Style::new().fg(TEXT_DIM),
            )]));

            if i < sessions.len() - 1 {
                lines.push(Line::from(""));
            }
        }
    }

    let paragraph = Paragraph::new(lines);
    f.render_widget(block.clone(), area);
    f.render_widget(paragraph, block.inner(area));
}
