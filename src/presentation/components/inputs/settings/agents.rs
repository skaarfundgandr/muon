use crate::application::config::{
    AgentDef, AgentSettings, AgentsConfig, ClarifierConfig, DeepResearcherConfig, MuonConfig,
    ShallowResearcherConfig,
};
use crate::presentation::click::{ClickAction, ClickTarget};
use crate::presentation::components::inputs::settings::dropdown_overlay::PendingDropdown;
use crate::presentation::form::{FieldDef, FormState};
use crate::presentation::theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

const FIELDS: &[FieldDef] = &[
    FieldDef::dropdown("IC Model", &[]),          // 0
    FieldDef::dropdown("IC Provider", &[]),       // 1
    FieldDef::number("IC Timeout"),               // 2
    FieldDef::dropdown("Cl Model", &[]),          // 3
    FieldDef::dropdown("Cl Provider", &[]),       // 4
    FieldDef::number("Cl Max turns"),             // 5
    FieldDef::checkbox("Cl Plan approval"),       // 6
    FieldDef::number("Cl Max iterations"),        // 7
    FieldDef::dropdown("Sh Model", &[]),          // 8
    FieldDef::dropdown("Sh Provider", &[]),       // 9
    FieldDef::number("Sh Max LLM turns"),         // 10
    FieldDef::number("Sh Max tool iters"),        // 11
    FieldDef::dropdown("Deep Orchestrator Model", &[]),    // 12
    FieldDef::dropdown("Deep Orchestrator Provider", &[]), // 13
    FieldDef::dropdown("Deep Planner Model", &[]),         // 14
    FieldDef::dropdown("Deep Planner Provider", &[]),      // 15
    FieldDef::dropdown("Deep Researcher Model", &[]),      // 16
    FieldDef::dropdown("Deep Researcher Provider", &[]),   // 17
    FieldDef::number("Deep Orch ReAct Cycles"),   // 18
    FieldDef::number("Deep Max Retries"),         // 19
    FieldDef::number("Deep Planner Cycles"),      // 20
    FieldDef::number("Deep Orch Tool Calls"),     // 21
    FieldDef::number("Deep Planner Tool Calls"),  // 22
    FieldDef::number("Deep Researcher Tool Calls"), // 23
    FieldDef::checkbox("Deep Citation Verify"),   // 24
];

pub fn fields() -> &'static [FieldDef] {
    FIELDS
}

pub fn get_field(config: &AgentsConfig, agents: &AgentSettings, index: usize) -> String {
    match index {
        0 => agents.intent_classifier.model.clone(),
        1 => agents.intent_classifier.provider.clone(),
        2 => agents.intent_classifier.timeout_secs.to_string(),
        3 => agents.clarifier.model.clone(),
        4 => agents.clarifier.provider.clone(),
        5 => config.clarifier.max_turns.to_string(),
        6 => config.clarifier.plan_approval.to_string(),
        7 => config.clarifier.max_iterations.to_string(),
        8 => agents.shallow_researcher.model.clone(),
        9 => agents.shallow_researcher.provider.clone(),
        10 => config.shallow_researcher.max_llm_turns.to_string(),
        11 => config.shallow_researcher.max_tool_iters.to_string(),
        12 => agents.deep_orchestrator.model.clone(),
        13 => agents.deep_orchestrator.provider.clone(),
        14 => agents.planner.model.clone(),
        15 => agents.planner.provider.clone(),
        16 => agents.researcher.model.clone(),
        17 => agents.researcher.provider.clone(),
        18 => config.deep_researcher.iterations.to_string(),
        19 => config.deep_researcher.max_retries.to_string(),
        20 => config.deep_researcher.planner_max_cycles.to_string(),
        21 => config
            .deep_researcher
            .orchestrator_max_tool_calls
            .to_string(),
        22 => config.deep_researcher.planner_max_tool_calls.to_string(),
        23 => config.deep_researcher.researcher_max_tool_calls.to_string(),
        24 => config.deep_researcher.citation_verify.to_string(),
        _ => String::new(),
    }
}

pub fn set_field(
    config: &mut AgentsConfig,
    agents: &mut AgentSettings,
    index: usize,
    value: &str,
) {
    match index {
        0 => agents.intent_classifier.model = value.to_string(),
        1 => agents.intent_classifier.provider = value.to_string(),
        2 => agents.intent_classifier.timeout_secs = value.parse().unwrap_or(90),
        3 => agents.clarifier.model = value.to_string(),
        4 => agents.clarifier.provider = value.to_string(),
        5 => config.clarifier.max_turns = value.parse().unwrap_or(3),
        6 => config.clarifier.plan_approval = value == "true",
        7 => config.clarifier.max_iterations = value.parse().unwrap_or(10),
        8 => agents.shallow_researcher.model = value.to_string(),
        9 => agents.shallow_researcher.provider = value.to_string(),
        10 => config.shallow_researcher.max_llm_turns = value.parse().unwrap_or(10),
        11 => config.shallow_researcher.max_tool_iters = value.parse().unwrap_or(5),
        12 => agents.deep_orchestrator.model = value.to_string(),
        13 => agents.deep_orchestrator.provider = value.to_string(),
        14 => agents.planner.model = value.to_string(),
        15 => agents.planner.provider = value.to_string(),
        16 => agents.researcher.model = value.to_string(),
        17 => agents.researcher.provider = value.to_string(),
        18 => {
            config.deep_researcher.iterations = value.parse().unwrap_or(8).max(1);
        }
        19 => {
            config.deep_researcher.max_retries = value.parse().unwrap_or(3).max(1);
        }
        20 => {
            config.deep_researcher.planner_max_cycles = value.parse().unwrap_or(3).max(1);
        }
        21 => {
            config.deep_researcher.orchestrator_max_tool_calls = value.parse().unwrap_or(2).max(1);
        }
        22 => {
            config.deep_researcher.planner_max_tool_calls = value.parse().unwrap_or(4).max(1);
        }
        23 => {
            config.deep_researcher.researcher_max_tool_calls = value.parse().unwrap_or(4).max(1);
        }
        24 => config.deep_researcher.citation_verify = value == "true",
        _ => {}
    }
}

pub fn toggle_field(config: &mut AgentsConfig, _agents: &mut AgentSettings, index: usize) {
    match index {
        6 => config.clarifier.plan_approval = !config.clarifier.plan_approval,
        24 => config.deep_researcher.citation_verify = !config.deep_researcher.citation_verify,
        _ => {}
    }
}

fn provider_options(config: &MuonConfig) -> Vec<String> {
    if config.providers.is_empty() {
        vec!["<no providers>".to_string()]
    } else {
        config.providers.iter().map(|p| p.name.clone()).collect()
    }
}

fn model_options(config: &MuonConfig, provider_name: &str) -> Vec<String> {
    let provider = config.providers.iter().find(|p| p.name == provider_name);
    match provider {
        Some(p) => {
            if p.models.is_empty() {
                vec!["<no models; edit/fetch in Providers>".to_string()]
            } else {
                p.models.iter().map(|m| m.name.clone()).collect()
            }
        }
        None => Vec::new(),
    }
}

/// Returns the effective options for a field index, dynamic for provider/model dropdowns.
pub fn options_for(
    field_index: usize,
    config: &MuonConfig,
    agents: &AgentSettings,
) -> Vec<String> {
    const PROVIDER_FIELDS: &[usize] = &[1, 4, 9, 13, 15, 17];
    const MODEL_FIELDS: &[usize] = &[0, 3, 8, 12, 14, 16];
    if PROVIDER_FIELDS.contains(&field_index) {
        provider_options(config)
    } else if MODEL_FIELDS.contains(&field_index) {
        let provider_name = match field_index {
            0 | 1 => agents.intent_classifier.provider.as_str(),
            3 | 4 => agents.clarifier.provider.as_str(),
            8 | 9 => agents.shallow_researcher.provider.as_str(),
            12 | 13 => agents.deep_orchestrator.provider.as_str(),
            14 | 15 => agents.planner.provider.as_str(),
            16 | 17 => agents.researcher.provider.as_str(),
            _ => return Vec::new(),
        };
        model_options(config, provider_name)
    } else {
        Vec::new()
    }
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
    agents: &AgentSettings,
    form: &FormState,
    hit_registry: &mut Vec<ClickTarget>,
    _mouse_col: u16,
    _mouse_row: u16,
    pending_dropdown: &mut Option<PendingDropdown>,
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

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(grid[1]);

    hit_registry.push(ClickTarget {
        rect: left_chunks[0],
        action: ClickAction::FocusField(0),
    });
    hit_registry.push(ClickTarget {
        rect: left_chunks[1],
        action: ClickAction::FocusField(3),
    });
    hit_registry.push(ClickTarget {
        rect: right_chunks[0],
        action: ClickAction::FocusField(8),
    });
    hit_registry.push(ClickTarget {
        rect: right_chunks[1],
        action: ClickAction::FocusField(12),
    });

    render_intent_classifier(
        f,
        left_chunks[0],
        &agents.intent_classifier,
        form,
        hit_registry,
        config,
        agents,
        pending_dropdown,
    );
    render_clarifier(
        f,
        left_chunks[1],
        &agents.clarifier,
        &config.agents.clarifier,
        form,
        hit_registry,
        config,
        agents,
        pending_dropdown,
    );
    render_shallow_researcher(
        f,
        right_chunks[0],
        &agents.shallow_researcher,
        &config.agents.shallow_researcher,
        form,
        hit_registry,
        config,
        agents,
        pending_dropdown,
    );
    render_deep_researcher(
        f,
        right_chunks[1],
        agents,
        &config.agents.deep_researcher,
        form,
        hit_registry,
        config,
        pending_dropdown,
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

fn dropdown_line<'a>(label: &'a str, value: &'a str, focused: bool, hovered: bool) -> Line<'a> {
    if focused {
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
            Span::styled("\u{25BC}", Style::new().fg(theme::border_focus())),
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
            Span::styled(value, Style::new().fg(theme::accent())),
            Span::styled("\u{25BC}", Style::new().fg(theme::accent())),
            Span::styled(
                "]",
                Style::new().fg(crate::presentation::theme::border_hover()),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled(format!("{:<14}", label), Style::new().fg(theme::text_dim())),
            Span::styled("[", Style::new().fg(theme::text_dim())),
            Span::styled(value, Style::new().fg(theme::accent())),
            Span::styled("\u{25BC}", Style::new().fg(theme::accent())),
            Span::styled("]", Style::new().fg(theme::text_dim())),
        ])
    }
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

fn render_intent_classifier(
    f: &mut ratatui::Frame,
    area: Rect,
    cfg: &AgentDef,
    form: &FormState,
    hit_registry: &mut Vec<ClickTarget>,
    config: &MuonConfig,
    agents: &AgentSettings,
    pending_dropdown: &mut Option<PendingDropdown>,
) {
    let focused = section_has_focus(form, 0, 2);
    let timeout_str = cfg.timeout_secs.to_string();
    let hovered = crate::presentation::click::is_hovering(area, form.mouse_col, form.mouse_row);
    let inner = agent_block("INTENT CLASSIFIER", focused, hovered && !focused).inner(area);
    f.render_widget(
        agent_block("INTENT CLASSIFIER", focused, hovered && !focused),
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
        dropdown_line(
            "Model",
            &cfg.model,
            is_focused(form, 0),
            crate::presentation::click::is_hovering(rows[0], form.mouse_col, form.mouse_row)
                && !is_focused(form, 0),
        ),
        dropdown_line(
            "Provider",
            &cfg.provider,
            is_focused(form, 1),
            crate::presentation::click::is_hovering(rows[1], form.mouse_col, form.mouse_row)
                && !is_focused(form, 1),
        ),
        input_line(
            "Timeout (sec)",
            &timeout_str,
            is_focused(form, 2),
            is_focused(form, 2) && form.is_editing(),
            form.edit_cursor,
            form.edit_buffer.as_deref(),
            crate::presentation::click::is_hovering(rows[2], form.mouse_col, form.mouse_row)
                && !is_focused(form, 2),
        ),
    ];

    f.render_widget(Paragraph::new(lines), inner);

    if form.dropdown_open && (form.focus == 0 || form.focus == 1) {
        let row_below = rows[form.focus];
        let field_label = fields()[form.focus].label;
        *pending_dropdown = Some(PendingDropdown {
            below: row_below,
            field_label: field_label.to_string(),
            options: options_for(form.focus, config, agents),
        });
    }
}

fn render_clarifier(
    f: &mut ratatui::Frame,
    area: Rect,
    def: &AgentDef,
    knobs: &ClarifierConfig,
    form: &FormState,
    hit_registry: &mut Vec<ClickTarget>,
    config: &MuonConfig,
    agents: &AgentSettings,
    pending_dropdown: &mut Option<PendingDropdown>,
) {
    let focused = section_has_focus(form, 3, 7);
    let max_turns_str = knobs.max_turns.to_string();
    let max_iters_str = knobs.max_iterations.to_string();
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
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    for (i, row_rect) in rows.iter().enumerate() {
        hit_registry.push(ClickTarget {
            rect: *row_rect,
            action: ClickAction::ActivateField(3 + i),
        });
    }

    let lines: Vec<Line> = vec![
        dropdown_line(
            "Model",
            &def.model,
            is_focused(form, 3),
            crate::presentation::click::is_hovering(rows[0], form.mouse_col, form.mouse_row)
                && !is_focused(form, 3),
        ),
        dropdown_line(
            "Provider",
            &def.provider,
            is_focused(form, 4),
            crate::presentation::click::is_hovering(rows[1], form.mouse_col, form.mouse_row)
                && !is_focused(form, 4),
        ),
        input_line(
            "Max turns",
            &max_turns_str,
            is_focused(form, 5),
            is_focused(form, 5) && form.is_editing(),
            form.edit_cursor,
            form.edit_buffer.as_deref(),
            crate::presentation::click::is_hovering(rows[2], form.mouse_col, form.mouse_row)
                && !is_focused(form, 5),
        ),
        checkbox_line(
            "Plan approval  ",
            knobs.plan_approval,
            is_focused(form, 6),
            crate::presentation::click::is_hovering(rows[3], form.mouse_col, form.mouse_row)
                && !is_focused(form, 6),
        ),
        input_line(
            "Max iterations",
            &max_iters_str,
            is_focused(form, 7),
            is_focused(form, 7) && form.is_editing(),
            form.edit_cursor,
            form.edit_buffer.as_deref(),
            crate::presentation::click::is_hovering(rows[4], form.mouse_col, form.mouse_row)
                && !is_focused(form, 7),
        ),
    ];

    f.render_widget(Paragraph::new(lines), inner);

    if form.dropdown_open && (form.focus == 3 || form.focus == 4) {
        let row_below = rows[form.focus - 3];
        let field_label = fields()[form.focus].label;
        *pending_dropdown = Some(PendingDropdown {
            below: row_below,
            field_label: field_label.to_string(),
            options: options_for(form.focus, config, agents),
        });
    }
}

fn render_shallow_researcher(
    f: &mut ratatui::Frame,
    area: Rect,
    def: &AgentDef,
    knobs: &ShallowResearcherConfig,
    form: &FormState,
    hit_registry: &mut Vec<ClickTarget>,
    config: &MuonConfig,
    agents: &AgentSettings,
    pending_dropdown: &mut Option<PendingDropdown>,
) {
    let focused = section_has_focus(form, 8, 11);
    let llm_turns_str = knobs.max_llm_turns.to_string();
    let tool_iters_str = knobs.max_tool_iters.to_string();
    let hovered = crate::presentation::click::is_hovering(area, form.mouse_col, form.mouse_row);
    let inner = agent_block("SHALLOW RESEARCHER", focused, hovered && !focused).inner(area);
    f.render_widget(
        agent_block("SHALLOW RESEARCHER", focused, hovered && !focused),
        area,
    );

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    for (i, row_rect) in rows.iter().enumerate() {
        hit_registry.push(ClickTarget {
            rect: *row_rect,
            action: ClickAction::ActivateField(8 + i),
        });
    }

    let lines: Vec<Line> = vec![
        dropdown_line(
            "Model",
            &def.model,
            is_focused(form, 8),
            crate::presentation::click::is_hovering(rows[0], form.mouse_col, form.mouse_row)
                && !is_focused(form, 8),
        ),
        dropdown_line(
            "Provider",
            &def.provider,
            is_focused(form, 9),
            crate::presentation::click::is_hovering(rows[1], form.mouse_col, form.mouse_row)
                && !is_focused(form, 9),
        ),
        input_line(
            "Max LLM turns",
            &llm_turns_str,
            is_focused(form, 10),
            is_focused(form, 10) && form.is_editing(),
            form.edit_cursor,
            form.edit_buffer.as_deref(),
            crate::presentation::click::is_hovering(rows[2], form.mouse_col, form.mouse_row)
                && !is_focused(form, 10),
        ),
        input_line(
            "Max tool iters",
            &tool_iters_str,
            is_focused(form, 11),
            is_focused(form, 11) && form.is_editing(),
            form.edit_cursor,
            form.edit_buffer.as_deref(),
            crate::presentation::click::is_hovering(rows[3], form.mouse_col, form.mouse_row)
                && !is_focused(form, 11),
        ),
    ];

    f.render_widget(Paragraph::new(lines), inner);

    if form.dropdown_open && (form.focus == 8 || form.focus == 9) {
        let row_below = rows[form.focus - 8];
        let field_label = fields()[form.focus].label;
        *pending_dropdown = Some(PendingDropdown {
            below: row_below,
            field_label: field_label.to_string(),
            options: options_for(form.focus, config, agents),
        });
    }
}

fn deep_role_line(
    role: &str,
    model: &str,
    provider: &str,
    model_field: usize,
    provider_field: usize,
    form: &FormState,
) -> Line<'static> {
    let role_focused = is_focused(form, model_field) || is_focused(form, provider_field);
    let mut spans = Vec::new();
    if role_focused {
        spans.push(Span::styled(
            "> ",
            Style::new()
                .fg(theme::border_focus())
                .add_modifier(Modifier::BOLD),
        ));
    } else {
        spans.push(Span::raw("  "));
    }
    spans.push(Span::styled(
        format!("{role:<12}"),
        if role_focused {
            Style::new()
                .fg(theme::border_focus())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(theme::text_dim())
        },
    ));
    let mf = is_focused(form, model_field);
    let pf = is_focused(form, provider_field);
    let bracket = |on: bool| {
        if on {
            Style::new().fg(theme::border_focus())
        } else {
            Style::new().fg(theme::text_dim())
        }
    };
    let val = |on: bool| {
        if on {
            Style::new()
                .fg(theme::border_focus())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(theme::accent())
        }
    };
    spans.push(Span::styled("[", bracket(mf)));
    spans.push(Span::styled(model.to_string(), val(mf)));
    spans.push(Span::styled("\u{25BC}", val(mf)));
    spans.push(Span::styled("] ", bracket(mf)));
    spans.push(Span::styled("[", bracket(pf)));
    spans.push(Span::styled(provider.to_string(), val(pf)));
    spans.push(Span::styled("\u{25BC}", val(pf)));
    spans.push(Span::styled("]", bracket(pf)));
    Line::from(spans)
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
    agents: &AgentSettings,
    knobs: &DeepResearcherConfig,
    form: &FormState,
    hit_registry: &mut Vec<ClickTarget>,
    config: &MuonConfig,
    pending_dropdown: &mut Option<PendingDropdown>,
) {
    let focused = section_has_focus(form, 12, 24);
    let hovered = crate::presentation::click::is_hovering(area, form.mouse_col, form.mouse_row);
    let block = agent_block("DEEP RESEARCHER", focused, hovered && !focused);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    let role_rows = [
        (
            0usize,
            12usize,
            13usize,
            "Orchestrator",
            agents.deep_orchestrator.model.as_str(),
            agents.deep_orchestrator.provider.as_str(),
        ),
        (
            1,
            14,
            15,
            "Planner",
            agents.planner.model.as_str(),
            agents.planner.provider.as_str(),
        ),
        (
            2,
            16,
            17,
            "Researcher",
            agents.researcher.model.as_str(),
            agents.researcher.provider.as_str(),
        ),
    ];
    for &(row, model_f, prov_f, role, model, provider) in &role_rows {
        let rect = rows[row];
        let prefix_w: u16 = 14;
        let model_w = format!("[{model}\u{25BC}] ").chars().count() as u16;
        let model_x = rect.x.saturating_add(prefix_w.min(rect.width));
        let after_prefix = rect.width.saturating_sub(prefix_w);
        let model_width = model_w.min(after_prefix).max(1);
        let prov_x = model_x.saturating_add(model_width);
        let prov_width = rect
            .width
            .saturating_sub(prefix_w.saturating_add(model_width))
            .max(1);
        hit_registry.push(ClickTarget {
            rect: Rect::new(model_x, rect.y, model_width, rect.height),
            action: ClickAction::ActivateField(model_f),
        });
        hit_registry.push(ClickTarget {
            rect: Rect::new(prov_x, rect.y, prov_width, rect.height),
            action: ClickAction::ActivateField(prov_f),
        });
        f.render_widget(
            Paragraph::new(deep_role_line(role, model, provider, model_f, prov_f, form)),
            rect,
        );
    }

    let pair = |row: Rect| {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(row)
    };

    #[allow(clippy::type_complexity)]
    let limit_pairs: [((usize, usize, &str, String), (usize, usize, &str, String)); 2] = [
        (
            (3, 18, "Orch ReAct cycles", knobs.iterations.to_string()),
            (
                3,
                21,
                "Orch tool calls",
                knobs.orchestrator_max_tool_calls.to_string(),
            ),
        ),
        (
            (4, 20, "Planner cycles", knobs.planner_max_cycles.to_string()),
            (
                4,
                22,
                "Planner tool calls",
                knobs.planner_max_tool_calls.to_string(),
            ),
        ),
    ];

    for ((row_idx, field_l, label_l, value_l), (_, field_r, label_r, value_r)) in &limit_pairs {
        let cols = pair(rows[*row_idx]);
        hit_registry.push(ClickTarget {
            rect: cols[0],
            action: ClickAction::ActivateField(*field_l),
        });
        hit_registry.push(ClickTarget {
            rect: cols[1],
            action: ClickAction::ActivateField(*field_r),
        });
        f.render_widget(
            Paragraph::new(deep_num_cell(label_l, value_l, *field_l, form, cols[0])),
            cols[0],
        );
        f.render_widget(
            Paragraph::new(deep_num_cell(label_r, value_r, *field_r, form, cols[1])),
            cols[1],
        );
    }

    let researcher_tool_calls = knobs.researcher_max_tool_calls.to_string();
    hit_registry.push(ClickTarget {
        rect: rows[5],
        action: ClickAction::ActivateField(23),
    });
    f.render_widget(
        Paragraph::new(deep_num_cell(
            "Researcher tool calls",
            &researcher_tool_calls,
            23,
            form,
            rows[5],
        )),
        rows[5],
    );

    let footer = pair(rows[6]);
    hit_registry.push(ClickTarget {
        rect: footer[0],
        action: ClickAction::ActivateField(19),
    });
    hit_registry.push(ClickTarget {
        rect: footer[1],
        action: ClickAction::ActivateField(24),
    });
    let retries = knobs.max_retries.to_string();
    f.render_widget(
        Paragraph::new(deep_num_cell("Max retries", &retries, 19, form, footer[0])),
        footer[0],
    );
    f.render_widget(
        Paragraph::new(checkbox_line(
            "Citation Verify",
            knobs.citation_verify,
            is_focused(form, 24),
            crate::presentation::click::is_hovering(footer[1], form.mouse_col, form.mouse_row)
                && !is_focused(form, 24),
        )),
        footer[1],
    );

    if form.dropdown_open && (12..=17).contains(&form.focus) {
        let grid_idx = match form.focus {
            12 | 13 => 0,
            14 | 15 => 1,
            16 | 17 => 2,
            _ => 0,
        };
        let field_label = fields()[form.focus].label;
        *pending_dropdown = Some(PendingDropdown {
            below: rows[grid_idx],
            field_label: field_label.to_string(),
            options: options_for(form.focus, config, agents),
        });
    }
}
