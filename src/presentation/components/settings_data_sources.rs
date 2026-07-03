use crate::presentation::theme::{ACCENT, BORDER, CYAN, ERROR, PURPLE, SUCCESS, TEXT_DIM, TEXT_MAIN, WARNING};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

#[allow(clippy::vec_init_then_push)]
pub fn render(f: &mut ratatui::Frame, area: Rect) {
    let grid = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_source_providers(f, grid[0]);
    render_rag_indexes(f, grid[1]);
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

#[allow(clippy::vec_init_then_push)]
fn render_source_providers(f: &mut ratatui::Frame, area: Rect) {
    let block = section_block("SOURCE PROVIDERS");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
        ])
        .split(inner);

    let cards: &[(&str, &str, bool)] = &[
        ("Web Search", "Access live search engines for query results", true),
        ("Paper Search", "Search Semantic Scholar & ArXiv archives", true),
        ("Enterprise Systems", "Index internal Wikis, Confluence & Slack channels", false),
        ("Knowledge Layer (RAG)", "Search locally embedded directories & files", true),
    ];

    for (i, (title, desc, enabled)) in cards.iter().enumerate() {
        let status_text = if *enabled { "✓ ENABLED" } else { "○ DISABLED" };
        let status_color = if *enabled { SUCCESS } else { TEXT_DIM };
        let checkbox_str = if *enabled { "[✓]" } else { "[ ]" };
        let checkbox_color = if *enabled { SUCCESS } else { TEXT_DIM };

        let card_lines = vec![
            Line::from(vec![
                Span::styled(format!("  [{}] ", i + 1), Style::new().fg(TEXT_DIM)),
                Span::styled(*title, Style::new().fg(TEXT_MAIN).add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled(status_text, Style::new().fg(status_color)),
                Span::raw("  "),
                Span::styled(checkbox_str, Style::new().fg(checkbox_color)),
            ]),
            Line::from(vec![
                Span::styled(format!("      {}", desc), Style::new().fg(TEXT_DIM)),
            ]),
        ];

        f.render_widget(Paragraph::new(card_lines), rows[i]);
    }
}

#[allow(clippy::vec_init_then_push)]
fn render_rag_indexes(f: &mut ratatui::Frame, area: Rect) {
    let block = section_block("KNOWLEDGE LAYER / RAG INDEXES");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(inner);

    render_add_source_form(f, sections[0]);
    render_source_table(f, sections[1]);
}

#[allow(clippy::vec_init_then_push)]
fn render_add_source_form(f: &mut ratatui::Frame, area: Rect) {
    let form_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(55),
            Constraint::Percentage(30),
            Constraint::Percentage(15),
        ])
        .split(area);

    let path_line = Line::from(vec![
        Span::styled("~/documents/research/                  ", Style::new().fg(TEXT_MAIN)),
    ]);
    f.render_widget(Paragraph::new(path_line), form_cols[0]);

    let type_line = Line::from(vec![
        Span::styled("[", Style::new().fg(TEXT_DIM)),
        Span::styled("Directory▼", Style::new().fg(ACCENT)),
        Span::styled("]", Style::new().fg(TEXT_DIM)),
    ]);
    f.render_widget(Paragraph::new(type_line), form_cols[1]);

    let btn_line = Line::from(vec![
        Span::styled("[ + Add ]", Style::new().fg(ACCENT).add_modifier(Modifier::BOLD)),
    ]);
    f.render_widget(Paragraph::new(btn_line), form_cols[2]);
}

#[allow(clippy::vec_init_then_push)]
fn render_source_table(f: &mut ratatui::Frame, area: Rect) {
    let usable_w = area.width as usize;

    let col_path = ((usable_w as f64 * 0.40) as usize).max(20);
    let col_type = ((usable_w as f64 * 0.12) as usize).max(8);
    let col_status = ((usable_w as f64 * 0.18) as usize).max(12);
    let col_chunks = ((usable_w as f64 * 0.12) as usize).max(8);
    let col_actions = usable_w.saturating_sub(col_path + col_type + col_status + col_chunks);

    let header = Line::from(vec![
        Span::styled(
            format!("{:<width$}", "SOURCE PATH", width = col_path),
            Style::new().fg(PURPLE).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:<width$}", "TYPE", width = col_type),
            Style::new().fg(PURPLE).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:<width$}", "STATUS", width = col_status),
            Style::new().fg(PURPLE).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:<width$}", "CHUNKS", width = col_chunks),
            Style::new().fg(PURPLE).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:<width$}", "ACTIONS", width = col_actions),
            Style::new().fg(PURPLE).add_modifier(Modifier::BOLD),
        ),
    ]);

    let separator = Line::from(Span::styled(
        "─".repeat(usable_w),
        Style::new().fg(BORDER),
    ));

    let rows_data: &[(&str, &str, &str, &str)] = &[
        ("~/Documents/research/", "DIR", "✓ indexed", "2,841"),
        ("~/.muon/notes/", "DIR", "✓ indexed", "412"),
        ("papers/*.pdf", "GLOB", "◉ indexing", "1,209"),
        ("README.md", "FILE", "○ pending", "0"),
        ("~/.muon/sources.csv", "FILE", "✓ indexed", "89"),
    ];

    let mut lines: Vec<Line> = vec![header, separator];

    for (path, kind, status, chunks) in rows_data {
        let status_color = match *status {
            "✓ indexed" => SUCCESS,
            "◉ indexing" => ACCENT,
            "○ pending" => WARNING,
            _ => TEXT_DIM,
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!("{:<width$}", path, width = col_path),
                Style::new().fg(TEXT_MAIN),
            ),
            Span::styled(
                format!("{:<width$}", kind, width = col_type),
                Style::new().fg(CYAN),
            ),
            Span::styled(
                format!("{:<width$}", status, width = col_status),
                Style::new().fg(status_color),
            ),
            Span::styled(
                format!("{:<width$}", chunks, width = col_chunks),
                Style::new().fg(TEXT_MAIN),
            ),
            Span::styled(
                "[",
                Style::new().fg(TEXT_DIM),
            ),
            Span::styled(
                "↻",
                Style::new().fg(ACCENT),
            ),
            Span::styled(
                "] [",
                Style::new().fg(TEXT_DIM),
            ),
            Span::styled(
                "×",
                Style::new().fg(ERROR),
            ),
            Span::styled(
                "]",
                Style::new().fg(TEXT_DIM),
            ),
        ]));
    }

    f.render_widget(Paragraph::new(lines), area);
}
