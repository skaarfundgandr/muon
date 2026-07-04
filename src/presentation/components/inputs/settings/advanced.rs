use crate::config::AdvancedConfig;
use crate::presentation::form::{FieldDef, FormState};
use crate::presentation::theme::{ACCENT, BORDER, BORDER_FOCUS, CYAN, ERROR, PURPLE, SUCCESS, TEXT_DIM, TEXT_MAIN, WARNING};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

const EMBEDDING_MODELS: &[&str] = &[
    "Xenova/bge-small-en-v1.5",
    "Xenova/bge-base-en-v1.5",
    "Xenova/bge-large-en-v1.5",
    "text-embedding-3-small",
    "text-embedding-3-large",
];

pub fn fields() -> &'static [FieldDef] {
    Box::leak(Box::new([
        // Pipeline Knobs (0-6)
        FieldDef::number("Max Researcher Loops"),
        FieldDef::number("Max Clarifier Turns"),
        FieldDef::number("Max Plan Iterations"),
        FieldDef::number("Max Shallow Turns"),
        FieldDef::number("Max Deep Turns"),
        FieldDef::checkbox("Escalate Agent"),
        FieldDef::checkbox("Plan Approval"),
        // Compaction & Preamble (7-8)
        FieldDef::number("Compaction Threshold"),
        FieldDef::text("Agent Preamble"),
        // Storage (9-11)
        FieldDef::text("Session DB Path"),
        FieldDef::text("RAG DB Path"),
        FieldDef::number("Max search items"),
        // Embedding & RAG (12-14)
        FieldDef::dropdown("Embedding Model", EMBEDDING_MODELS),
        FieldDef::number("RAG Top-K"),
        FieldDef::number("Similarity Threshold"),
    ])) as &'static [FieldDef]
}

pub fn get_field(config: &AdvancedConfig, index: usize) -> String {
    match index {
        0 => config.max_researcher_loops.to_string(),
        1 => config.max_clarifier_turns.to_string(),
        2 => config.max_plan_iterations.to_string(),
        3 => config.max_shallow_turns.to_string(),
        4 => config.max_deep_turns.to_string(),
        5 => config.escalate_agent.to_string(),
        6 => config.plan_approval.to_string(),
        7 => config.compaction_threshold.to_string(),
        8 => config.agent_preamble.clone(),
        9 => config.session_db_path.clone(),
        10 => config.rag_db_path.clone(),
        11 => config.max_search_items.to_string(),
        12 => config.embedding_model.clone(),
        13 => config.rag_top_k.to_string(),
        14 => config.similarity_threshold.to_string(),
        _ => String::new(),
    }
}

pub fn set_field(config: &mut AdvancedConfig, index: usize, value: &str) {
    match index {
        0 => config.max_researcher_loops = value.parse().unwrap_or(2),
        1 => config.max_clarifier_turns = value.parse().unwrap_or(3),
        2 => config.max_plan_iterations = value.parse().unwrap_or(10),
        3 => config.max_shallow_turns = value.parse().unwrap_or(10),
        4 => config.max_deep_turns = value.parse().unwrap_or(25),
        5 => config.escalate_agent = value == "true",
        6 => config.plan_approval = value == "true",
        7 => config.compaction_threshold = value.parse().ok().unwrap_or(0.80),
        8 => config.agent_preamble = value.to_string(),
        9 => config.session_db_path = value.to_string(),
        10 => config.rag_db_path = value.to_string(),
        11 => config.max_search_items = value.parse().unwrap_or(15),
        12 => config.embedding_model = value.to_string(),
        13 => config.rag_top_k = value.parse().unwrap_or(5),
        14 => config.similarity_threshold = value.parse().ok().unwrap_or(0.70),
        _ => {}
    }
}

pub fn toggle_field(config: &mut AdvancedConfig, index: usize) {
    match index {
        5 => config.escalate_agent = !config.escalate_agent,
        6 => config.plan_approval = !config.plan_approval,
        _ => {}
    }
}

fn is_focused(form: &FormState, index: usize) -> bool {
    form.focus == index
}

fn section_has_focus(form: &FormState, start: usize, end: usize) -> bool {
    (start..=end).any(|i| is_focused(form, i))
}

pub fn render(f: &mut ratatui::Frame, area: Rect, config: &AdvancedConfig, form: &FormState) {
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

    render_pipeline_knobs(f, top_cols[0], config, form);
    render_compaction(f, top_cols[1], config, form);
    render_storage(f, bot_cols[0], config, form);
    render_embedding(f, bot_cols[1], config, form);
}

fn section_block<'a>(title: &'a str, focused: bool) -> Block<'a> {
    let border_color = if focused { BORDER_FOCUS } else { BORDER };
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(border_color))
        .title(Span::styled(
            format!(" {} ", title),
            Style::new()
                .fg(PURPLE)
                .add_modifier(Modifier::BOLD),
        ))
}

fn numeric_row<'a>(label: &'a str, value: &'a str, focused: bool) -> Line<'a> {
    if focused {
        Line::from(vec![
            Span::styled("> ", Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:<24}", label), Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)),
            Span::styled("[", Style::new().fg(BORDER_FOCUS)),
            Span::styled(value, Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)),
            Span::styled("]", Style::new().fg(BORDER_FOCUS)),
        ])
    } else {
        Line::from(vec![
            Span::styled(format!("{:<24}", label), Style::new().fg(TEXT_MAIN)),
            Span::styled("[", Style::new().fg(TEXT_DIM)),
            Span::styled(value, Style::new().fg(SUCCESS)),
            Span::styled("]", Style::new().fg(TEXT_DIM)),
        ])
    }
}

fn checkbox_row<'a>(label: &'a str, hint: &'a str, checked: bool, focused: bool) -> Line<'a> {
    let mark = if checked { "[\u{2713}]" } else { "[ ]" };
    let mark_color = if checked { SUCCESS } else { TEXT_DIM };
    if focused {
        Line::from(vec![
            Span::styled("> ", Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:<24}", label), Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)),
            Span::styled(mark, Style::new().fg(BORDER_FOCUS)),
            Span::styled(" ", Style::new().fg(BORDER_FOCUS)),
            Span::styled(hint, Style::new().fg(BORDER_FOCUS)),
        ])
    } else {
        Line::from(vec![
            Span::styled(format!("{:<24}", label), Style::new().fg(TEXT_MAIN)),
            Span::styled(mark, Style::new().fg(mark_color)),
            Span::styled(" ", Style::new().fg(TEXT_DIM)),
            Span::styled(hint, Style::new().fg(TEXT_DIM)),
        ])
    }
}

fn dropdown_line<'a>(label: &'a str, value: &'a str, focused: bool) -> Line<'a> {
    if focused {
        Line::from(vec![
            Span::styled("> ", Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:<24}", label), Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)),
            Span::styled("[", Style::new().fg(BORDER_FOCUS)),
            Span::styled(value, Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)),
            Span::styled(" \u{25BC}", Style::new().fg(BORDER_FOCUS)),
            Span::styled("]", Style::new().fg(BORDER_FOCUS)),
        ])
    } else {
        Line::from(vec![
            Span::styled(format!("{:<24}", label), Style::new().fg(TEXT_MAIN)),
            Span::styled("[", Style::new().fg(TEXT_DIM)),
            Span::styled(value, Style::new().fg(CYAN)),
            Span::styled(" \u{25BC}", Style::new().fg(TEXT_DIM)),
            Span::styled("]", Style::new().fg(TEXT_DIM)),
        ])
    }
}


fn render_pipeline_knobs(
    f: &mut ratatui::Frame,
    area: Rect,
    config: &AdvancedConfig,
    form: &FormState,
) {
    let focused = section_has_focus(form, 0, 6);
    let block = section_block("PIPELINE KNOB SETTINGS", focused);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let s1 = config.max_researcher_loops.to_string();
    let s2 = config.max_clarifier_turns.to_string();
    let s3 = config.max_plan_iterations.to_string();
    let s4 = config.max_shallow_turns.to_string();
    let s5 = config.max_deep_turns.to_string();

    let lines: Vec<Line> = vec![
        numeric_row("Max Researcher Loops", &s1, is_focused(form, 0)),
        numeric_row("Max Clarifier Turns", &s2, is_focused(form, 1)),
        numeric_row("Max Plan Iterations", &s3, is_focused(form, 2)),
        numeric_row("Max Shallow Turns", &s4, is_focused(form, 3)),
        numeric_row("Max Deep Turns", &s5, is_focused(form, 4)),
        checkbox_row(
            "Escalate Agent",
            "Enable Shallow -> Deep escalation",
            config.escalate_agent,
            is_focused(form, 5),
        ),
        checkbox_row(
            "Plan Approval",
            "Enable Clarifier approval gates",
            config.plan_approval,
            is_focused(form, 6),
        ),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

fn render_compaction(
    f: &mut ratatui::Frame,
    area: Rect,
    config: &AdvancedConfig,
    form: &FormState,
) {
    let focused = section_has_focus(form, 7, 8);
    let block = section_block("COMPACTION & PREAMBLE", focused);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(4),
            Constraint::Length(1),
        ])
        .split(inner);

    let ratio_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(if is_focused(form, 7) { BORDER_FOCUS } else { BORDER }));
    let ratio_inner = ratio_block.inner(chunks[0]);
    f.render_widget(ratio_block, chunks[0]);

    let threshold_str = config.compaction_threshold.to_string();
    let ratio_lines: Vec<Line> = vec![
        numeric_row("Compaction Threshold", &threshold_str, is_focused(form, 7)),
        Line::from(Span::styled(
            "0.0-1.0: context fill ratio that triggers message summary/compaction",
            Style::new().fg(TEXT_DIM),
        )),
    ];
    f.render_widget(Paragraph::new(ratio_lines), ratio_inner);

    let preamble_border = if is_focused(form, 8) { BORDER_FOCUS } else { BORDER };
    let preamble_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(preamble_border))
        .title(Span::styled(
            " Agent Preamble ",
            Style::new().fg(PURPLE),
        ));
    let preamble_inner = preamble_block.inner(chunks[1]);
    f.render_widget(preamble_block, chunks[1]);

    let preamble_style = if is_focused(form, 8) {
        Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(TEXT_MAIN)
    };
    let preamble = Paragraph::new(config.agent_preamble.as_str())
        .style(preamble_style)
        .wrap(Wrap { trim: false });
    f.render_widget(preamble, preamble_inner);

    f.render_widget(
        Paragraph::new(Span::styled(
            "Default system prompt injected for all agent instances.",
            Style::new().fg(TEXT_DIM),
        )),
        chunks[2],
    );
}

fn render_storage(
    f: &mut ratatui::Frame,
    area: Rect,
    config: &AdvancedConfig,
    form: &FormState,
) {
    let focused = section_has_focus(form, 9, 11);
    let block = section_block("STORAGE CONFIGURATION", focused);
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

    let path1_prefix = if is_focused(form, 9) { "> " } else { "  " };
    let path1_label_style = if is_focused(form, 9) {
        Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(TEXT_MAIN)
    };
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(path1_prefix, Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:<16}", "Session DB Path"), path1_label_style),
            Span::styled(&config.session_db_path, if is_focused(form, 9) { Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD) } else { Style::new().fg(CYAN) }),
            Span::styled("  ", Style::new().fg(TEXT_DIM)),
            Span::styled("[Browse]", Style::new().fg(ACCENT)),
        ])),
        chunks[0],
    );

    let path2_prefix = if is_focused(form, 10) { "> " } else { "  " };
    let path2_label_style = if is_focused(form, 10) {
        Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(TEXT_MAIN)
    };
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(path2_prefix, Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:<16}", "RAG DB Path"), path2_label_style),
            Span::styled(&config.rag_db_path, if is_focused(form, 10) { Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD) } else { Style::new().fg(CYAN) }),
            Span::styled("  ", Style::new().fg(TEXT_DIM)),
            Span::styled("[Browse]", Style::new().fg(ACCENT)),
        ])),
        chunks[1],
    );

    let max_search_str = config.max_search_items.to_string();
    f.render_widget(
        Paragraph::new(numeric_row("Max search items", &max_search_str, is_focused(form, 11))),
        chunks[2],
    );

    let help = Span::styled(
        "Max retrieval items returned per search provider query.",
        Style::new().fg(TEXT_DIM),
    );
    f.render_widget(Paragraph::new(help), chunks[3]);
}

fn render_embedding(
    f: &mut ratatui::Frame,
    area: Rect,
    config: &AdvancedConfig,
    form: &FormState,
) {
    let focused = section_has_focus(form, 12, 14);
    let block = section_block("EMBEDDING & RAG", focused);
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

    f.render_widget(
        Paragraph::new(dropdown_line("Embedding Model", &config.embedding_model, is_focused(form, 12))),
        chunks[0],
    );

    let rag_top_k_str = config.rag_top_k.to_string();
    let sim_thresh_str = config.similarity_threshold.to_string();
    f.render_widget(
        Paragraph::new(numeric_row("RAG Top-K", &rag_top_k_str, is_focused(form, 13))),
        chunks[1],
    );
    f.render_widget(
        Paragraph::new(numeric_row("Similarity Threshold", &sim_thresh_str, is_focused(form, 14))),
        chunks[2],
    );
    f.render_widget(
        Paragraph::new(Span::styled(
            "Cosine distance similarity threshold for retrieving text chunks.",
            Style::new().fg(TEXT_DIM),
        )),
        chunks[3],
    );

    f.render_widget(
        Paragraph::new(Line::from(vec![Span::styled(
            "[ Rebuild Index ]",
            Style::new()
                .fg(WARNING)
                .add_modifier(Modifier::BOLD),
        )])),
        chunks[5],
    );
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                "Warning: ",
                Style::new()
                    .fg(ERROR)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Clears vector tables and re-embeds all active sources.",
                Style::new().fg(TEXT_DIM),
            ),
        ])),
        chunks[6],
    );
}
