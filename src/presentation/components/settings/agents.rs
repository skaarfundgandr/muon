use crate::config::{AgentsConfig, DeepResearcherConfig};
use crate::presentation::form::{FieldDef, FormState};
use crate::presentation::theme::{ACCENT, BORDER, BORDER_FOCUS, PURPLE, SUCCESS, TEXT_DIM};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

const MODELS: &[&str] = &[
    "glm-5.2",
    "glm-5.2-short",
    "glm-5.2-flex",
    "gpt-4o",
    "claude-3.5-sonnet",
    "gemini-2.0-flash",
];
const PROVIDERS: &[&str] = &["opencode-go", "NeuralWatt", "ClinePass"];

pub fn fields() -> &'static [FieldDef] {
    Box::leak(Box::new([
        // Intent Classifier (0-3)
        FieldDef::dropdown("IC Model", MODELS),
        FieldDef::dropdown("IC Provider", PROVIDERS),
        FieldDef::number("IC Timeout"),
        FieldDef::checkbox("IC Verbose"),
        // Clarifier (4-8)
        FieldDef::dropdown("Cl Model", MODELS),
        FieldDef::dropdown("Cl Provider", PROVIDERS),
        FieldDef::number("Cl Max turns"),
        FieldDef::checkbox("Cl Plan approval"),
        FieldDef::number("Cl Max iterations"),
        // Shallow (9-12)
        FieldDef::dropdown("Sh Model", MODELS),
        FieldDef::dropdown("Sh Provider", PROVIDERS),
        FieldDef::number("Sh Max LLM turns"),
        FieldDef::number("Sh Max tool iters"),
        // Deep (13-20)
        FieldDef::dropdown("D Orch Model", MODELS),
        FieldDef::dropdown("D Orch Provider", PROVIDERS),
        FieldDef::dropdown("D Plan Model", MODELS),
        FieldDef::dropdown("D Plan Provider", PROVIDERS),
        FieldDef::dropdown("D Res Model", MODELS),
        FieldDef::dropdown("D Res Provider", PROVIDERS),
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

fn is_focused(form: &FormState, index: usize) -> bool {
    form.focus == index
}

fn section_has_focus(form: &FormState, start: usize, end: usize) -> bool {
    (start..=end).any(|i| is_focused(form, i))
}

#[allow(clippy::vec_init_then_push)]
pub fn render(f: &mut ratatui::Frame, area: Rect, config: &AgentsConfig, form: &FormState) {
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

    render_intent_classifier(f, left_chunks[0], &config.intent_classifier, form);
    render_clarifier(f, left_chunks[1], &config.clarifier, form);
    render_shallow_researcher(f, right_chunks[0], &config.shallow_researcher, form);
    render_deep_researcher(f, right_chunks[1], &config.deep_researcher, form);
}

fn agent_block<'a>(title: &'a str, focused: bool) -> Block<'a> {
    let border_color = if focused { BORDER_FOCUS } else { BORDER };
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(border_color))
        .title(Span::styled(
            format!(" {} ", title),
            Style::new().fg(PURPLE).add_modifier(Modifier::BOLD),
        ))
}

fn dropdown_line<'a>(label: &'a str, value: &'a str, focused: bool) -> Line<'a> {
    if focused {
        Line::from(vec![
            Span::styled("> ", Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:<14}", label), Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)),
            Span::styled("[", Style::new().fg(BORDER_FOCUS)),
            Span::styled(value, Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)),
            Span::styled("\u{25BC}", Style::new().fg(BORDER_FOCUS)),
            Span::styled("]", Style::new().fg(BORDER_FOCUS)),
        ])
    } else {
        Line::from(vec![
            Span::styled(format!("{:<14}", label), Style::new().fg(TEXT_DIM)),
            Span::styled("[", Style::new().fg(TEXT_DIM)),
            Span::styled(value, Style::new().fg(ACCENT)),
            Span::styled("\u{25BC}", Style::new().fg(ACCENT)),
            Span::styled("]", Style::new().fg(TEXT_DIM)),
        ])
    }
}

fn input_line<'a>(label: &'a str, value: &'a str, focused: bool) -> Line<'a> {
    if focused {
        Line::from(vec![
            Span::styled("> ", Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:<14}", label), Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)),
            Span::styled("[", Style::new().fg(BORDER_FOCUS)),
            Span::styled(value, Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)),
            Span::styled("]", Style::new().fg(BORDER_FOCUS)),
        ])
    } else {
        Line::from(vec![
            Span::styled(format!("{:<14}", label), Style::new().fg(TEXT_DIM)),
            Span::styled("[", Style::new().fg(TEXT_DIM)),
            Span::styled(value, Style::new().fg(SUCCESS)),
            Span::styled("]", Style::new().fg(TEXT_DIM)),
        ])
    }
}

fn checkbox_line(label: &str, checked: bool, focused: bool) -> Line<'_> {
    let mark = if checked { "[\u{2713}]" } else { "[ ]" };
    let mark_color = if checked { SUCCESS } else { TEXT_DIM };
    if focused {
        Line::from(vec![
            Span::styled("> ", Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{} ", label), Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)),
            Span::styled(mark, Style::new().fg(BORDER_FOCUS)),
        ])
    } else {
        Line::from(vec![
            Span::styled(format!("{} ", label), Style::new().fg(TEXT_DIM)),
            Span::styled(mark, Style::new().fg(mark_color)),
        ])
    }
}

fn render_intent_classifier(
    f: &mut ratatui::Frame,
    area: Rect,
    cfg: &crate::config::AgentEntryConfig,
    form: &FormState,
) {
    let focused = section_has_focus(form, 0, 3);
    let timeout_str = cfg.timeout_sec.to_string();
    let lines: Vec<Line> = vec![
        dropdown_line("Model", &cfg.model, is_focused(form, 0)),
        dropdown_line("Provider", &cfg.provider, is_focused(form, 1)),
        input_line("Timeout (sec)", &timeout_str, is_focused(form, 2)),
        checkbox_line("Verbose Output ", cfg.verbose, is_focused(form, 3)),
    ];

    f.render_widget(
        Paragraph::new(lines).block(agent_block("INTENT CLASSIFIER", focused)),
        area,
    );
}

fn render_clarifier(
    f: &mut ratatui::Frame,
    area: Rect,
    cfg: &crate::config::ClarifierConfig,
    form: &FormState,
) {
    let focused = section_has_focus(form, 4, 8);
    let max_turns_str = cfg.max_turns.to_string();
    let max_iters_str = cfg.max_iterations.to_string();
    let lines: Vec<Line> = vec![
        dropdown_line("Model", &cfg.model, is_focused(form, 4)),
        dropdown_line("Provider", &cfg.provider, is_focused(form, 5)),
        input_line("Max turns", &max_turns_str, is_focused(form, 6)),
        checkbox_line("Plan approval  ", cfg.plan_approval, is_focused(form, 7)),
        input_line("Max iterations", &max_iters_str, is_focused(form, 8)),
    ];

    f.render_widget(
        Paragraph::new(lines).block(agent_block("CLARIFIER (HITL)", focused)),
        area,
    );
}

fn render_shallow_researcher(
    f: &mut ratatui::Frame,
    area: Rect,
    cfg: &crate::config::ShallowResearcherConfig,
    form: &FormState,
) {
    let focused = section_has_focus(form, 9, 12);
    let llm_turns_str = cfg.max_llm_turns.to_string();
    let tool_iters_str = cfg.max_tool_iters.to_string();
    let lines: Vec<Line> = vec![
        dropdown_line("Model", &cfg.model, is_focused(form, 9)),
        dropdown_line("Provider", &cfg.provider, is_focused(form, 10)),
        input_line("Max LLM turns", &llm_turns_str, is_focused(form, 11)),
        input_line("Max tool iters", &tool_iters_str, is_focused(form, 12)),
    ];

    f.render_widget(
        Paragraph::new(lines).block(agent_block("SHALLOW RESEARCHER", focused)),
        area,
    );
}

fn render_deep_researcher(
    f: &mut ratatui::Frame,
    area: Rect,
    cfg: &DeepResearcherConfig,
    form: &FormState,
) {
    let focused = section_has_focus(form, 13, 20);
    let block = agent_block("DEEP RESEARCHER", focused);
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
            Constraint::Length(1),
        ])
        .split(inner);

    let title = Line::from(Span::styled(
        "4. DEEP RESEARCHER",
        Style::new().fg(PURPLE).add_modifier(Modifier::BOLD),
    ));
    f.render_widget(Paragraph::new(title), grid[0]);

    // Orchestrator (13, 14)
    let orch_line = Line::from(vec![
        Span::styled(
            format!("  {:<13}", "Orchestrator"),
            if is_focused(form, 13) || is_focused(form, 14) {
                Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(TEXT_DIM)
            },
        ),
        Span::styled("[", if is_focused(form, 13) { Style::new().fg(BORDER_FOCUS) } else { Style::new().fg(TEXT_DIM) }),
        Span::styled(&cfg.orchestrator.model, if is_focused(form, 13) { Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD) } else { Style::new().fg(ACCENT) }),
        Span::styled("\u{25BC}", if is_focused(form, 13) { Style::new().fg(BORDER_FOCUS) } else { Style::new().fg(ACCENT) }),
        Span::styled("] Provider: [", if is_focused(form, 14) { Style::new().fg(BORDER_FOCUS) } else { Style::new().fg(TEXT_DIM) }),
        Span::styled(&cfg.orchestrator.provider, if is_focused(form, 14) { Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD) } else { Style::new().fg(ACCENT) }),
        Span::styled("\u{25BC}", if is_focused(form, 14) { Style::new().fg(BORDER_FOCUS) } else { Style::new().fg(ACCENT) }),
        Span::styled("]", if is_focused(form, 14) { Style::new().fg(BORDER_FOCUS) } else { Style::new().fg(TEXT_DIM) }),
    ]);
    f.render_widget(Paragraph::new(orch_line), grid[1]);

    // Planner (15, 16)
    let plan_line = Line::from(vec![
        Span::styled(
            format!("  {:<13}", "Planner"),
            if is_focused(form, 15) || is_focused(form, 16) {
                Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(TEXT_DIM)
            },
        ),
        Span::styled("[", if is_focused(form, 15) { Style::new().fg(BORDER_FOCUS) } else { Style::new().fg(TEXT_DIM) }),
        Span::styled(&cfg.planner.model, if is_focused(form, 15) { Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD) } else { Style::new().fg(ACCENT) }),
        Span::styled("\u{25BC}", if is_focused(form, 15) { Style::new().fg(BORDER_FOCUS) } else { Style::new().fg(ACCENT) }),
        Span::styled("] Provider: [", if is_focused(form, 16) { Style::new().fg(BORDER_FOCUS) } else { Style::new().fg(TEXT_DIM) }),
        Span::styled(&cfg.planner.provider, if is_focused(form, 16) { Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD) } else { Style::new().fg(ACCENT) }),
        Span::styled("\u{25BC}", if is_focused(form, 16) { Style::new().fg(BORDER_FOCUS) } else { Style::new().fg(ACCENT) }),
        Span::styled("]", if is_focused(form, 16) { Style::new().fg(BORDER_FOCUS) } else { Style::new().fg(TEXT_DIM) }),
    ]);
    f.render_widget(Paragraph::new(plan_line), grid[2]);

    // Researcher (17, 18)
    let res_line = Line::from(vec![
        Span::styled(
            format!("  {:<13}", "Researcher"),
            if is_focused(form, 17) || is_focused(form, 18) {
                Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(TEXT_DIM)
            },
        ),
        Span::styled("[", if is_focused(form, 17) { Style::new().fg(BORDER_FOCUS) } else { Style::new().fg(TEXT_DIM) }),
        Span::styled(&cfg.researcher.model, if is_focused(form, 17) { Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD) } else { Style::new().fg(ACCENT) }),
        Span::styled("\u{25BC}", if is_focused(form, 17) { Style::new().fg(BORDER_FOCUS) } else { Style::new().fg(ACCENT) }),
        Span::styled("] Provider: [", if is_focused(form, 18) { Style::new().fg(BORDER_FOCUS) } else { Style::new().fg(TEXT_DIM) }),
        Span::styled(&cfg.researcher.provider, if is_focused(form, 18) { Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD) } else { Style::new().fg(ACCENT) }),
        Span::styled("\u{25BC}", if is_focused(form, 18) { Style::new().fg(BORDER_FOCUS) } else { Style::new().fg(ACCENT) }),
        Span::styled("]", if is_focused(form, 18) { Style::new().fg(BORDER_FOCUS) } else { Style::new().fg(TEXT_DIM) }),
    ]);
    f.render_widget(Paragraph::new(res_line), grid[3]);

    // Iterations (19)
    let iter_str = cfg.iterations.to_string();
    let iter_line = input_line("Iterations", &iter_str, is_focused(form, 19));
    f.render_widget(Paragraph::new(iter_line), grid[4]);

    // Citation Verify (20)
    let cit_line = checkbox_line("Citation Verify", cfg.citation_verify, is_focused(form, 20));
    f.render_widget(Paragraph::new(cit_line), grid[5]);
}
