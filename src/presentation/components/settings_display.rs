use crate::presentation::theme::{BORDER, CYAN, PURPLE, SUCCESS, TEXT_DIM, TEXT_MAIN, WARNING};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(f: &mut ratatui::Frame, area: Rect) {
    let grid = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_left(f, grid[0]);
    render_right(f, grid[1]);
}

fn section_block<'a>(title: &'a str) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(BORDER))
        .title(Span::styled(
            format!(" {} ", title),
            Style::new().fg(PURPLE).add_modifier(Modifier::BOLD),
        ))
}

fn dropdown_line<'a>(label: &'a str, value: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("{:<14}", label), Style::new().fg(TEXT_DIM)),
        Span::styled("[", Style::new().fg(TEXT_DIM)),
        Span::styled(value, Style::new().fg(TEXT_MAIN)),
        Span::styled("▼", Style::new().fg(TEXT_DIM)),
        Span::styled("]", Style::new().fg(TEXT_DIM)),
    ])
}

fn info_row<'a>(label: &'a str, value: &'a str, val_style: Style) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("{:<22}", label), Style::new().fg(TEXT_DIM)),
        Span::styled(value, val_style.add_modifier(Modifier::BOLD)),
    ])
}

fn render_left(f: &mut ratatui::Frame, area: Rect) {
    let block = section_block("TERMINAL DISPLAY SETTINGS");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    f.render_widget(dropdown_line("Visual Theme", "Tokyo Night"), chunks[0]);
    f.render_widget(dropdown_line("Font Size", "Medium 14px"), chunks[1]);
    f.render_widget(
        Paragraph::new(Span::styled("Live Preview", Style::new().fg(TEXT_DIM))),
        chunks[2],
    );

    let preview_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(BORDER));
    let preview_inner = preview_block.inner(chunks[3]);
    f.render_widget(preview_block, chunks[3]);

    let preview_lines: Vec<Line> = vec![
        Line::from(Span::styled(
            "The quick brown fox jumps over the lazy dog.",
            Style::new().fg(TEXT_MAIN),
        )),
        Line::from(Span::styled(
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            Style::new().fg(TEXT_MAIN),
        )),
        Line::from(Span::styled(
            "abcdefghijklmnopqrstuvwxyz",
            Style::new().fg(TEXT_MAIN),
        )),
        Line::from(Span::styled(
            "0123456789",
            Style::new().fg(TEXT_MAIN),
        )),
    ];
    f.render_widget(Paragraph::new(preview_lines), preview_inner);
}

fn render_right(f: &mut ratatui::Frame, area: Rect) {
    let block = section_block("STATUS BAR & ENVIRONMENT INFO (READ-ONLY)");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let lines: Vec<Line> = vec![
        info_row("Active Renderer:", "HTML TUI Emulator (Bex/Ratatui Mock)", Style::new().fg(CYAN)),
        info_row("Font Stack:", "Tokyo Night / 14px / JetBrains Mono", Style::new().fg(PURPLE)),
        info_row("Terminal Encoding:", "UTF-8 / Unicode Standard", Style::new().fg(SUCCESS)),
        info_row("Color Standard:", "True Color (24-bit RGB)", Style::new().fg(WARNING)),
        info_row("Window Size:", "1200 x 800 (Simulated Viewport)", Style::new().fg(CYAN)),
    ];
    f.render_widget(Paragraph::new(lines), inner);
}
