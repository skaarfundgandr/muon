use crate::presentation::theme::{ACCENT, BORDER, CYAN, PURPLE, SUCCESS, TEXT_DIM, TEXT_MAIN, WARNING};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(f: &mut ratatui::Frame, area: Rect) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let grid = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(outer[0]);

    render_model_providers(f, grid[0]);
    render_search_providers(f, grid[1]);
    render_bottom_note(f, outer[1]);
}

fn section_block(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(BORDER))
        .title(Span::styled(
            format!(" {} ", title),
            Style::new()
                .fg(PURPLE)
                .add_modifier(Modifier::BOLD),
        ))
}

fn provider_row<'a>(
    label: &'a str,
    masked: &'a str,
    status: &'a str,
    status_color: ratatui::style::Color,
) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("{:<18}", label), Style::new().fg(TEXT_MAIN).add_modifier(Modifier::BOLD)),
        Span::styled(masked, Style::new().fg(SUCCESS)),
        Span::styled(" ", Style::new().fg(TEXT_DIM)),
        Span::styled("[Reveal]", Style::new().fg(ACCENT)),
        Span::styled(" ", Style::new().fg(TEXT_DIM)),
        Span::styled("[Test]", Style::new().fg(ACCENT)),
        Span::styled(format!(" {}", status), Style::new().fg(status_color)),
    ])
}

fn provider_row_empty<'a>(label: &'a str, placeholder: &'a str, status: &'a str, status_color: ratatui::style::Color) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("{:<18}", label), Style::new().fg(TEXT_MAIN).add_modifier(Modifier::BOLD)),
        Span::styled(placeholder, Style::new().fg(TEXT_DIM)),
        Span::styled(" ", Style::new().fg(TEXT_DIM)),
        Span::styled("[Reveal]", Style::new().fg(ACCENT)),
        Span::styled(" ", Style::new().fg(TEXT_DIM)),
        Span::styled("[Test]", Style::new().fg(ACCENT)),
        Span::styled(format!(" {}", status), Style::new().fg(status_color)),
    ])
}

fn render_model_providers(f: &mut ratatui::Frame, area: Rect) {
    let block = section_block("MODEL PROVIDERS API KEYS");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(inner);

    // OpenCode-Go
    let opencode_lines = vec![
        Line::from(vec![
            Span::styled(
                format!("{:<16}", "OpenCode-Go"),
                Style::new().fg(TEXT_MAIN).add_modifier(Modifier::BOLD),
            ),
            Span::styled("$OPENCODE_API_KEY", Style::new().fg(SUCCESS)),
        ]),
        provider_row("", "●●●●●●●●●●●●●●●●●●●●●●", "✓", SUCCESS),
        Line::from(vec![
            Span::styled("  ", Style::new().fg(TEXT_DIM)),
            Span::styled(
                "Using system environment override. File secret ignored.",
                Style::new().fg(SUCCESS),
            ),
        ]),
    ];
    f.render_widget(Paragraph::new(opencode_lines), rows[0]);

    // NeuralWatt
    let neural_lines = vec![
        Line::from(Span::styled(
            format!("{:<16}", "NeuralWatt"),
            Style::new().fg(TEXT_MAIN).add_modifier(Modifier::BOLD),
        )),
        provider_row_empty("", "Enter NeuralWatt API Key...", "⚠", WARNING),
    ];
    f.render_widget(Paragraph::new(neural_lines), rows[1]);

    // ClinePass
    let cline_lines = vec![
        Line::from(Span::styled(
            format!("{:<16}", "ClinePass"),
            Style::new().fg(TEXT_MAIN).add_modifier(Modifier::BOLD),
        )),
        provider_row("", "●●●●●●●●●●●●●●●●●●●●●●", "✓", SUCCESS),
    ];
    f.render_widget(Paragraph::new(cline_lines), rows[2]);
}

fn render_search_providers(f: &mut ratatui::Frame, area: Rect) {
    let block = section_block("SEARCH PROVIDERS CONFIGURATION");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(6),
            Constraint::Length(3),
            Constraint::Length(2),
        ])
        .split(inner);

    // Brave Search
    let brave_lines = vec![
        Line::from(vec![
            Span::styled(
                format!("{:<18}", "Brave Search"),
                Style::new().fg(TEXT_MAIN).add_modifier(Modifier::BOLD),
            ),
            Span::styled("web default", Style::new().fg(ACCENT)),
        ]),
        provider_row("", "●●●●●●●●●●●●●●●●●●", "✓", SUCCESS),
    ];
    f.render_widget(Paragraph::new(brave_lines), sections[0]);

    // SearXNG sub-panel
    let searx_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(CYAN))
        .title(Span::styled(
            " SearXNG Custom Instance ",
            Style::new().fg(CYAN),
        ));
    let searx_inner = searx_block.inner(sections[1]);
    f.render_widget(searx_block, sections[1]);

    let searx_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Min(0)])
        .split(searx_inner);

    let url_line = Line::from(vec![
        Span::styled(
            format!("{:<18}", "Instance URL"),
            Style::new().fg(TEXT_DIM),
        ),
        Span::styled("https://searxng.local", Style::new().fg(CYAN)),
    ]);
    f.render_widget(Paragraph::new(url_line), searx_rows[0]);

    let api_line = Line::from(vec![
        Span::styled(
            format!("{:<18}", "API Key"),
            Style::new().fg(TEXT_DIM),
        ),
        Span::styled("●●●●●●●●●●", Style::new().fg(SUCCESS)),
        Span::styled(" ", Style::new().fg(TEXT_DIM)),
        Span::styled("[Reveal]", Style::new().fg(ACCENT)),
        Span::styled(" ", Style::new().fg(TEXT_DIM)),
        Span::styled("[Test]", Style::new().fg(ACCENT)),
        Span::styled(" ✓", Style::new().fg(SUCCESS)),
    ]);
    f.render_widget(Paragraph::new(api_line), searx_rows[1]);

    // Semantic Scholar
    let sem_lines = vec![
        Line::from(vec![
            Span::styled(
                format!("{:<18}", "Semantic Scholar"),
                Style::new().fg(TEXT_MAIN).add_modifier(Modifier::BOLD),
            ),
            Span::styled("papers default", Style::new().fg(ACCENT)),
        ]),
        provider_row_empty("", "Enter Semantic Scholar API Key...", "⚠", WARNING),
    ];
    f.render_widget(Paragraph::new(sem_lines), sections[2]);

    // ArXiv
    let arxiv_line = Line::from(vec![
        Span::styled(
            format!("{:<18}", "ArXiv Search"),
            Style::new().fg(TEXT_MAIN).add_modifier(Modifier::BOLD),
        ),
        Span::styled("[✓] ", Style::new().fg(SUCCESS)),
        Span::styled("Enabled ", Style::new().fg(TEXT_MAIN)),
        Span::styled("(No Key Required)", Style::new().fg(SUCCESS)),
    ]);
    f.render_widget(Paragraph::new(arxiv_line), sections[3]);
}

fn render_bottom_note(f: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(TEXT_DIM));

    let note = Line::from(vec![
        Span::styled("[i] ", Style::new().fg(WARNING)),
        Span::styled("Keys are stored locally in ", Style::new().fg(TEXT_DIM)),
        Span::styled("~/.config/muon/secrets.toml", Style::new().fg(PURPLE)),
        Span::styled(
            ". Environment variables override file values.",
            Style::new().fg(TEXT_DIM),
        ),
    ]);
    f.render_widget(Paragraph::new(note).block(block), area);
}
