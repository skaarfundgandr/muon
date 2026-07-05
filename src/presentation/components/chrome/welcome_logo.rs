use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::presentation::theme;

const HINT_LINES: &[(&str, &str)] = &[
    ("Enter", "to start a new research query"),
    ("Tab", "to cycle views"),
    ("?", "for help"),
];

pub fn render(f: &mut ratatui::Frame, area: Rect) {
    f.render_widget(Block::default().style(Style::default().bg(theme::bg_main())), area);

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let content_height: u16 = 17;
    let vertical_pad = outer[0].height.saturating_sub(content_height) / 2;
    let content_area = Rect {
        x: outer[0].x,
        y: outer[0].y + vertical_pad,
        width: outer[0].width,
        height: content_height.min(outer[0].height),
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(6),
            Constraint::Length(1),
        ])
        .split(content_area);

    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "μon",
            Style::new().fg(theme::accent()).add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Center),
        chunks[0],
    );

    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "Deep Research Agent",
            Style::new().fg(theme::purple()).add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Center),
        chunks[1],
    );

    f.render_widget(
        Paragraph::new(Line::from(Span::styled("v0.1.0-alpha", theme::dim_style())))
            .alignment(Alignment::Center),
        chunks[2],
    );

    f.render_widget(
        Paragraph::new(Line::from("")).alignment(Alignment::Center),
        chunks[3],
    );

    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "Terminal-based multi-agent research system",
            Style::new().fg(theme::text_main()),
        )))
        .alignment(Alignment::Center),
        chunks[4],
    );

    render_hints_box(f, chunks[6]);

    let prompt_line = Line::from(vec![
        Span::styled(
            "muon-agent",
            Style::new().fg(theme::success()).add_modifier(Modifier::BOLD),
        ),
        Span::styled(":", theme::text_dim()),
        Span::styled("~", Style::new().fg(theme::accent()).add_modifier(Modifier::BOLD)),
        Span::styled("$", theme::text_dim()),
        Span::styled(" awaiting input", theme::text_dim()),
        Span::styled("█", Style::new().fg(theme::accent()).add_modifier(Modifier::BOLD)),
    ]);
    f.render_widget(
        Paragraph::new(prompt_line).alignment(Alignment::Center),
        chunks[7],
    );
}

fn render_hints_box(f: &mut ratatui::Frame, area: Rect) {
    let max_label_len = HINT_LINES
        .iter()
        .map(|(k, _)| k.chars().count() + 2)
        .max()
        .unwrap_or(0) as u16;
    let max_text_len = HINT_LINES
        .iter()
        .map(|(_, t)| t.chars().count())
        .max()
        .unwrap_or(0) as u16;
    let inner_text_width = "Press ".len() as u16 + max_label_len + 1 + max_text_len;
    let hints_width = (inner_text_width + 6).min(area.width);
    let hints_height = HINT_LINES.len() as u16 + 2;
    let start_x = area.x + area.width.saturating_sub(hints_width) / 2;
    let start_y = area.y + area.height.saturating_sub(hints_height) / 2;

    let cell = Rect {
        x: start_x,
        y: start_y,
        width: hints_width,
        height: hints_height,
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme::border()))
        .style(Style::default().bg(theme::bg_dark()));

    let inner = block.inner(cell);
    f.render_widget(block, cell);

    let line_constraints: Vec<Constraint> =
        HINT_LINES.iter().map(|_| Constraint::Length(1)).collect();
    let line_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(line_constraints)
        .split(inner);

    for (i, (key, text)) in HINT_LINES.iter().enumerate() {
        let line = Line::from(vec![
            Span::styled("> ", Style::new().fg(theme::accent()).add_modifier(Modifier::BOLD)),
            Span::styled("Press ", Style::new().fg(theme::text_main())),
            Span::styled(
                format!("[{}]", key),
                Style::new().fg(theme::accent()).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!(" {}", text), Style::new().fg(theme::text_main())),
        ]);
        f.render_widget(Paragraph::new(line), line_chunks[i]);
    }
}
