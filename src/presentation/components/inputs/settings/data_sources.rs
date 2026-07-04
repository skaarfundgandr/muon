use crate::config::DataSourcesConfig;
use crate::presentation::click::{ClickAction, ClickTarget};
use crate::presentation::form::{FieldDef, FormState};
use crate::presentation::theme::{
    ACCENT, BORDER, BORDER_FOCUS, CYAN, ERROR, PURPLE, SUCCESS, TEXT_DIM, TEXT_MAIN, WARNING,
};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn fields() -> &'static [FieldDef] {
    Box::leak(Box::new([
        FieldDef::checkbox("Web Search"),
        FieldDef::checkbox("Paper Search"),
        FieldDef::checkbox("Enterprise Systems"),
        FieldDef::checkbox("Knowledge Layer (RAG)"),
        FieldDef::text("Source Path"),
        FieldDef::dropdown("Source Type", &["Directory", "File", "Glob"]),
        FieldDef::button("[ + Add ]"),
    ])) as &'static [FieldDef]
}

pub fn get_field(config: &DataSourcesConfig, index: usize) -> String {
    match index {
        0 => config.web_search.to_string(),
        1 => config.paper_search.to_string(),
        2 => config.enterprise_systems.to_string(),
        3 => config.knowledge_layer_rag.to_string(),
        _ => String::new(),
    }
}

pub fn set_field(config: &mut DataSourcesConfig, index: usize, value: &str) {
    match index {
        0 => config.web_search = value == "true",
        1 => config.paper_search = value == "true",
        2 => config.enterprise_systems = value == "true",
        3 => config.knowledge_layer_rag = value == "true",
        _ => {}
    }
}

pub fn toggle_field(config: &mut DataSourcesConfig, index: usize) {
    match index {
        0 => config.web_search = !config.web_search,
        1 => config.paper_search = !config.paper_search,
        2 => config.enterprise_systems = !config.enterprise_systems,
        3 => config.knowledge_layer_rag = !config.knowledge_layer_rag,
        _ => {}
    }
}

fn is_focused(form: &FormState, index: usize) -> bool {
    form.focus == index
}

#[allow(clippy::vec_init_then_push)]
pub fn render(
    f: &mut ratatui::Frame,
    area: Rect,
    config: &DataSourcesConfig,
    form: &FormState,
    hit_registry: &mut Vec<ClickTarget>,
) {
    let grid = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_source_providers(f, grid[0], config, form, hit_registry);
    render_rag_indexes(f, grid[1], form, hit_registry);
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

#[allow(clippy::vec_init_then_push)]
fn render_source_providers(
    f: &mut ratatui::Frame,
    area: Rect,
    config: &DataSourcesConfig,
    form: &FormState,
    hit_registry: &mut Vec<ClickTarget>,
) {
    let block = section_block(
        "SOURCE PROVIDERS",
        is_focused(form, 0) || is_focused(form, 1) || is_focused(form, 2) || is_focused(form, 3),
    );
    let inner = block.inner(area);
    f.render_widget(block, area);

    hit_registry.push(ClickTarget {
        rect: area,
        action: ClickAction::FocusField(0),
    });

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
        ])
        .split(inner);

    for (i, row_rect) in rows.iter().enumerate() {
        hit_registry.push(ClickTarget {
            rect: *row_rect,
            action: ClickAction::ActivateField(i),
        });
    }

    let cards: &[(&str, &str, bool)] = &[
        (
            "Web Search",
            "Access live search engines for query results",
            config.web_search,
        ),
        (
            "Paper Search",
            "Search Semantic Scholar & ArXiv archives",
            config.paper_search,
        ),
        (
            "Enterprise Systems",
            "Index internal Wikis, Confluence & Slack channels",
            config.enterprise_systems,
        ),
        (
            "Knowledge Layer (RAG)",
            "Search locally embedded directories & files",
            config.knowledge_layer_rag,
        ),
    ];

    for (i, (title, desc, enabled)) in cards.iter().enumerate() {
        let focused = is_focused(form, i);
        let status_text = if *enabled {
            "\u{2713} ENABLED"
        } else {
            "\u{25CB} DISABLED"
        };
        let status_color = if *enabled { SUCCESS } else { TEXT_DIM };
        let checkbox_str = if *enabled { "[\u{2713}]" } else { "[ ]" };
        let checkbox_color = if *enabled { SUCCESS } else { TEXT_DIM };

        let prefix = if focused { "> " } else { "  " };
        let title_style = if focused {
            Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(TEXT_MAIN).add_modifier(Modifier::BOLD)
        };

        let card_lines = vec![
            Line::from(vec![
                Span::styled(prefix, Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("[{}] ", i + 1),
                    Style::new().fg(TEXT_DIM),
                ),
                Span::styled(*title, title_style),
                Span::raw("  "),
                Span::styled(status_text, Style::new().fg(status_color)),
                Span::raw("  "),
                Span::styled(checkbox_str, Style::new().fg(checkbox_color)),
            ]),
            Line::from(vec![Span::styled(
                format!("      {}", desc),
                Style::new().fg(TEXT_DIM),
            )]),
        ];

        f.render_widget(Paragraph::new(card_lines), rows[i]);
    }
}

#[allow(clippy::vec_init_then_push)]
fn render_rag_indexes(f: &mut ratatui::Frame, area: Rect, form: &FormState, hit_registry: &mut Vec<ClickTarget>) {
    let block = section_block(
        "KNOWLEDGE LAYER / RAG INDEXES",
        is_focused(form, 4) || is_focused(form, 5) || is_focused(form, 6),
    );
    let inner = block.inner(area);
    f.render_widget(block, area);

    hit_registry.push(ClickTarget {
        rect: area,
        action: ClickAction::FocusField(4),
    });

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(inner);

    render_add_source_form(f, sections[0], form, hit_registry);
    render_source_table(f, sections[1]);
}

#[allow(clippy::vec_init_then_push)]
fn render_add_source_form(f: &mut ratatui::Frame, area: Rect, form: &FormState, hit_registry: &mut Vec<ClickTarget>) {
    let form_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(55),
            Constraint::Percentage(30),
            Constraint::Percentage(15),
        ])
        .split(area);

    hit_registry.push(ClickTarget {
        rect: form_cols[0],
        action: ClickAction::ActivateField(4),
    });
    hit_registry.push(ClickTarget {
        rect: form_cols[1],
        action: ClickAction::ActivateField(5),
    });
    hit_registry.push(ClickTarget {
        rect: form_cols[2],
        action: ClickAction::ActivateField(6),
    });

    let path_prefix = if is_focused(form, 4) { "> " } else { "  " };
    let path_value: String = if is_focused(form, 4) && form.is_editing() {
        if let Some(buf) = &form.edit_buffer {
            let cur = form.edit_cursor.min(buf.len());
            format!("{}{}{}", &buf[..cur], "\u{258E}", &buf[cur..])
        } else {
            String::new()
        }
    } else {
        "~/documents/research/".to_string()
    };
    let path_value_style = if is_focused(form, 4) {
        Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(TEXT_MAIN)
    };
    let path_line = Line::from(vec![
        Span::styled(path_prefix, Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)),
        Span::styled(path_value, path_value_style),
    ]);
    f.render_widget(Paragraph::new(path_line), form_cols[0]);

    let type_prefix = if is_focused(form, 5) { "> " } else { "  " };
    let type_label_style = if is_focused(form, 5) {
        Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(TEXT_DIM)
    };
    let type_line = Line::from(vec![
        Span::styled(type_prefix, Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)),
        Span::styled("[", type_label_style),
        Span::styled("Directory\u{25BC}", if is_focused(form, 5) { Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD) } else { Style::new().fg(ACCENT) }),
        Span::styled("]", type_label_style),
    ]);
    f.render_widget(Paragraph::new(type_line), form_cols[1]);

    let btn_prefix = if is_focused(form, 6) { "> " } else { "  " };
    let btn_style = if is_focused(form, 6) {
        Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(ACCENT).add_modifier(Modifier::BOLD)
    };
    let btn_line = Line::from(vec![
        Span::styled(btn_prefix, Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)),
        Span::styled("[ + Add ]", btn_style),
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
            Style::new()
                .fg(PURPLE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:<width$}", "TYPE", width = col_type),
            Style::new()
                .fg(PURPLE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:<width$}", "STATUS", width = col_status),
            Style::new()
                .fg(PURPLE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:<width$}", "CHUNKS", width = col_chunks),
            Style::new()
                .fg(PURPLE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:<width$}", "ACTIONS", width = col_actions),
            Style::new()
                .fg(PURPLE)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let separator = Line::from(Span::styled(
        "\u{2500}".repeat(usable_w),
        Style::new().fg(BORDER),
    ));

    let rows_data: &[(&str, &str, &str, &str)] = &[
        ("~/Documents/research/", "DIR", "\u{2713} indexed", "2,841"),
        ("~/.muon/notes/", "DIR", "\u{2713} indexed", "412"),
        ("papers/*.pdf", "GLOB", "\u{25C9} indexing", "1,209"),
        ("README.md", "FILE", "\u{25CB} pending", "0"),
        ("~/.muon/sources.csv", "FILE", "\u{2713} indexed", "89"),
    ];

    let mut lines: Vec<Line> = vec![header, separator];

    for (path, kind, status, chunks) in rows_data {
        let status_color = match *status {
            "\u{2713} indexed" => SUCCESS,
            "\u{25C9} indexing" => ACCENT,
            "\u{25CB} pending" => WARNING,
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
            Span::styled("[", Style::new().fg(TEXT_DIM)),
            Span::styled("\u{21BB}", Style::new().fg(ACCENT)),
            Span::styled("] [", Style::new().fg(TEXT_DIM)),
            Span::styled("\u{00D7}", Style::new().fg(ERROR)),
            Span::styled("]", Style::new().fg(TEXT_DIM)),
        ]));
    }

    f.render_widget(Paragraph::new(lines), area);
}
