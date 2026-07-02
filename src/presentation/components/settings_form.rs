use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use crate::presentation::theme::{ACCENT, BORDER, PURPLE, SUCCESS, TEXT_DIM, TEXT_MAIN};

#[allow(clippy::vec_init_then_push)]
pub fn render(f: &mut ratatui::Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    let tab_bar = Line::from(vec![
        Span::styled("[Agents]", Style::new().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled("Tools", Style::new().fg(TEXT_DIM)),
        Span::raw("  "),
        Span::styled("Data Sources", Style::new().fg(TEXT_DIM)),
        Span::raw("  "),
        Span::styled("Display", Style::new().fg(TEXT_DIM)),
        Span::raw("  "),
        Span::styled("Advanced", Style::new().fg(TEXT_DIM)),
    ]);
    f.render_widget(Paragraph::new(tab_bar), chunks[0]);

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(vec![
        Span::styled(
            "1. Intent Classifier",
            Style::new().fg(PURPLE).add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Model:        ", Style::new().fg(TEXT_DIM)),
        Span::styled("[glm-5.2        ", Style::new().fg(TEXT_MAIN)),
        Span::styled("▼", Style::new().fg(ACCENT)),
        Span::styled("]", Style::new().fg(TEXT_MAIN)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Provider:    ", Style::new().fg(TEXT_DIM)),
        Span::styled("[opencode-go    ", Style::new().fg(TEXT_MAIN)),
        Span::styled("▼", Style::new().fg(ACCENT)),
        Span::styled("]", Style::new().fg(TEXT_MAIN)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Max turns:   ", Style::new().fg(TEXT_DIM)),
        Span::styled("[10]", Style::new().fg(TEXT_MAIN)),
        Span::raw("           "),
        Span::styled("Temperature: ", Style::new().fg(TEXT_DIM)),
        Span::styled("[0.1]", Style::new().fg(TEXT_MAIN)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Tools:       ", Style::new().fg(TEXT_DIM)),
        Span::styled("✗ none", Style::new().fg(TEXT_DIM)),
    ]));

    lines.push(Line::raw(""));

    lines.push(Line::from(vec![
        Span::styled(
            "2. Deep Researcher",
            Style::new().fg(PURPLE).add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Model:        ", Style::new().fg(TEXT_DIM)),
        Span::styled("[glm-5.2-flex   ", Style::new().fg(TEXT_MAIN)),
        Span::styled("▼", Style::new().fg(ACCENT)),
        Span::styled("]", Style::new().fg(TEXT_MAIN)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Provider:    ", Style::new().fg(TEXT_DIM)),
        Span::styled("[opencode-go    ", Style::new().fg(TEXT_MAIN)),
        Span::styled("▼", Style::new().fg(ACCENT)),
        Span::styled("]", Style::new().fg(TEXT_MAIN)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Max turns:   ", Style::new().fg(TEXT_DIM)),
        Span::styled("[25]", Style::new().fg(TEXT_MAIN)),
        Span::raw("           "),
        Span::styled("Temperature: ", Style::new().fg(TEXT_DIM)),
        Span::styled("[0.3]", Style::new().fg(TEXT_MAIN)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Tools:       ", Style::new().fg(TEXT_DIM)),
        Span::styled("✓ ", Style::new().fg(SUCCESS)),
        Span::styled("web_search  ", Style::new().fg(TEXT_MAIN)),
        Span::styled("✓ ", Style::new().fg(SUCCESS)),
        Span::styled("fetch  ", Style::new().fg(TEXT_MAIN)),
        Span::styled("✓ ", Style::new().fg(SUCCESS)),
        Span::styled("rag", Style::new().fg(TEXT_MAIN)),
    ]));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(BORDER))
        .title(" AGENT CONFIGURATION ");
    f.render_widget(Paragraph::new(lines).block(block), chunks[1]);
}
