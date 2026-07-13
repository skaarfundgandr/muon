use crate::application::config::{AgentsConfig, MuonConfig};
use crate::presentation::click::{ClickAction, ClickTarget};
use crate::presentation::components::inputs::settings::dropdown_overlay::PendingDropdown;
use crate::presentation::form::{FieldDef, FormState};
use crate::presentation::theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

const FIELDS: &[FieldDef] = &[
    FieldDef::number("Cl Max turns"),        // 0
    FieldDef::checkbox("Cl Plan approval"),   // 1
    FieldDef::number("Cl Max iterations"),    // 2
    FieldDef::number("Sh Max LLM turns"),     // 3
    FieldDef::number("Sh Max tool iters"),    // 4
    FieldDef::number("Deep Orch ReAct Cycles"),   // 5
    FieldDef::number("Deep Max Retries"),         // 6
    FieldDef::number("Deep Planner Cycles"),      // 7
    FieldDef::number("Deep Orch Tool Calls"),     // 8
    FieldDef::number("Deep Planner Tool Calls"),  // 9
    FieldDef::number("Deep Researcher Tool Calls"), // 10
    FieldDef::checkbox("Deep Citation Verify"),   // 11
];

pub fn fields() -> &'static [FieldDef] {
    FIELDS
}

pub fn get_field(config: &AgentsConfig, index: usize) -> String {
    match index {
        0 => config.clarifier.max_turns.to_string(),
        1 => config.clarifier.plan_approval.to_string(),
        2 => config.clarifier.max_iterations.to_string(),
        3 => config.shallow_researcher.max_llm_turns.to_string(),
        4 => config.shallow_researcher.max_tool_iters.to_string(),
        5 => config.deep_researcher.iterations.to_string(),
        6 => config.deep_researcher.max_retries.to_string(),
        7 => config.deep_researcher.planner_max_cycles.to_string(),
        8 => config
            .deep_researcher
            .orchestrator_max_tool_calls
            .to_string(),
        9 => config.deep_researcher.planner_max_tool_calls.to_string(),
        10 => config.deep_researcher.researcher_max_tool_calls.to_string(),
        11 => config.deep_researcher.citation_verify.to_string(),
        _ => String::new(),
    }
}

pub fn set_field(config: &mut AgentsConfig, index: usize, value: &str) {
    match index {
        0 => config.clarifier.max_turns = value.parse().unwrap_or(3),
        1 => config.clarifier.plan_approval = value == "true",
        2 => config.clarifier.max_iterations = value.parse().unwrap_or(10),
        3 => config.shallow_researcher.max_llm_turns = value.parse().unwrap_or(10),
        4 => config.shallow_researcher.max_tool_iters = value.parse().unwrap_or(5),
        5 => {
            config.deep_researcher.iterations = value.parse().unwrap_or(8).max(1);
        }
        6 => {
            config.deep_researcher.max_retries = value.parse().unwrap_or(3).max(1);
        }
        7 => {
            config.deep_researcher.planner_max_cycles = value.parse().unwrap_or(3).max(1);
        }
        8 => {
            config.deep_researcher.orchestrator_max_tool_calls =
                value.parse().unwrap_or(2).max(1);
        }
        9 => {
            config.deep_researcher.planner_max_tool_calls = value.parse().unwrap_or(4).max(1);
        }
        10 => {
            config.deep_researcher.researcher_max_tool_calls =
                value.parse().unwrap_or(4).max(1);
        }
        11 => config.deep_researcher.citation_verify = value == "true",
        _ => {}
    }
}

pub fn toggle_field(config: &mut AgentsConfig, index: usize) {
    match index {
        1 => config.clarifier.plan_approval = !config.clarifier.plan_approval,
        11 => config.deep_researcher.citation_verify = !config.deep_researcher.citation_verify,
        _ => {}
    }
}

/// Returns the effective options for a field index (empty — no dropdown fields).
pub fn options_for(_field_index: usize, _config: &MuonConfig) -> Vec<String> {
    Vec::new()
}

fn is_focused(form: &FormState, index: usize) -> bool {
    form.focus == index
}

fn section_has_focus(form: &FormState, start: usize, end: usize) -> bool {
    (start..=end).any(|i| is_focused(form, i))
}

#[allow(clippy::vec_init_then_push, clippy::too_many_arguments)]
pub fn render(
    f: &mut ratatui::Frame,
    area: Rect,
    config: &MuonConfig,
    form: &FormState,
    hit_registry: &mut Vec<ClickTarget>,
    _mouse_col: u16,
    _mouse_row: u16,
    _pending_dropdown: &mut Option<PendingDropdown>,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .split(area);

    let grid = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(grid[0]);

    hit_registry.push(ClickTarget {
        rect: left_chunks[0],
        action: ClickAction::FocusField(0),
    });
    hit_registry.push(ClickTarget {
        rect: left_chunks[1],
        action: ClickAction::FocusField(3),
    });
    hit_registry.push(ClickTarget {
        rect: grid[1],
        action: ClickAction::FocusField(5),
    });

    render_clarifier(
        f,
        left_chunks[0],
        &config.agents.clarifier,
        form,
        hit_registry,
    );
    render_shallow_researcher(
        f,
        left_chunks[1],
        &config.agents.shallow_researcher,
        form,
        hit_registry,
    );
    render_deep_researcher(
        f,
        grid[1],
        &config.agents.deep_researcher,
        form,
        hit_registry,
    );
}

fn agent_block<'a>(title: &'a str, focused: bool, hovered: bool) -> Block<'a> {
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

fn input_line<'a>(
    label: &'a str,
    value: &'a str,
    focused: bool,
    editing: bool,
    cursor: usize,
    buffer: Option<&'a str>,
    hovered: bool,
) -> Line<'a> {
    if editing {
        let buf = buffer.unwrap_or("");
        let cur = cursor.min(buf.len());
        let pre = &buf[..cur];
        let post = &buf[cur..];
        Line::from(vec![
            Span::styled(
                "> ",
                Style::new()
                    .fg(theme::border_focus())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{:<14}", label),
                Style::new()
                    .fg(theme::border_focus())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("[", Style::new().fg(theme::border_focus())),
            Span::styled(
                pre.to_string(),
                Style::new()
                    .fg(theme::accent())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("\u{2588}", Style::new().fg(theme::border_focus())),
            Span::styled(
                post.to_string(),
                Style::new()
                    .fg(theme::accent())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("]", Style::new().fg(theme::border_focus())),
        ])
    } else if focused {
        Line::from(vec![
            Span::styled(
                "> ",
                Style::new()
                    .fg(theme::border_focus())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{:<14}", label),
                Style::new()
                    .fg(theme::border_focus())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("[", Style::new().fg(theme::border_focus())),
            Span::styled(
                value,
                Style::new()
                    .fg(theme::border_focus())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("]", Style::new().fg(theme::border_focus())),
        ])
    } else if hovered {
        Line::from(vec![
            Span::styled(
                format!("{:<14}", label),
                Style::new().fg(crate::presentation::theme::border_hover()),
            ),
            Span::styled(
                "[",
                Style::new().fg(crate::presentation::theme::border_hover()),
            ),
            Span::styled(value, Style::new().fg(theme::success())),
            Span::styled(
                "]",
                Style::new().fg(crate::presentation::theme::border_hover()),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled(format!("{:<14}", label), Style::new().fg(theme::text_dim())),
            Span::styled("[", Style::new().fg(theme::text_dim())),
            Span::styled(value, Style::new().fg(theme::success())),
            Span::styled("]", Style::new().fg(theme::text_dim())),
        ])
    }
}

fn checkbox_line(label: &str, checked: bool, focused: bool, hovered: bool) -> Line<'_> {
    let mark = if checked { "[\u{2713}]" } else { "[ ]" };
    let mark_color = if checked {
        theme::success()
    } else {
        theme::text_dim()
    };
    if focused {
        Line::from(vec![
            Span::styled(
                "> ",
                Style::new()
                    .fg(theme::border_focus())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{} ", label),
                Style::new()
                    .fg(theme::border_focus())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(mark, Style::new().fg(theme::border_focus())),
        ])
    } else if hovered {
        Line::from(vec![
            Span::styled(
                format!("{} ", label),
                Style::new().fg(crate::presentation::theme::border_hover()),
            ),
            Span::styled(mark, Style::new().fg(mark_color)),
        ])
    } else {
        Line::from(vec![
            Span::styled(format!("{} ", label), Style::new().fg(theme::text_dim())),
            Span::styled(mark, Style::new().fg(mark_color)),
        ])
    }
}

fn render_clarifier(
    f: &mut ratatui::Frame,
    area: Rect,
    cfg: &crate::application::config::ClarifierConfig,
    form: &FormState,
    hit_registry: &mut Vec<ClickTarget>,
) {
    let focused = section_has_focus(form, 0, 2);
    let max_turns_str = cfg.max_turns.to_string();
    let max_iters_str = cfg.max_iterations.to_string();
    let hovered = crate::presentation::click::is_hovering(area, form.mouse_col, form.mouse_row);
    let inner = agent_block("CLARIFIER (HITL)", focused, hovered && !focused).inner(area);
    f.render_widget(
        agent_block("CLARIFIER (HITL)", focused, hovered && !focused),
        area,
    );

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    for (i, row_rect) in rows.iter().enumerate() {
        hit_registry.push(ClickTarget {
            rect: *row_rect,
            action: ClickAction::ActivateField(i),
        });
    }

    let lines: Vec<Line> = vec![
        input_line(
            "Max turns",
            &max_turns_str,
            is_focused(form, 0),
            is_focused(form, 0) && form.is_editing(),
            form.edit_cursor,
            form.edit_buffer.as_deref(),
            crate::presentation::click::is_hovering(rows[0], form.mouse_col, form.mouse_row)
                && !is_focused(form, 0),
        ),
        checkbox_line(
            "Plan approval  ",
            cfg.plan_approval,
            is_focused(form, 1),
            crate::presentation::click::is_hovering(rows[1], form.mouse_col, form.mouse_row)
                && !is_focused(form, 1),
        ),
        input_line(
            "Max iterations",
            &max_iters_str,
            is_focused(form, 2),
            is_focused(form, 2) && form.is_editing(),
            form.edit_cursor,
            form.edit_buffer.as_deref(),
            crate::presentation::click::is_hovering(rows[2], form.mouse_col, form.mouse_row)
                && !is_focused(form, 2),
        ),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

fn render_shallow_researcher(
    f: &mut ratatui::Frame,
    area: Rect,
    cfg: &crate::application::config::ShallowResearcherConfig,
    form: &FormState,
    hit_registry: &mut Vec<ClickTarget>,
) {
    let focused = section_has_focus(form, 3, 4);
    let llm_turns_str = cfg.max_llm_turns.to_string();
    let tool_iters_str = cfg.max_tool_iters.to_string();
    let hovered = crate::presentation::click::is_hovering(area, form.mouse_col, form.mouse_row);
    let inner = agent_block("SHALLOW RESEARCHER", focused, hovered && !focused).inner(area);
    f.render_widget(
        agent_block("SHALLOW RESEARCHER", focused, hovered && !focused),
        area,
    );

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    for (i, row_rect) in rows.iter().enumerate() {
        hit_registry.push(ClickTarget {
            rect: *row_rect,
            action: ClickAction::ActivateField(3 + i),
        });
    }

    let lines: Vec<Line> = vec![
        input_line(
            "Max LLM turns",
            &llm_turns_str,
            is_focused(form, 3),
            is_focused(form, 3) && form.is_editing(),
            form.edit_cursor,
            form.edit_buffer.as_deref(),
            crate::presentation::click::is_hovering(rows[0], form.mouse_col, form.mouse_row)
                && !is_focused(form, 3),
        ),
        input_line(
            "Max tool iters",
            &tool_iters_str,
            is_focused(form, 4),
            is_focused(form, 4) && form.is_editing(),
            form.edit_cursor,
            form.edit_buffer.as_deref(),
            crate::presentation::click::is_hovering(rows[1], form.mouse_col, form.mouse_row)
                && !is_focused(form, 4),
        ),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

fn deep_num_cell(
    label: &str,
    value: &str,
    field: usize,
    form: &FormState,
    area: Rect,
) -> Line<'static> {
    let focused = is_focused(form, field);
    let editing = focused && form.is_editing();
    let hovered =
        crate::presentation::click::is_hovering(area, form.mouse_col, form.mouse_row) && !focused;
    let display = if editing {
        form.edit_buffer.as_deref().unwrap_or(value)
    } else {
        value
    };
    let label_style = if focused {
        Style::new()
            .fg(theme::border_focus())
            .add_modifier(Modifier::BOLD)
    } else if hovered {
        Style::new().fg(theme::border_hover())
    } else {
        Style::new().fg(theme::text_dim())
    };
    let val_style = if focused || editing {
        Style::new()
            .fg(theme::border_focus())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(theme::success())
    };
    let mut spans = Vec::new();
    if focused {
        spans.push(Span::styled(
            "> ",
            Style::new()
                .fg(theme::border_focus())
                .add_modifier(Modifier::BOLD),
        ));
    }
    spans.push(Span::styled(format!("{label} "), label_style));
    spans.push(Span::styled("[", label_style));
    if editing {
        let buf = form.edit_buffer.as_deref().unwrap_or("");
        let cur = form.edit_cursor.min(buf.len());
        spans.push(Span::styled(buf[..cur].to_string(), val_style));
        spans.push(Span::styled(
            "\u{2588}",
            Style::new().fg(theme::border_focus()),
        ));
        spans.push(Span::styled(buf[cur..].to_string(), val_style));
    } else {
        spans.push(Span::styled(display.to_string(), val_style));
    }
    spans.push(Span::styled("]", label_style));
    Line::from(spans)
}

fn render_deep_researcher(
    f: &mut ratatui::Frame,
    area: Rect,
    cfg: &crate::application::config::DeepResearcherConfig,
    form: &FormState,
    hit_registry: &mut Vec<ClickTarget>,
) {
    let focused = section_has_focus(form, 5, 11);
    let hovered = crate::presentation::click::is_hovering(area, form.mouse_col, form.mouse_row);
    let block = agent_block("DEEP RESEARCHER", focused, hovered && !focused);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // row 0: orch cycles + orch tool calls
            Constraint::Length(1), // row 1: planner cycles + planner tool calls
            Constraint::Length(1), // row 2: researcher tool calls (full width)
            Constraint::Length(1), // row 3: retries + citation verify
            Constraint::Min(0),   // spacer
        ])
        .split(inner);

    let pair = |row: Rect| {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(row)
    };

    // Row 0: orch cycles + orch tool calls
    let cols0 = pair(rows[0]);
    hit_registry.push(ClickTarget {
        rect: cols0[0],
        action: ClickAction::ActivateField(5),
    });
    hit_registry.push(ClickTarget {
        rect: cols0[1],
        action: ClickAction::ActivateField(8),
    });
    f.render_widget(
        Paragraph::new(deep_num_cell(
            "Orch ReAct cycles",
            &cfg.iterations.to_string(),
            5,
            form,
            cols0[0],
        )),
        cols0[0],
    );
    f.render_widget(
        Paragraph::new(deep_num_cell(
            "Orch tool calls",
            &cfg.orchestrator_max_tool_calls.to_string(),
            8,
            form,
            cols0[1],
        )),
        cols0[1],
    );

    // Row 1: planner cycles + planner tool calls
    let cols1 = pair(rows[1]);
    hit_registry.push(ClickTarget {
        rect: cols1[0],
        action: ClickAction::ActivateField(7),
    });
    hit_registry.push(ClickTarget {
        rect: cols1[1],
        action: ClickAction::ActivateField(9),
    });
    f.render_widget(
        Paragraph::new(deep_num_cell(
            "Planner cycles",
            &cfg.planner_max_cycles.to_string(),
            7,
            form,
            cols1[0],
        )),
        cols1[0],
    );
    f.render_widget(
        Paragraph::new(deep_num_cell(
            "Planner tool calls",
            &cfg.planner_max_tool_calls.to_string(),
            9,
            form,
            cols1[1],
        )),
        cols1[1],
    );

    // Row 2: researcher tool calls (full width)
    hit_registry.push(ClickTarget {
        rect: rows[2],
        action: ClickAction::ActivateField(10),
    });
    f.render_widget(
        Paragraph::new(deep_num_cell(
            "Researcher tool calls",
            &cfg.researcher_max_tool_calls.to_string(),
            10,
            form,
            rows[2],
        )),
        rows[2],
    );

    // Row 3: retries + citation verify
    let footer = pair(rows[3]);
    hit_registry.push(ClickTarget {
        rect: footer[0],
        action: ClickAction::ActivateField(6),
    });
    hit_registry.push(ClickTarget {
        rect: footer[1],
        action: ClickAction::ActivateField(11),
    });
    f.render_widget(
        Paragraph::new(deep_num_cell(
            "Max retries",
            &cfg.max_retries.to_string(),
            6,
            form,
            footer[0],
        )),
        footer[0],
    );
    f.render_widget(
        Paragraph::new(checkbox_line(
            "Citation Verify",
            cfg.citation_verify,
            is_focused(form, 11),
            crate::presentation::click::is_hovering(footer[1], form.mouse_col, form.mouse_row)
                && !is_focused(form, 11),
        )),
        footer[1],
    );
}
