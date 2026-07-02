use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use crate::presentation::theme::{ACCENT, BG_MAIN, CYAN, DIM_STYLE, PURPLE, TEXT_DIM, TEXT_MAIN};

pub fn render(f: &mut ratatui::Frame, area: Rect) {
    let lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(Span::styled(
            "μon",
            Style::new().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled("Deep Research Agent", Style::new().fg(PURPLE))),
        Line::from(Span::styled("v0.1.0-alpha", DIM_STYLE)),
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
        Line::from(vec![
            Span::styled("> ", Style::new().fg(TEXT_DIM)),
            Span::styled("Press ", Style::new().fg(TEXT_MAIN)),
            Span::styled("[Enter]", Style::new().fg(ACCENT)),
            Span::styled(" to start a new research query", Style::new().fg(TEXT_MAIN)),
        ]),
        Line::from(vec![
            Span::styled("> ", Style::new().fg(TEXT_DIM)),
            Span::styled("Press ", Style::new().fg(TEXT_MAIN)),
            Span::styled("[F4]", Style::new().fg(ACCENT)),
            Span::styled(" for settings", Style::new().fg(TEXT_MAIN)),
        ]),
        Line::from(vec![
            Span::styled("> ", Style::new().fg(TEXT_DIM)),
            Span::styled("Press ", Style::new().fg(TEXT_MAIN)),
            Span::styled("[?]", Style::new().fg(ACCENT)),
            Span::styled(" for help", Style::new().fg(TEXT_MAIN)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("muon-agent ", Style::new().fg(ACCENT)),
            Span::styled(": ~ $ ", Style::new().fg(TEXT_DIM)),
            Span::styled("awaiting input", Style::new().fg(CYAN)),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(Block::default().style(Style::new().bg(BG_MAIN)));

    f.render_widget(paragraph, area);
}
