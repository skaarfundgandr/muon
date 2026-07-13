use crate::application::config::{DataSourcesConfig, MuonConfig};
use crate::presentation::click::{ClickAction, ClickTarget, is_hovering};
use crate::presentation::components::inputs::settings::dropdown_overlay::PendingDropdown;
use crate::presentation::form::{FieldDef, FormState};
use crate::presentation::theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn fields(config: &MuonConfig) -> Vec<FieldDef> {
    let _ = config;
    vec![
        FieldDef::checkbox("Web Search"),
        FieldDef::checkbox("Paper Search"),
        FieldDef::checkbox("Enterprise Systems (reserved)"),
        FieldDef::checkbox("Knowledge Layer (RAG)"),
        FieldDef::text("Source Path"),
        FieldDef::dropdown("Source Type", &["Directory", "File", "Glob"]),
        FieldDef::button("[ + Add ]"),
    ]
}

pub fn get_field(config: &MuonConfig, index: usize) -> String {
    match index {
        0 => config.data_sources.web_search.to_string(),
        1 => config.data_sources.paper_search.to_string(),
        2 => config.data_sources.enterprise_systems.to_string(),
        3 => config.data_sources.knowledge_layer_rag.to_string(),
        4 => config.data_sources.source_path.clone(),
        5 => config.data_sources.source_type.clone(),
        _ => String::new(),
    }
}

pub fn set_field(config: &mut MuonConfig, index: usize, value: &str) {
    match index {
        0 => config.data_sources.web_search = value == "true",
        1 => config.data_sources.paper_search = value == "true",
        2 => config.data_sources.enterprise_systems = value == "true",
        3 => config.data_sources.knowledge_layer_rag = value == "true",
        4 => config.data_sources.source_path = value.to_string(),
        5 => config.data_sources.source_type = value.to_string(),
        _ => {}
    }
}

pub fn toggle_field(config: &mut MuonConfig, index: usize) {
    match index {
        0 => config.data_sources.web_search = !config.data_sources.web_search,
        1 => config.data_sources.paper_search = !config.data_sources.paper_search,
        2 => config.data_sources.enterprise_systems = !config.data_sources.enterprise_systems,
        3 => config.data_sources.knowledge_layer_rag = !config.data_sources.knowledge_layer_rag,
        _ => {}
    }
}

fn is_focused(form: &FormState, index: usize) -> bool {
    form.focus == index
}

#[allow(clippy::too_many_arguments)]
pub fn render(
    f: &mut ratatui::Frame,
    area: Rect,
    config: &MuonConfig,
    form: &FormState,
    hit_registry: &mut Vec<ClickTarget>,
    mouse_col: u16,
    mouse_row: u16,
    pending_dropdown: &mut Option<PendingDropdown>,
) {
    let grid = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_source_providers(
        f,
        grid[0],
        &config.data_sources,
        form,
        hit_registry,
        mouse_col,
        mouse_row,
    );
    render_rag_indexes(
        f,
        grid[1],
        config,
        form,
        hit_registry,
        mouse_col,
        mouse_row,
        pending_dropdown,
    );
}

fn section_block<'a>(title: &'a str, focused: bool, hovered: bool) -> Block<'a> {
    let border_color = if focused {
        theme::border_focus()
    } else if hovered {
        theme::border_hover()
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

fn render_source_providers(
    f: &mut ratatui::Frame,
    area: Rect,
    config: &DataSourcesConfig,
    form: &FormState,
    hit_registry: &mut Vec<ClickTarget>,
    mouse_col: u16,
    mouse_row: u16,
) {
    let any_focused =
        is_focused(form, 0) || is_focused(form, 1) || is_focused(form, 2) || is_focused(form, 3);
    let block = section_block(
        "SOURCE PROVIDERS",
        any_focused,
        is_hovering(area, mouse_col, mouse_row) && !any_focused,
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
            "Search arXiv paper archives",
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
        let hovered = is_hovering(rows[i], mouse_col, mouse_row);
        let status_text = if *enabled {
            "\u{2713} ENABLED"
        } else {
            "\u{25CB} DISABLED"
        };
        let status_color = if *enabled {
            theme::success()
        } else {
            theme::text_dim()
        };
        let checkbox_str = if *enabled { "[\u{2713}]" } else { "[ ]" };
        let checkbox_color = if *enabled {
            theme::success()
        } else {
            theme::text_dim()
        };

        let prefix = if focused { "> " } else { "  " };
        let title_style = if focused {
            Style::new()
                .fg(theme::border_focus())
                .add_modifier(Modifier::BOLD)
        } else if hovered {
            Style::new()
                .fg(theme::border_hover())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::new()
                .fg(theme::text_main())
                .add_modifier(Modifier::BOLD)
        };

        let card_lines = vec![
            Line::from(vec![
                Span::styled(
                    prefix,
                    Style::new()
                        .fg(theme::border_focus())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!("[{}] ", i + 1), Style::new().fg(theme::text_dim())),
                Span::styled(*title, title_style),
                Span::raw("  "),
                Span::styled(status_text, Style::new().fg(status_color)),
                Span::raw("  "),
                Span::styled(checkbox_str, Style::new().fg(checkbox_color)),
            ]),
            Line::from(vec![Span::styled(
                format!("      {}", desc),
                Style::new().fg(theme::text_dim()),
            )]),
        ];

        f.render_widget(Paragraph::new(card_lines), rows[i]);
    }
}

#[allow(clippy::too_many_arguments)]
fn render_rag_indexes(
    f: &mut ratatui::Frame,
    area: Rect,
    config: &MuonConfig,
    form: &FormState,
    hit_registry: &mut Vec<ClickTarget>,
    mouse_col: u16,
    mouse_row: u16,
    pending_dropdown: &mut Option<PendingDropdown>,
) {
    let any_focused = is_focused(form, 4) || is_focused(form, 5) || is_focused(form, 6);
    let block = section_block(
        "KNOWLEDGE LAYER / RAG INDEXES",
        any_focused,
        is_hovering(area, mouse_col, mouse_row) && !any_focused,
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

    render_add_source_form(
        f,
        sections[0],
        config,
        form,
        hit_registry,
        mouse_col,
        mouse_row,
    );
    render_source_table(f, sections[1], config, hit_registry, mouse_col, mouse_row);

    if form.dropdown_open && form.focus == 5 {
        let form_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(55),
                Constraint::Percentage(30),
                Constraint::Percentage(15),
            ])
            .split(sections[0]);
        let field_label = "Source Type";
        let options: Vec<String> = vec![
            "Directory".to_string(),
            "File".to_string(),
            "Glob".to_string(),
        ];
        *pending_dropdown = Some(PendingDropdown {
            below: form_cols[1],
            field_label: field_label.to_string(),
            options,
        });
    }
}

fn render_add_source_form(
    f: &mut ratatui::Frame,
    area: Rect,
    config: &MuonConfig,
    form: &FormState,
    hit_registry: &mut Vec<ClickTarget>,
    mouse_col: u16,
    mouse_row: u16,
) {
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
            format!("{}{}{}", &buf[..cur], "\u{2588}", &buf[cur..])
        } else {
            String::new()
        }
    } else {
        config.data_sources.source_path.clone()
    };
    let path_value_style = if is_focused(form, 4) {
        Style::new()
            .fg(theme::border_focus())
            .add_modifier(Modifier::BOLD)
    } else if is_hovering(form_cols[0], mouse_col, mouse_row) {
        Style::new().fg(theme::border_hover())
    } else {
        Style::new().fg(theme::text_main())
    };
    let path_line = Line::from(vec![
        Span::styled(
            path_prefix,
            Style::new()
                .fg(theme::border_focus())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(path_value, path_value_style),
    ]);
    f.render_widget(Paragraph::new(path_line), form_cols[0]);

    let type_prefix = if is_focused(form, 5) { "> " } else { "  " };
    let type_label_style = if is_focused(form, 5) {
        Style::new()
            .fg(theme::border_focus())
            .add_modifier(Modifier::BOLD)
    } else if is_hovering(form_cols[1], mouse_col, mouse_row) {
        Style::new().fg(theme::border_hover())
    } else {
        Style::new().fg(theme::text_dim())
    };
    let type_val_style = if is_focused(form, 5) {
        Style::new()
            .fg(theme::border_focus())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(theme::accent())
    };
    let type_line = Line::from(vec![
        Span::styled(
            type_prefix,
            Style::new()
                .fg(theme::border_focus())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("[", type_label_style),
        Span::styled(
            format!("{}\u{25BC}", config.data_sources.source_type),
            type_val_style,
        ),
        Span::styled("]", type_label_style),
    ]);
    f.render_widget(Paragraph::new(type_line), form_cols[1]);

    let btn_prefix = if is_focused(form, 6) { "> " } else { "  " };
    let btn_style = if is_focused(form, 6) {
        Style::new()
            .fg(theme::border_focus())
            .add_modifier(Modifier::BOLD)
    } else if is_hovering(form_cols[2], mouse_col, mouse_row) {
        Style::new()
            .fg(theme::accent())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(theme::accent())
    };
    let btn_line = Line::from(vec![
        Span::styled(
            btn_prefix,
            Style::new()
                .fg(theme::border_focus())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("[ + Add ]", btn_style),
    ]);
    f.render_widget(Paragraph::new(btn_line), form_cols[2]);
}

fn render_source_table(
    f: &mut ratatui::Frame,
    area: Rect,
    config: &MuonConfig,
    hit_registry: &mut Vec<ClickTarget>,
    mouse_col: u16,
    mouse_row: u16,
) {
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
                .fg(theme::purple())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:<width$}", "TYPE", width = col_type),
            Style::new()
                .fg(theme::purple())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:<width$}", "STATUS", width = col_status),
            Style::new()
                .fg(theme::purple())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:<width$}", "CHUNKS", width = col_chunks),
            Style::new()
                .fg(theme::purple())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:<width$}", "ACTIONS", width = col_actions),
            Style::new()
                .fg(theme::purple())
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let separator = Line::from(Span::styled(
        "\u{2500}".repeat(usable_w),
        Style::new().fg(theme::border()),
    ));

    let mut lines: Vec<Line> = vec![header, separator];

    let row_count = config.data_sources.rag_indexes.len();
    for i in 0..row_count {
        let index_item = &config.data_sources.rag_indexes[i];
        let status_color = if index_item.status.contains("indexed") {
            theme::success()
        } else if index_item.status.contains("indexing") {
            theme::accent()
        } else {
            theme::warning()
        };

        // Determine refresh & remove hit test areas
        let row_y = area.y + 2 + i as u16;
        if row_y < area.y + area.height {
            let refresh_x = area.x + (col_path + col_type + col_status + col_chunks) as u16 + 1;
            let refresh_rect = Rect::new(refresh_x, row_y, 1, 1);
            hit_registry.push(ClickTarget {
                rect: refresh_rect,
                action: ClickAction::ReindexRagIndex(i),
            });

            let remove_x = refresh_x + 4;
            let remove_rect = Rect::new(remove_x, row_y, 1, 1);
            hit_registry.push(ClickTarget {
                rect: remove_rect,
                action: ClickAction::RemoveRagIndex(i),
            });
        }

        let ref_style = if row_y < area.y + area.height
            && is_hovering(
                Rect::new(
                    area.x + (col_path + col_type + col_status + col_chunks) as u16 + 1,
                    row_y,
                    1,
                    1,
                ),
                mouse_col,
                mouse_row,
            ) {
            Style::new()
                .fg(theme::accent())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(theme::accent())
        };

        let rem_style = if row_y < area.y + area.height
            && is_hovering(
                Rect::new(
                    area.x + (col_path + col_type + col_status + col_chunks) as u16 + 5,
                    row_y,
                    1,
                    1,
                ),
                mouse_col,
                mouse_row,
            ) {
            Style::new().fg(theme::error()).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(theme::error())
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!("{:<width$}", index_item.path, width = col_path),
                Style::new().fg(theme::text_main()),
            ),
            Span::styled(
                format!("{:<width$}", index_item.kind, width = col_type),
                Style::new().fg(theme::cyan()),
            ),
            Span::styled(
                format!("{:<width$}", index_item.status, width = col_status),
                Style::new().fg(status_color),
            ),
            Span::styled(
                format!("{:<width$}", index_item.chunks, width = col_chunks),
                Style::new().fg(theme::text_main()),
            ),
            Span::styled("[", Style::new().fg(theme::text_dim())),
            Span::styled("\u{21BB}", ref_style),
            Span::styled("] [", Style::new().fg(theme::text_dim())),
            Span::styled("\u{00D7}", rem_style),
            Span::styled("]", Style::new().fg(theme::text_dim())),
        ]));
    }

    f.render_widget(Paragraph::new(lines), area);
}
