use crate::presentation::theme::{ACCENT, BORDER, CYAN, ERROR, PURPLE, SUCCESS, TEXT_DIM, TEXT_MAIN, WARNING};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub fn render(f: &mut ratatui::Frame, area: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let top_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[0]);

    let bot_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);

    render_pipeline_knobs(f, top_cols[0]);
    render_compaction(f, top_cols[1]);
    render_storage(f, bot_cols[0]);
    render_embedding(f, bot_cols[1]);
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

fn numeric_row<'a>(label: &'a str, value: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("{:<24}", label), Style::new().fg(TEXT_MAIN)),
        Span::styled("[", Style::new().fg(TEXT_DIM)),
        Span::styled(value, Style::new().fg(SUCCESS)),
        Span::styled("]", Style::new().fg(TEXT_DIM)),
    ])
}

fn checkbox_row<'a>(label: &'a str, hint: &'a str, checked: bool) -> Line<'a> {
    let mark = if checked { "[✓]" } else { "[ ]" };
    let mark_color = if checked { SUCCESS } else { TEXT_DIM };
    Line::from(vec![
        Span::styled(format!("{:<24}", label), Style::new().fg(TEXT_MAIN)),
        Span::styled(mark, Style::new().fg(mark_color)),
        Span::styled(" ", Style::new().fg(TEXT_DIM)),
        Span::styled(hint, Style::new().fg(TEXT_DIM)),
    ])
}

fn render_pipeline_knobs(f: &mut ratatui::Frame, area: Rect) {
    let block = section_block("PIPELINE KNOB SETTINGS");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let lines: Vec<Line> = vec![
        numeric_row("Max Researcher Loops", "2"),
        numeric_row("Max Clarifier Turns", "3"),
        numeric_row("Max Plan Iterations", "10"),
        numeric_row("Max Shallow Turns", "10"),
        numeric_row("Max Deep Turns", "25"),
        checkbox_row("Escalate Agent", "Enable Shallow -> Deep escalation", true),
        checkbox_row("Plan Approval", "Enable Clarifier approval gates", true),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

fn render_compaction(f: &mut ratatui::Frame, area: Rect) {
    let block = section_block("COMPACTION & PREAMBLE");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(4), Constraint::Length(1)])
        .split(inner);

    // Compaction ratio with help text
    let ratio_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(BORDER));
    let ratio_inner = ratio_block.inner(chunks[0]);
    f.render_widget(ratio_block, chunks[0]);

    let ratio_lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("Compaction Threshold  ", Style::new().fg(TEXT_MAIN)),
            Span::styled("0.80", Style::new().fg(SUCCESS)),
        ]),
        Line::from(Span::styled(
            "0.0-1.0: context fill ratio that triggers message summary/compaction",
            Style::new().fg(TEXT_DIM),
        )),
    ];
    f.render_widget(Paragraph::new(ratio_lines), ratio_inner);

    // Agent preamble textarea (in a bordered box)
    let preamble_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(BORDER))
        .title(Span::styled(
            " Agent Preamble ",
            Style::new().fg(PURPLE),
        ));
    let preamble_inner = preamble_block.inner(chunks[1]);
    f.render_widget(preamble_block, chunks[1]);

    let preamble_text = "You are \u{03BC}on, a deep research agent. Be extremely precise, \
fact-check everything, compile structured summaries, and cite sources in full \
formatting. Maintain terminal safety.";
    let preamble = Paragraph::new(preamble_text)
        .style(Style::new().fg(TEXT_MAIN))
        .wrap(Wrap { trim: false });
    f.render_widget(preamble, preamble_inner);

    // Help text under textarea
    f.render_widget(
        Paragraph::new(Span::styled(
            "Default system prompt injected for all agent instances.",
            Style::new().fg(TEXT_DIM),
        )),
        chunks[2],
    );
}

fn render_storage(f: &mut ratatui::Frame, area: Rect) {
    let block = section_block("STORAGE CONFIGURATION");
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

    f.render_widget(
        Paragraph::new(path_with_browse("Session DB Path", "~/.local/share/muon/sessions.db")),
        chunks[0],
    );
    f.render_widget(
        Paragraph::new(path_with_browse("RAG DB Path", "~/.local/share/muon/rag.db")),
        chunks[1],
    );
    f.render_widget(
        Paragraph::new(numeric_row("Max search items", "15")),
        chunks[2],
    );

    let help = Span::styled(
        "Max retrieval items returned per search provider query.",
        Style::new().fg(TEXT_DIM),
    );
    f.render_widget(Paragraph::new(help), chunks[3]);
}

fn path_with_browse<'a>(label: &'a str, value: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("{:<16}", label), Style::new().fg(TEXT_MAIN)),
        Span::styled(value, Style::new().fg(CYAN)),
        Span::styled("  ", Style::new().fg(TEXT_DIM)),
        Span::styled("[Browse]", Style::new().fg(ACCENT)),
    ])
}

fn render_embedding(f: &mut ratatui::Frame, area: Rect) {
    let block = section_block("EMBEDDING & RAG");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    // Embedding Model dropdown
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(format!("{:<24}", "Embedding Model"), Style::new().fg(TEXT_MAIN)),
            Span::styled("[", Style::new().fg(TEXT_DIM)),
            Span::styled("Xenova/bge-small-en-v1.5", Style::new().fg(CYAN)),
            Span::styled(" \u{25BC}", Style::new().fg(TEXT_DIM)),
            Span::styled("]", Style::new().fg(TEXT_DIM)),
        ])),
        chunks[0],
    );

    f.render_widget(Paragraph::new(numeric_row("RAG Top-K", "5")), chunks[1]);
    f.render_widget(Paragraph::new(numeric_row("Similarity Threshold", "0.70")), chunks[2]);
    f.render_widget(
        Paragraph::new(Span::styled(
            "Cosine distance similarity threshold for retrieving text chunks.",
            Style::new().fg(TEXT_DIM),
        )),
        chunks[3],
    );

    // Rebuild Index button (destructive)
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("[ Rebuild Index ]", Style::new().fg(WARNING).add_modifier(Modifier::BOLD)),
        ])),
        chunks[5],
    );
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Warning: ", Style::new().fg(ERROR).add_modifier(Modifier::BOLD)),
            Span::styled(
                "Clears vector tables and re-embeds all active sources.",
                Style::new().fg(TEXT_DIM),
            ),
        ])),
        chunks[6],
    );
}
