use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::presentation::theme::{ACCENT, BG_MAIN, BORDER, CYAN, DIM_STYLE, PURPLE, SUCCESS, TEXT_DIM, TEXT_MAIN};

pub fn render(f: &mut ratatui::Frame, area: Rect) {
    let bg = Block::default().style(Style::default().bg(BG_MAIN));
    f.render_widget(bg, area);

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(7),
            Constraint::Length(1),
        ])
        .split(area);

    let logo_block = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .split(outer[0]);

    let content_height: u16 = 9;
    let vertical_pad = logo_block[0].height.saturating_sub(content_height) / 2;
    let logo_area = Rect {
        x: logo_block[0].x,
        y: logo_block[0].y + vertical_pad,
        width: logo_block[0].width,
        height: content_height.min(logo_block[0].height),
    };

    let logo_lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(Span::styled(
            "μon",
            Style::new().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled("Deep Research Agent", Style::new().fg(PURPLE))),
        Line::from(Span::styled("v0.1.0-alpha", DIM_STYLE)),
        Line::from(""),
        Line::from(Span::styled(
            "Terminal-based multi-agent research system",
            Style::new().fg(TEXT_MAIN),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("[", Style::new().fg(ACCENT)),
            Span::styled("4 Agents", Style::new().fg(TEXT_MAIN)),
            Span::styled("]", Style::new().fg(ACCENT)),
            Span::styled("  ", Style::new().fg(TEXT_DIM)),
            Span::styled("[", Style::new().fg(ACCENT)),
            Span::styled("Rust + Ratatui", Style::new().fg(TEXT_MAIN)),
            Span::styled("]", Style::new().fg(ACCENT)),
            Span::styled("  ", Style::new().fg(TEXT_DIM)),
            Span::styled("[", Style::new().fg(ACCENT)),
            Span::styled("Rig Framework", Style::new().fg(TEXT_MAIN)),
            Span::styled("]", Style::new().fg(ACCENT)),
        ]),
        Line::from(""),
    ];

    let logo = Paragraph::new(logo_lines).alignment(Alignment::Center);
    f.render_widget(logo, logo_area);

    let hints_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(BORDER))
        .style(Style::default().bg(BG_MAIN));

    let hint_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(hints_block.inner(outer[1]));

    f.render_widget(hints_block, outer[1]);

    let prompt_style = Style::new().fg(ACCENT).add_modifier(Modifier::BOLD);
    let key_style = Style::new().fg(ACCENT).add_modifier(Modifier::BOLD);

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("> ", prompt_style),
            Span::styled("Press ", Style::new().fg(TEXT_MAIN)),
            Span::styled("[Enter]", key_style),
            Span::styled(" to start a new research query", Style::new().fg(TEXT_MAIN)),
        ])),
        hint_chunks[0],
    );

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("> ", prompt_style),
            Span::styled("Press ", Style::new().fg(TEXT_MAIN)),
            Span::styled("[F4]", key_style),
            Span::styled(" for settings", Style::new().fg(TEXT_MAIN)),
        ])),
        hint_chunks[1],
    );

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("> ", prompt_style),
            Span::styled("Press ", Style::new().fg(TEXT_MAIN)),
            Span::styled("[?]", key_style),
            Span::styled(" for help", Style::new().fg(TEXT_MAIN)),
        ])),
        hint_chunks[2],
    );

    let prompt_line = Line::from(vec![
        Span::styled("       ", Style::new().fg(TEXT_DIM)),
        Span::styled("muon-agent ", Style::new().fg(SUCCESS).add_modifier(Modifier::BOLD)),
        Span::styled(": ~ $ ", Style::new().fg(TEXT_DIM)),
        Span::styled("awaiting input", Style::new().fg(CYAN)),
        Span::styled("█", Style::new().fg(ACCENT)),
    ]);
    f.render_widget(
        Paragraph::new(prompt_line).alignment(Alignment::Center),
        outer[2],
    );
}
