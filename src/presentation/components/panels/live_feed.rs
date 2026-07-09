use crate::domain::models::log_entry::{AgentTag, LogEntry, LogLevel};
use crate::presentation::click::is_hovering;
use crate::presentation::theme;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(
    f: &mut ratatui::Frame,
    area: Rect,
    entries: &[LogEntry],
    scroll_offset: usize,
    mouse_col: u16,
    mouse_row: u16,
) {
    let hovering = is_hovering(area, mouse_col, mouse_row);
    let border_color = if hovering {
        theme::border_hover()
    } else {
        theme::border()
    };
    let block = Block::default()
        .title(" >_ LIVE FEED ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

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
        let paragraph = Paragraph::new(lines);
        f.render_widget(block.clone(), area);
        f.render_widget(paragraph, inner_area);
        return;
    }

    let width = inner_area.width.max(1) as usize;
    let msg_budget = width.saturating_sub(18).max(16);

    let mut all_lines: Vec<Line> = Vec::new();
    for entry in entries {
        let ts = entry
            .timestamp
            .with_timezone(&chrono::Local)
            .format("[%H:%M:%S] ")
            .to_string();
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
        } else if entry.level == LogLevel::Warn {
            Style::default().fg(theme::warning())
        } else {
            main
        };

        let keep_full = matches!(entry.level, LogLevel::Error | LogLevel::Warn);
        let msg = if !keep_full && entry.message.chars().count() > msg_budget {
            let mut s: String = entry
                .message
                .chars()
                .take(msg_budget.saturating_sub(1))
                .collect();
            s.push('…');
            s
        } else {
            entry.message.clone()
        };

        if keep_full {
            let prefix_len = ts.chars().count() + tag_str.chars().count();
            let first_w = width.saturating_sub(prefix_len).max(8);
            let rest = msg.as_str();
            let (head, tail) = split_at_chars(rest, first_w);
            all_lines.push(Line::from(vec![
                Span::styled(ts, dim),
                Span::styled(tag_str, Style::default().fg(color)),
                Span::styled(head, msg_style),
            ]));
            for chunk in wrap_chars(tail, width) {
                if chunk.is_empty() {
                    continue;
                }
                all_lines.push(Line::from(Span::styled(chunk, msg_style)));
            }
        } else {
            all_lines.push(Line::from(vec![
                Span::styled(ts, dim),
                Span::styled(tag_str, Style::default().fg(color)),
                Span::styled(msg, msg_style),
            ]));
        }
    }

    let show_indicator = all_lines.len() > inner_height;
    let max_lines = if show_indicator {
        inner_height.saturating_sub(1)
    } else {
        inner_height
    };

    let max_scroll = all_lines.len().saturating_sub(max_lines);
    let scroll_offset = scroll_offset.min(max_scroll);
    let skip = all_lines
        .len()
        .saturating_sub(max_lines)
        .saturating_sub(scroll_offset);
    let end = all_lines.len().saturating_sub(scroll_offset);

    let mut visible = all_lines[skip..end].to_vec();
    if visible.len() > max_lines {
        visible.truncate(max_lines);
    }
    if skip > 0 {
        visible.push(Line::from(vec![Span::styled(" (scroll ↑↓)", dim)]));
    }

    let paragraph = Paragraph::new(visible);
    f.render_widget(block.clone(), area);
    f.render_widget(paragraph, inner_area);
}

fn split_at_chars(text: &str, n: usize) -> (String, &str) {
    if n == 0 {
        return (String::new(), text);
    }
    let mut end = 0;
    let mut count = 0;
    for (i, _) in text.char_indices() {
        if count >= n {
            return (text[..i].to_string(), &text[i..]);
        }
        end = i;
        count += 1;
    }
    if count <= n {
        return (text.to_string(), "");
    }
    let _ = end;
    (text.to_string(), "")
}

fn wrap_chars(text: &str, width: usize) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }
    if width == 0 {
        return vec![text.to_string()];
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch == '\n' {
            lines.push(std::mem::take(&mut current));
            continue;
        }
        if current.chars().count() >= width {
            lines.push(std::mem::take(&mut current));
        }
        current.push(ch);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}
