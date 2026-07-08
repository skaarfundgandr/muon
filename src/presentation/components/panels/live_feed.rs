use crate::domain::models::log_entry::{AgentTag, LogEntry, LogLevel};
use crate::presentation::theme;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub fn render(f: &mut ratatui::Frame, area: Rect, entries: &[LogEntry], scroll_offset: usize) {
    let block = Block::default()
        .title(" >_ LIVE FEED ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::border()));

    let inner_area = block.inner(area);
    let inner_height = inner_area.height as usize;
    if inner_height == 0 {
        f.render_widget(block, area);
        return;
    }

    let dim = Style::default().fg(theme::text_dim());
    let main = Style::default().fg(theme::text_main());

    if entries.is_empty() {
        let lines = vec![Line::from(vec![Span::styled("No activity yet", dim)])];
        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
        f.render_widget(block.clone(), area);
        f.render_widget(paragraph, inner_area);
        return;
    }

    let show_indicator = entries.len() > inner_height;
    let max_entries_to_show = if show_indicator {
        inner_height.saturating_sub(1)
    } else {
        inner_height
    };

    let max_scroll = entries.len().saturating_sub(max_entries_to_show);
    let scroll_offset = scroll_offset.min(max_scroll);
    let skip = entries.len().saturating_sub(max_entries_to_show).saturating_sub(scroll_offset);
    let end = entries.len().saturating_sub(scroll_offset);

    let mut visible_entries = &entries[skip..end];
    if visible_entries.len() > max_entries_to_show {
        visible_entries = &visible_entries[..max_entries_to_show];
    }

    let mut lines: Vec<Line> = visible_entries
        .iter()
        .map(|entry| {
            let ts = entry.timestamp.with_timezone(&chrono::Local).format("[%H:%M:%S] ").to_string();
            let tag_str = format!("[{}] ", entry.agent_tag.as_str().to_uppercase());
            let color = if entry.level == LogLevel::Error {
                theme::error()
            } else {
                match entry.agent_tag {
                    AgentTag::Intent => theme::purple(),
                    AgentTag::Clarify => theme::cyan(),
                    AgentTag::Plan => theme::accent(),
                    AgentTag::Search => theme::success(),
                    AgentTag::Extract => theme::warning(),
                    AgentTag::Verify => theme::success(),
                    AgentTag::Orchestrate => theme::accent(),
                    AgentTag::Sys => theme::text_dim(),
                }
            };
            let msg_style = if entry.level == LogLevel::Error {
                Style::default().fg(theme::error())
            } else {
                main
            };
            Line::from(vec![
                Span::styled(ts, dim),
                Span::styled(tag_str, Style::default().fg(color)),
                Span::styled(&entry.message, msg_style),
            ])
        })
        .collect();

    if skip > 0 {
        lines.push(Line::from(vec![
            Span::styled(" (scroll ↑↓)", dim),
        ]));
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(block.clone(), area);
    f.render_widget(paragraph, inner_area);
}
