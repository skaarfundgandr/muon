use crate::config::{AgentsConfig, DeepResearcherConfig, MuonConfig};
use crate::presentation::click::{ClickAction, ClickTarget};
use crate::presentation::form::{FieldDef, FormState};
use crate::presentation::theme;
use crate::presentation::components::inputs::settings::dropdown_overlay::PendingDropdown;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn fields() -> &'static [FieldDef] {
    Box::leak(Box::new([
        // Intent Classifier (0-3)
        FieldDef::dropdown("IC Model", &[]),
        FieldDef::dropdown("IC Provider", &[]),
        FieldDef::number("IC Timeout"),
        FieldDef::checkbox("IC Verbose"),
        // Clarifier (4-8)
        FieldDef::dropdown("Cl Model", &[]),
        FieldDef::dropdown("Cl Provider", &[]),
        FieldDef::number("Cl Max turns"),
        FieldDef::checkbox("Cl Plan approval"),
        FieldDef::number("Cl Max iterations"),
        // Shallow (9-12)
        FieldDef::dropdown("Sh Model", &[]),
        FieldDef::dropdown("Sh Provider", &[]),
        FieldDef::number("Sh Max LLM turns"),
        FieldDef::number("Sh Max tool iters"),
        // Deep (13-20)
        FieldDef::dropdown("D Orch Model", &[]),
        FieldDef::dropdown("D Orch Provider", &[]),
        FieldDef::dropdown("D Plan Model", &[]),
        FieldDef::dropdown("D Plan Provider", &[]),
        FieldDef::dropdown("D Res Model", &[]),
        FieldDef::dropdown("D Res Provider", &[]),
        FieldDef::number("D Iterations"),
        FieldDef::checkbox("D Citation Verify"),
    ])) as &'static [FieldDef]
}

pub fn get_field(config: &AgentsConfig, index: usize) -> String {
    match index {
        0 => config.intent_classifier.model.clone(),
        1 => config.intent_classifier.provider.clone(),
        2 => config.intent_classifier.timeout_sec.to_string(),
        3 => config.intent_classifier.verbose.to_string(),
        4 => config.clarifier.model.clone(),
        5 => config.clarifier.provider.clone(),
        6 => config.clarifier.max_turns.to_string(),
        7 => config.clarifier.plan_approval.to_string(),
        8 => config.clarifier.max_iterations.to_string(),
        9 => config.shallow_researcher.model.clone(),
        10 => config.shallow_researcher.provider.clone(),
        11 => config.shallow_researcher.max_llm_turns.to_string(),
        12 => config.shallow_researcher.max_tool_iters.to_string(),
        13 => config.deep_researcher.orchestrator.model.clone(),
        14 => config.deep_researcher.orchestrator.provider.clone(),
        15 => config.deep_researcher.planner.model.clone(),
        16 => config.deep_researcher.planner.provider.clone(),
        17 => config.deep_researcher.researcher.model.clone(),
        18 => config.deep_researcher.researcher.provider.clone(),
        19 => config.deep_researcher.iterations.to_string(),
        20 => config.deep_researcher.citation_verify.to_string(),
        _ => String::new(),
    }
}

pub fn set_field(config: &mut AgentsConfig, index: usize, value: &str) {
    match index {
        0 => config.intent_classifier.model = value.to_string(),
        1 => config.intent_classifier.provider = value.to_string(),
        2 => config.intent_classifier.timeout_sec = value.parse().unwrap_or(90),
        3 => config.intent_classifier.verbose = value == "true",
        4 => config.clarifier.model = value.to_string(),
        5 => config.clarifier.provider = value.to_string(),
        6 => config.clarifier.max_turns = value.parse().unwrap_or(3),
        7 => config.clarifier.plan_approval = value == "true",
        8 => config.clarifier.max_iterations = value.parse().unwrap_or(10),
        9 => config.shallow_researcher.model = value.to_string(),
        10 => config.shallow_researcher.provider = value.to_string(),
        11 => config.shallow_researcher.max_llm_turns = value.parse().unwrap_or(10),
        12 => config.shallow_researcher.max_tool_iters = value.parse().unwrap_or(5),
        13 => config.deep_researcher.orchestrator.model = value.to_string(),
        14 => config.deep_researcher.orchestrator.provider = value.to_string(),
        15 => config.deep_researcher.planner.model = value.to_string(),
        16 => config.deep_researcher.planner.provider = value.to_string(),
        17 => config.deep_researcher.researcher.model = value.to_string(),
        18 => config.deep_researcher.researcher.provider = value.to_string(),
        19 => config.deep_researcher.iterations = value.parse().unwrap_or(2),
        20 => config.deep_researcher.citation_verify = value == "true",
        _ => {}
    }
}

pub fn toggle_field(config: &mut AgentsConfig, index: usize) {
    match index {
        3 => config.intent_classifier.verbose = !config.intent_classifier.verbose,
        7 => config.clarifier.plan_approval = !config.clarifier.plan_approval,
        20 => config.deep_researcher.citation_verify = !config.deep_researcher.citation_verify,
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
pub fn options_for(field_index: usize, config: &MuonConfig) -> Vec<String> {
    const PROVIDER_FIELDS: &[usize] = &[1, 5, 10, 14, 16, 18];
    const MODEL_FIELDS: &[usize] = &[0, 4, 9, 13, 15, 17];
    if PROVIDER_FIELDS.contains(&field_index) {
        provider_options(config)
    } else if MODEL_FIELDS.contains(&field_index) {
        let provider_idx = match field_index {
            0 | 1 => config.agents.intent_classifier.provider.clone(),
            4 | 5 => config.agents.clarifier.provider.clone(),
            9 | 10 => config.agents.shallow_researcher.provider.clone(),
            13 | 14 => config.agents.deep_researcher.orchestrator.provider.clone(),
            15 | 16 => config.agents.deep_researcher.planner.provider.clone(),
            17 | 18 => config.agents.deep_researcher.researcher.provider.clone(),
            _ => return Vec::new(),
        };
        model_options(config, &provider_idx)
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

#[allow(clippy::vec_init_then_push)]
pub fn render(f: &mut ratatui::Frame, area: Rect, config: &MuonConfig, form: &FormState, hit_registry: &mut Vec<ClickTarget>, _mouse_col: u16, _mouse_row: u16, pending_dropdown: &mut Option<PendingDropdown>) {
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
        action: ClickAction::FocusField(4),
    });
    hit_registry.push(ClickTarget {
        rect: right_chunks[0],
        action: ClickAction::FocusField(9),
    });
    hit_registry.push(ClickTarget {
        rect: right_chunks[1],
        action: ClickAction::FocusField(13),
    });

    render_intent_classifier(f, left_chunks[0], &config.agents.intent_classifier, form, hit_registry, config, pending_dropdown);
    render_clarifier(f, left_chunks[1], &config.agents.clarifier, form, hit_registry, config, pending_dropdown);
    render_shallow_researcher(f, right_chunks[0], &config.agents.shallow_researcher, form, hit_registry, config, pending_dropdown);
    render_deep_researcher(f, right_chunks[1], &config.agents.deep_researcher, form, hit_registry, config, pending_dropdown);
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
            Style::new().fg(theme::purple()).add_modifier(Modifier::BOLD),
        ))
}

fn dropdown_line<'a>(label: &'a str, value: &'a str, focused: bool, hovered: bool) -> Line<'a> {
    if focused {
        Line::from(vec![
            Span::styled("> ", Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:<14}", label), Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)),
            Span::styled("[", Style::new().fg(theme::border_focus())),
            Span::styled(value, Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)),
            Span::styled("\u{25BC}", Style::new().fg(theme::border_focus())),
            Span::styled("]", Style::new().fg(theme::border_focus())),
        ])
    } else if hovered {
        Line::from(vec![
            Span::styled(format!("{:<14}", label), Style::new().fg(crate::presentation::theme::border_hover())),
            Span::styled("[", Style::new().fg(crate::presentation::theme::border_hover())),
            Span::styled(value, Style::new().fg(theme::accent())),
            Span::styled("\u{25BC}", Style::new().fg(theme::accent())),
            Span::styled("]", Style::new().fg(crate::presentation::theme::border_hover())),
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

fn input_line<'a>(label: &'a str, value: &'a str, focused: bool, editing: bool, cursor: usize, buffer: Option<&'a str>, hovered: bool) -> Line<'a> {
    if editing {
        let buf = buffer.unwrap_or("");
        let cur = cursor.min(buf.len());
        let pre = &buf[..cur];
        let post = &buf[cur..];
        Line::from(vec![
            Span::styled("> ", Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:<14}", label), Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)),
            Span::styled("[", Style::new().fg(theme::border_focus())),
            Span::styled(pre.to_string(), Style::new().fg(theme::accent()).add_modifier(Modifier::BOLD)),
            Span::styled("\u{258E}", Style::new().fg(theme::border_focus())),
            Span::styled(post.to_string(), Style::new().fg(theme::accent()).add_modifier(Modifier::BOLD)),
            Span::styled("]", Style::new().fg(theme::border_focus())),
        ])
    } else if focused {
        Line::from(vec![
            Span::styled("> ", Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:<14}", label), Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)),
            Span::styled("[", Style::new().fg(theme::border_focus())),
            Span::styled(value, Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)),
            Span::styled("]", Style::new().fg(theme::border_focus())),
        ])
    } else if hovered {
        Line::from(vec![
            Span::styled(format!("{:<14}", label), Style::new().fg(crate::presentation::theme::border_hover())),
            Span::styled("[", Style::new().fg(crate::presentation::theme::border_hover())),
            Span::styled(value, Style::new().fg(theme::success())),
            Span::styled("]", Style::new().fg(crate::presentation::theme::border_hover())),
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
    let mark_color = if checked { theme::success() } else { theme::text_dim() };
    if focused {
        Line::from(vec![
            Span::styled("> ", Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{} ", label), Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)),
            Span::styled(mark, Style::new().fg(theme::border_focus())),
        ])
    } else if hovered {
        Line::from(vec![
            Span::styled(format!("{} ", label), Style::new().fg(crate::presentation::theme::border_hover())),
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
    cfg: &crate::config::AgentEntryConfig,
    form: &FormState,
    hit_registry: &mut Vec<ClickTarget>,
    config: &MuonConfig,
    pending_dropdown: &mut Option<PendingDropdown>,
) {
    let focused = section_has_focus(form, 0, 3);
    let timeout_str = cfg.timeout_sec.to_string();
    let hovered = crate::presentation::click::is_hovering(area, form.mouse_col, form.mouse_row);
    let inner = agent_block("INTENT CLASSIFIER", focused, hovered && !focused).inner(area);
    f.render_widget(agent_block("INTENT CLASSIFIER", focused, hovered && !focused), area);

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
            action: ClickAction::ActivateField(i),
        });
    }

    let lines: Vec<Line> = vec![
        dropdown_line("Model", &cfg.model, is_focused(form, 0), crate::presentation::click::is_hovering(rows[0], form.mouse_col, form.mouse_row) && !is_focused(form, 0)),
        dropdown_line("Provider", &cfg.provider, is_focused(form, 1), crate::presentation::click::is_hovering(rows[1], form.mouse_col, form.mouse_row) && !is_focused(form, 1)),
        input_line("Timeout (sec)", &timeout_str, is_focused(form, 2), is_focused(form, 2) && form.is_editing(), form.edit_cursor, form.edit_buffer.as_deref(), crate::presentation::click::is_hovering(rows[2], form.mouse_col, form.mouse_row) && !is_focused(form, 2)),
        checkbox_line("Verbose Output ", cfg.verbose, is_focused(form, 3), crate::presentation::click::is_hovering(rows[3], form.mouse_col, form.mouse_row) && !is_focused(form, 3)),
    ];

    f.render_widget(Paragraph::new(lines), inner);

    if form.dropdown_open && (form.focus == 0 || form.focus == 1) {
        let row_below = rows[form.focus];
        let field_label = fields()[form.focus].label;
        *pending_dropdown = Some(PendingDropdown {
            below: row_below,
            field_label: field_label.to_string(),
            options: options_for(form.focus, config),
        });
    }
}

fn render_clarifier(
    f: &mut ratatui::Frame,
    area: Rect,
    cfg: &crate::config::ClarifierConfig,
    form: &FormState,
    hit_registry: &mut Vec<ClickTarget>,
    config: &MuonConfig,
    pending_dropdown: &mut Option<PendingDropdown>,
) {
    let focused = section_has_focus(form, 4, 8);
    let max_turns_str = cfg.max_turns.to_string();
    let max_iters_str = cfg.max_iterations.to_string();
    let hovered = crate::presentation::click::is_hovering(area, form.mouse_col, form.mouse_row);
    let inner = agent_block("CLARIFIER (HITL)", focused, hovered && !focused).inner(area);
    f.render_widget(agent_block("CLARIFIER (HITL)", focused, hovered && !focused), area);

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
            action: ClickAction::ActivateField(4 + i),
        });
    }

    let lines: Vec<Line> = vec![
        dropdown_line("Model", &cfg.model, is_focused(form, 4), crate::presentation::click::is_hovering(rows[0], form.mouse_col, form.mouse_row) && !is_focused(form, 4)),
        dropdown_line("Provider", &cfg.provider, is_focused(form, 5), crate::presentation::click::is_hovering(rows[1], form.mouse_col, form.mouse_row) && !is_focused(form, 5)),
        input_line("Max turns", &max_turns_str, is_focused(form, 6), is_focused(form, 6) && form.is_editing(), form.edit_cursor, form.edit_buffer.as_deref(), crate::presentation::click::is_hovering(rows[2], form.mouse_col, form.mouse_row) && !is_focused(form, 6)),
        checkbox_line("Plan approval  ", cfg.plan_approval, is_focused(form, 7), crate::presentation::click::is_hovering(rows[3], form.mouse_col, form.mouse_row) && !is_focused(form, 7)),
        input_line("Max iterations", &max_iters_str, is_focused(form, 8), is_focused(form, 8) && form.is_editing(), form.edit_cursor, form.edit_buffer.as_deref(), crate::presentation::click::is_hovering(rows[4], form.mouse_col, form.mouse_row) && !is_focused(form, 8)),
    ];

    f.render_widget(Paragraph::new(lines), inner);

    if form.dropdown_open && (form.focus == 4 || form.focus == 5) {
        let row_below = rows[form.focus - 4];
        let field_label = fields()[form.focus].label;
        *pending_dropdown = Some(PendingDropdown {
            below: row_below,
            field_label: field_label.to_string(),
            options: options_for(form.focus, config),
        });
    }
}

fn render_shallow_researcher(
    f: &mut ratatui::Frame,
    area: Rect,
    cfg: &crate::config::ShallowResearcherConfig,
    form: &FormState,
    hit_registry: &mut Vec<ClickTarget>,
    config: &MuonConfig,
    pending_dropdown: &mut Option<PendingDropdown>,
) {
    let focused = section_has_focus(form, 9, 12);
    let llm_turns_str = cfg.max_llm_turns.to_string();
    let tool_iters_str = cfg.max_tool_iters.to_string();
    let hovered = crate::presentation::click::is_hovering(area, form.mouse_col, form.mouse_row);
    let inner = agent_block("SHALLOW RESEARCHER", focused, hovered && !focused).inner(area);
    f.render_widget(agent_block("SHALLOW RESEARCHER", focused, hovered && !focused), area);

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
            action: ClickAction::ActivateField(9 + i),
        });
    }

    let lines: Vec<Line> = vec![
        dropdown_line("Model", &cfg.model, is_focused(form, 9), crate::presentation::click::is_hovering(rows[0], form.mouse_col, form.mouse_row) && !is_focused(form, 9)),
        dropdown_line("Provider", &cfg.provider, is_focused(form, 10), crate::presentation::click::is_hovering(rows[1], form.mouse_col, form.mouse_row) && !is_focused(form, 10)),
        input_line("Max LLM turns", &llm_turns_str, is_focused(form, 11), is_focused(form, 11) && form.is_editing(), form.edit_cursor, form.edit_buffer.as_deref(), crate::presentation::click::is_hovering(rows[2], form.mouse_col, form.mouse_row) && !is_focused(form, 11)),
        input_line("Max tool iters", &tool_iters_str, is_focused(form, 12), is_focused(form, 12) && form.is_editing(), form.edit_cursor, form.edit_buffer.as_deref(), crate::presentation::click::is_hovering(rows[3], form.mouse_col, form.mouse_row) && !is_focused(form, 12)),
    ];

    f.render_widget(Paragraph::new(lines), inner);

    if form.dropdown_open && (form.focus == 9 || form.focus == 10) {
        let row_below = rows[form.focus - 9];
        let field_label = fields()[form.focus].label;
        *pending_dropdown = Some(PendingDropdown {
            below: row_below,
            field_label: field_label.to_string(),
            options: options_for(form.focus, config),
        });
    }
}

fn render_deep_researcher(
    f: &mut ratatui::Frame,
    area: Rect,
    cfg: &DeepResearcherConfig,
    form: &FormState,
    hit_registry: &mut Vec<ClickTarget>,
    config: &MuonConfig,
    pending_dropdown: &mut Option<PendingDropdown>,
) {
    let focused = section_has_focus(form, 13, 20);
    let hovered = crate::presentation::click::is_hovering(area, form.mouse_col, form.mouse_row);
    let block = agent_block("DEEP RESEARCHER", focused, hovered && !focused);
    let inner = block.inner(area);
    f.render_widget(block, area);
    let grid = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    let field_map: [Option<usize>; 5] = [None, None, None, Some(19), Some(20)];
    for (i, row_rect) in grid.iter().enumerate() {
        if let Some(field_idx) = field_map[i] {
            hit_registry.push(ClickTarget {
                rect: *row_rect,
                action: ClickAction::ActivateField(field_idx),
            });
        }
    }

    // Split click targets for Orchestrator (13, 14), Planner (15, 16), and Researcher (17, 18)
    let orch_focused = is_focused(form, 13) || is_focused(form, 14);
    let plan_focused = is_focused(form, 15) || is_focused(form, 16);
    let res_focused = is_focused(form, 17) || is_focused(form, 18);

    let rows_data = [
        (0, 13, 14, cfg.orchestrator.model.len() as u16, cfg.orchestrator.provider.len() as u16, orch_focused),
        (1, 15, 16, cfg.planner.model.len() as u16, cfg.planner.provider.len() as u16, plan_focused),
        (2, 17, 18, cfg.researcher.model.len() as u16, cfg.researcher.provider.len() as u16, res_focused),
    ];
    for &(grid_idx, model_field, provider_field, model_len, provider_len, focused) in &rows_data {
        let row_rect = grid[grid_idx];
        let label_offset = if focused { 2 } else { 0 };

        // Model clickable area (label + model bracket)
        let model_width = (label_offset + 17 + model_len).min(row_rect.width);
        hit_registry.push(ClickTarget {
            rect: Rect::new(row_rect.x, row_rect.y, model_width, row_rect.height),
            action: ClickAction::ActivateField(model_field),
        });

        // Provider clickable area (provider bracket)
        let provider_start = label_offset + 14 + 1 + model_len + 1 + 13;
        if provider_start < row_rect.width {
            let provider_width = (provider_len + 3).min(row_rect.width - provider_start);
            hit_registry.push(ClickTarget {
                rect: Rect::new(row_rect.x + provider_start, row_rect.y, provider_width, row_rect.height),
                action: ClickAction::ActivateField(provider_field),
            });
        }
    }

    // Orchestrator (13, 14)
    let mut orch_spans = Vec::new();
    if orch_focused {
        orch_spans.push(Span::styled("> ", Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)));
    }
    orch_spans.push(Span::styled(
        format!("{:<14}", "Orchestrator"),
        if orch_focused {
            Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(theme::text_dim())
        },
    ));
    orch_spans.push(Span::styled("[", if is_focused(form, 13) { Style::new().fg(theme::border_focus()) } else { Style::new().fg(theme::text_dim()) }));
    orch_spans.push(Span::styled(&cfg.orchestrator.model, if is_focused(form, 13) { Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD) } else { Style::new().fg(theme::accent()) }));
    orch_spans.push(Span::styled("\u{25BC}", if is_focused(form, 13) { Style::new().fg(theme::border_focus()) } else { Style::new().fg(theme::accent()) }));
    orch_spans.push(Span::styled("] Provider: [", if is_focused(form, 14) { Style::new().fg(theme::border_focus()) } else { Style::new().fg(theme::text_dim()) }));
    orch_spans.push(Span::styled(&cfg.orchestrator.provider, if is_focused(form, 14) { Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD) } else { Style::new().fg(theme::accent()) }));
    orch_spans.push(Span::styled("\u{25BC}", if is_focused(form, 14) { Style::new().fg(theme::border_focus()) } else { Style::new().fg(theme::accent()) }));
    orch_spans.push(Span::styled("]", if is_focused(form, 14) { Style::new().fg(theme::border_focus()) } else { Style::new().fg(theme::text_dim()) }));
    let orch_line = Line::from(orch_spans);
    f.render_widget(Paragraph::new(orch_line), grid[0]);

    // Planner (15, 16)
    let mut plan_spans = Vec::new();
    if plan_focused {
        plan_spans.push(Span::styled("> ", Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)));
    }
    plan_spans.push(Span::styled(
        format!("{:<14}", "Planner"),
        if plan_focused {
            Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(theme::text_dim())
        },
    ));
    plan_spans.push(Span::styled("[", if is_focused(form, 15) { Style::new().fg(theme::border_focus()) } else { Style::new().fg(theme::text_dim()) }));
    plan_spans.push(Span::styled(&cfg.planner.model, if is_focused(form, 15) { Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD) } else { Style::new().fg(theme::accent()) }));
    plan_spans.push(Span::styled("\u{25BC}", if is_focused(form, 15) { Style::new().fg(theme::border_focus()) } else { Style::new().fg(theme::accent()) }));
    plan_spans.push(Span::styled("] Provider: [", if is_focused(form, 16) { Style::new().fg(theme::border_focus()) } else { Style::new().fg(theme::text_dim()) }));
    plan_spans.push(Span::styled(&cfg.planner.provider, if is_focused(form, 16) { Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD) } else { Style::new().fg(theme::accent()) }));
    plan_spans.push(Span::styled("\u{25BC}", if is_focused(form, 16) { Style::new().fg(theme::border_focus()) } else { Style::new().fg(theme::accent()) }));
    plan_spans.push(Span::styled("]", if is_focused(form, 16) { Style::new().fg(theme::border_focus()) } else { Style::new().fg(theme::text_dim()) }));
    let plan_line = Line::from(plan_spans);
    f.render_widget(Paragraph::new(plan_line), grid[1]);

    // Researcher (17, 18)
    let mut res_spans = Vec::new();
    if res_focused {
        res_spans.push(Span::styled("> ", Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)));
    }
    res_spans.push(Span::styled(
        format!("{:<14}", "Researcher"),
        if res_focused {
            Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(theme::text_dim())
        },
    ));
    res_spans.push(Span::styled("[", if is_focused(form, 17) { Style::new().fg(theme::border_focus()) } else { Style::new().fg(theme::text_dim()) }));
    res_spans.push(Span::styled(&cfg.researcher.model, if is_focused(form, 17) { Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD) } else { Style::new().fg(theme::accent()) }));
    res_spans.push(Span::styled("\u{25BC}", if is_focused(form, 17) { Style::new().fg(theme::border_focus()) } else { Style::new().fg(theme::accent()) }));
    res_spans.push(Span::styled("] Provider: [", if is_focused(form, 18) { Style::new().fg(theme::border_focus()) } else { Style::new().fg(theme::text_dim()) }));
    res_spans.push(Span::styled(&cfg.researcher.provider, if is_focused(form, 18) { Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD) } else { Style::new().fg(theme::accent()) }));
    res_spans.push(Span::styled("\u{25BC}", if is_focused(form, 18) { Style::new().fg(theme::border_focus()) } else { Style::new().fg(theme::accent()) }));
    res_spans.push(Span::styled("]", if is_focused(form, 18) { Style::new().fg(theme::border_focus()) } else { Style::new().fg(theme::text_dim()) }));
    let res_line = Line::from(res_spans);
    f.render_widget(Paragraph::new(res_line), grid[2]);

    // Iterations (19)
    let iter_str = cfg.iterations.to_string();
    let iter_line = input_line("Iterations", &iter_str, is_focused(form, 19), is_focused(form, 19) && form.is_editing(), form.edit_cursor, form.edit_buffer.as_deref(), crate::presentation::click::is_hovering(grid[3], form.mouse_col, form.mouse_row) && !is_focused(form, 19));
    f.render_widget(Paragraph::new(iter_line), grid[3]);

    // Citation Verify (20)
    let cit_line = checkbox_line("Citation Verify", cfg.citation_verify, is_focused(form, 20), crate::presentation::click::is_hovering(grid[4], form.mouse_col, form.mouse_row) && !is_focused(form, 20));
    f.render_widget(Paragraph::new(cit_line), grid[4]);

    if form.dropdown_open && (13..=18).contains(&form.focus) {
        let grid_idx = match form.focus {
            13 | 14 => 0,
            15 | 16 => 1,
            17 | 18 => 2,
            _ => 0,
        };
        let field_label = fields()[form.focus].label;
        *pending_dropdown = Some(PendingDropdown {
            below: grid[grid_idx],
            field_label: field_label.to_string(),
            options: options_for(form.focus, config),
        });
    }
}
