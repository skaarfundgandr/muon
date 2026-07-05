use crate::config::ToolsConfig;
use crate::presentation::click::{ClickAction, ClickTarget};
use crate::presentation::form::{FieldDef, FormState};
use crate::presentation::theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn fields() -> &'static [FieldDef] {
    Box::leak(Box::new([
        FieldDef::text("OpenCode-Go API Key"),
        FieldDef::text("NeuralWatt API Key"),
        FieldDef::text("ClinePass API Key"),
        FieldDef::text("Brave Search API Key"),
        FieldDef::text("SearXNG URL"),
        FieldDef::text("SearXNG API Key"),
        FieldDef::text("Semantic Scholar API Key"),
        FieldDef::checkbox("ArXiv Search Enabled"),
    ])) as &'static [FieldDef]
}

pub fn get_field(config: &ToolsConfig, index: usize) -> String {
    match index {
        0 => config.opencode_go_api_key.clone(),
        1 => config.neuralwatt_api_key.clone(),
        2 => config.clinepass_api_key.clone(),
        3 => config.brave_api_key.clone(),
        4 => config.searxng_url.clone(),
        5 => config.searxng_api_key.clone(),
        6 => config.semantic_scholar_api_key.clone(),
        7 => config.arxiv_enabled.to_string(),
        _ => String::new(),
    }
}

pub fn set_field(config: &mut ToolsConfig, index: usize, value: &str) {
    match index {
        0 => config.opencode_go_api_key = value.to_string(),
        1 => config.neuralwatt_api_key = value.to_string(),
        2 => config.clinepass_api_key = value.to_string(),
        3 => config.brave_api_key = value.to_string(),
        4 => config.searxng_url = value.to_string(),
        5 => config.searxng_api_key = value.to_string(),
        6 => config.semantic_scholar_api_key = value.to_string(),
        7 => config.arxiv_enabled = value == "true",
        _ => {}
    }
}

pub fn toggle_field(config: &mut ToolsConfig, index: usize) {
    if index == 7 {
        config.arxiv_enabled = !config.arxiv_enabled;
    }
}

fn is_focused(form: &FormState, index: usize) -> bool {
    form.focus == index
}

pub fn render(f: &mut ratatui::Frame, area: Rect, config: &ToolsConfig, form: &FormState, hit_registry: &mut Vec<ClickTarget>, _mouse_col: u16, _mouse_row: u16) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let grid = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(outer[0]);

    render_model_providers(f, grid[0], config, form, hit_registry);
    render_search_providers(f, grid[1], config, form, hit_registry);
    render_bottom_note(f, outer[1]);
}

fn section_block(title: &str, focused: bool, hovered: bool) -> Block<'_> {
    let border_color = if focused {
        theme::border_focus()
    } else if hovered {
        crate::presentation::theme::border_hover()
    } else {
        theme::border()
    };
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(border_color))
        .title(Span::styled(
            format!(" {} ", title),
            Style::new()
                .fg(theme::purple())
                .add_modifier(Modifier::BOLD),
        ))
}

fn provider_row<'a>(
    label: &'a str,
    masked: &'a str,
    status: &'a str,
    status_color: ratatui::style::Color,
    focused: bool,
) -> Line<'a> {
    let label_style = if focused {
        Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(theme::text_main()).add_modifier(Modifier::BOLD)
    };
    Line::from(vec![
        Span::styled(format!("{:<18}", label), label_style),
        Span::styled(masked, Style::new().fg(theme::success())),
        Span::styled(" ", Style::new().fg(theme::text_dim())),
        Span::styled("[Reveal]", Style::new().fg(theme::accent())),
        Span::styled(" ", Style::new().fg(theme::text_dim())),
        Span::styled("[Test]", Style::new().fg(theme::accent())),
        Span::styled(format!(" {}", status), Style::new().fg(status_color)),
    ])
}

fn provider_row_empty<'a>(
    label: &'a str,
    placeholder: &'a str,
    status: &'a str,
    status_color: ratatui::style::Color,
    focused: bool,
) -> Line<'a> {
    let label_style = if focused {
        Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(theme::text_main()).add_modifier(Modifier::BOLD)
    };
    Line::from(vec![
        Span::styled(format!("{:<18}", label), label_style),
        Span::styled(placeholder, Style::new().fg(theme::text_dim())),
        Span::styled(" ", Style::new().fg(theme::text_dim())),
        Span::styled("[Reveal]", Style::new().fg(theme::accent())),
        Span::styled(" ", Style::new().fg(theme::text_dim())),
        Span::styled("[Test]", Style::new().fg(theme::accent())),
        Span::styled(format!(" {}", status), Style::new().fg(status_color)),
    ])
}

fn render_model_providers(
    f: &mut ratatui::Frame,
    area: Rect,
    config: &ToolsConfig,
    form: &FormState,
    hit_registry: &mut Vec<ClickTarget>,
) {
    let block = section_block("MODEL PROVIDERS API KEYS", is_focused(form, 0) || is_focused(form, 1) || is_focused(form, 2), crate::presentation::click::is_hovering(area, form.mouse_col, form.mouse_row) && !(is_focused(form, 0) || is_focused(form, 1) || is_focused(form, 2)));
    let inner = block.inner(area);
    f.render_widget(block, area);

    hit_registry.push(ClickTarget {
        rect: area,
        action: ClickAction::FocusField(0),
    });

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(inner);

    hit_registry.push(ClickTarget {
        rect: rows[0],
        action: ClickAction::ActivateField(0),
    });
    hit_registry.push(ClickTarget {
        rect: rows[1],
        action: ClickAction::ActivateField(1),
    });
    hit_registry.push(ClickTarget {
        rect: rows[2],
        action: ClickAction::ActivateField(2),
    });

    let opencode_lines = vec![
        Line::from(vec![
            Span::styled(
                format!("{:<16}", "OpenCode-Go"),
                Style::new()
                    .fg(theme::text_main())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("$OPENCODE_API_KEY", Style::new().fg(theme::success())),
        ]),
        provider_row("", "\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}", "\u{2713}", theme::success(), is_focused(form, 0)),
        Line::from(vec![
            Span::styled("  ", Style::new().fg(theme::text_dim())),
            Span::styled(
                "Using system environment override. File secret ignored.",
                Style::new().fg(theme::success()),
            ),
        ]),
    ];
    f.render_widget(Paragraph::new(opencode_lines), rows[0]);

    let has_nw_key = !config.neuralwatt_api_key.is_empty();
    let nw_status = if has_nw_key { "\u{2713}" } else { "\u{26A0}" };
    let nw_color = if has_nw_key { theme::success() } else { theme::warning() };
    let neural_lines = vec![
        Line::from(Span::styled(
            format!("{:<16}", "NeuralWatt"),
            Style::new()
                .fg(theme::text_main())
                .add_modifier(Modifier::BOLD),
        )),
        provider_row_empty(
            "",
            "Enter NeuralWatt API Key...",
            nw_status,
            nw_color,
            is_focused(form, 1),
        ),
    ];
    f.render_widget(Paragraph::new(neural_lines), rows[1]);

    let has_cp_key = !config.clinepass_api_key.is_empty();
    let cp_status = if has_cp_key { "\u{2713}" } else { "\u{26A0}" };
    let cp_color = if has_cp_key { theme::success() } else { theme::warning() };
    let cp_mask = if has_cp_key {
        "\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}"
    } else {
        ""
    };
    let cline_lines = vec![
        Line::from(Span::styled(
            format!("{:<16}", "ClinePass"),
            Style::new()
                .fg(theme::text_main())
                .add_modifier(Modifier::BOLD),
        )),
        provider_row(cp_mask, "", cp_status, cp_color, is_focused(form, 2)),
    ];
    f.render_widget(Paragraph::new(cline_lines), rows[2]);
}

fn render_search_providers(
    f: &mut ratatui::Frame,
    area: Rect,
    config: &ToolsConfig,
    form: &FormState,
    hit_registry: &mut Vec<ClickTarget>,
) {
    let block = section_block("SEARCH PROVIDERS CONFIGURATION", is_focused(form, 3) || is_focused(form, 4) || is_focused(form, 5) || is_focused(form, 6) || is_focused(form, 7), crate::presentation::click::is_hovering(area, form.mouse_col, form.mouse_row) && !(is_focused(form, 3) || is_focused(form, 4) || is_focused(form, 5) || is_focused(form, 6) || is_focused(form, 7)));
    let inner = block.inner(area);
    f.render_widget(block, area);

    hit_registry.push(ClickTarget {
        rect: area,
        action: ClickAction::FocusField(3),
    });

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(6),
            Constraint::Length(3),
            Constraint::Length(2),
        ])
        .split(inner);

    hit_registry.push(ClickTarget {
        rect: sections[0],
        action: ClickAction::ActivateField(3),
    });
    hit_registry.push(ClickTarget {
        rect: sections[1],
        action: ClickAction::ActivateField(4),
    });
    hit_registry.push(ClickTarget {
        rect: sections[2],
        action: ClickAction::ActivateField(6),
    });
    hit_registry.push(ClickTarget {
        rect: sections[3],
        action: ClickAction::ActivateField(7),
    });

    let has_brave_key = !config.brave_api_key.is_empty();
    let brave_status = if has_brave_key { "\u{2713}" } else { "\u{26A0}" };
    let brave_color = if has_brave_key { theme::success() } else { theme::warning() };
    let brave_lines = vec![
        Line::from(vec![
            Span::styled(
                format!("{:<18}", "Brave Search"),
                Style::new()
                    .fg(theme::text_main())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("web default", Style::new().fg(theme::accent())),
        ]),
        provider_row("", "\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}", brave_status, brave_color, is_focused(form, 3)),
    ];
    f.render_widget(Paragraph::new(brave_lines), sections[0]);

    let searx_border = if is_focused(form, 4) || is_focused(form, 5) { theme::border_focus() } else { theme::cyan() };
    let searx_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(searx_border))
        .title(Span::styled(
            " SearXNG Custom Instance ",
            Style::new().fg(theme::cyan()),
        ));
    let searx_inner = searx_block.inner(sections[1]);
    f.render_widget(searx_block, sections[1]);

    let searx_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(searx_inner);

    let url_label_style = if is_focused(form, 4) {
        Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(theme::text_dim())
    };
    let url_value: String = if is_focused(form, 4) && form.is_editing() {
        if let Some(buf) = &form.edit_buffer {
            let cur = form.edit_cursor.min(buf.len());
            format!("{}{}{}", &buf[..cur], "\u{258E}", &buf[cur..])
        } else {
            String::new()
        }
    } else {
        config.searxng_url.clone()
    };
    let url_value_style = if is_focused(form, 4) {
        Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(theme::cyan())
    };
    let url_line = Line::from(vec![
        Span::styled(format!("{:<18}", "Instance URL"), url_label_style),
        Span::styled(url_value, url_value_style),
    ]);
    f.render_widget(Paragraph::new(url_line), searx_rows[0]);

    let has_sx_key = !config.searxng_api_key.is_empty();
    let sx_status = if has_sx_key { "\u{2713}" } else { "\u{26A0}" };
    let sx_color = if has_sx_key { theme::success() } else { theme::warning() };
    let sx_mask = if has_sx_key {
        "\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}\u{25CF}"
    } else {
        ""
    };
    let sx_label_style = if is_focused(form, 5) {
        Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(theme::text_dim())
    };
    let api_line = Line::from(vec![
        Span::styled(format!("{:<18}", "API Key"), sx_label_style),
        Span::styled(sx_mask, Style::new().fg(theme::success())),
        Span::styled(" ", Style::new().fg(theme::text_dim())),
        Span::styled("[Reveal]", Style::new().fg(theme::accent())),
        Span::styled(" ", Style::new().fg(theme::text_dim())),
        Span::styled("[Test]", Style::new().fg(theme::accent())),
        Span::styled(format!(" {}", sx_status), Style::new().fg(sx_color)),
    ]);
    f.render_widget(Paragraph::new(api_line), searx_rows[1]);

    let has_ss_key = !config.semantic_scholar_api_key.is_empty();
    let ss_status = if has_ss_key { "\u{2713}" } else { "\u{26A0}" };
    let ss_color = if has_ss_key { theme::success() } else { theme::warning() };
    let sem_lines = vec![
        Line::from(vec![
            Span::styled(
                format!("{:<18}", "Semantic Scholar"),
                Style::new()
                    .fg(theme::text_main())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("papers default", Style::new().fg(theme::accent())),
        ]),
        provider_row_empty(
            "",
            "Enter Semantic Scholar API Key...",
            ss_status,
            ss_color,
            is_focused(form, 6),
        ),
    ];
    f.render_widget(Paragraph::new(sem_lines), sections[2]);

    let arxiv_mark = if config.arxiv_enabled { "[\u{2713}] " } else { "[ ] " };
    let arxiv_label_style = if is_focused(form, 7) {
        Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(theme::text_main()).add_modifier(Modifier::BOLD)
    };
    let arxiv_line = Line::from(vec![
        Span::styled(format!("{:<18}", "ArXiv Search"), arxiv_label_style),
        Span::styled(arxiv_mark, Style::new().fg(theme::success())),
        Span::styled("Enabled ", Style::new().fg(theme::text_main())),
        Span::styled("(No Key Required)", Style::new().fg(theme::success())),
    ]);
    f.render_widget(Paragraph::new(arxiv_line), sections[3]);
}

fn render_bottom_note(f: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme::text_dim()));

    let note = Line::from(vec![
        Span::styled("[i] ", Style::new().fg(theme::warning())),
        Span::styled(
            "Keys are stored locally in ",
            Style::new().fg(theme::text_dim()),
        ),
        Span::styled(
            "~/.config/muon/secrets.toml",
            Style::new().fg(theme::purple()),
        ),
        Span::styled(
            ". Environment variables override file values.",
            Style::new().fg(theme::text_dim()),
        ),
    ]);
    f.render_widget(Paragraph::new(note).block(block), area);
}
