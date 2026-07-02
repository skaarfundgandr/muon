use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use crate::presentation::theme::{ACCENT, DIM_STYLE, HEADER_STYLE, SUCCESS_STYLE};

pub fn render(f: &mut ratatui::Frame, area: Rect) {
    let block = Block::default().style(HEADER_STYLE);
    f.render_widget(block, area);

    let inner_w = area.width.saturating_sub(1);
    let inner = Rect::new(area.x + 1, area.y, inner_w, area.height);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    let left = Line::from(vec![
        Span::styled("μon // Deep Research Agent", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Span::styled(" v0.1.0", DIM_STYLE),
        Span::styled("  ●", SUCCESS_STYLE),
        Span::styled(" CONNECTED", SUCCESS_STYLE),
    ]);

    let sys_time = chrono::Local::now().format("%H:%M:%S");
    let ws_dir = std::env::current_dir()
        .map(|p| format!("WS: {}", p.display()))
        .unwrap_or_else(|_| "WS: ~".to_string());

    let right = Line::from(vec![
        Span::styled(format!("SYS: {}", sys_time), DIM_STYLE),
        Span::styled("  ", DIM_STYLE),
        Span::styled(ws_dir, DIM_STYLE),
    ]);

    let left_paragraph = Paragraph::new(left).alignment(Alignment::Left);
    let right_paragraph = Paragraph::new(right).alignment(Alignment::Right);

    let center_area = Rect::new(chunks[0].x, chunks[0].y, chunks[0].width + chunks[1].width, 1);
    let center_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(center_area);

    f.render_widget(left_paragraph, center_chunks[0]);
    f.render_widget(right_paragraph, center_chunks[1]);
}
