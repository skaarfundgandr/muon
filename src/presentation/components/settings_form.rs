use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use crate::presentation::theme::{ACCENT, BORDER, PURPLE, SUCCESS, TEXT_DIM};

#[allow(clippy::vec_init_then_push)]
pub fn render(f: &mut ratatui::Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    // Tab bar
    let tab_bar = Line::from(vec![
        Span::styled("⚙ ", Style::new().fg(ACCENT)),
        Span::styled("[Agents]", Style::new().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled("🛠 Tools", Style::new().fg(TEXT_DIM)),
        Span::raw("  "),
        Span::styled("📁 Data Sources", Style::new().fg(TEXT_DIM)),
        Span::raw("  "),
        Span::styled("📺 Display", Style::new().fg(TEXT_DIM)),
        Span::raw("  "),
        Span::styled("⚡ Advanced", Style::new().fg(TEXT_DIM)),
    ]);
    f.render_widget(Paragraph::new(tab_bar), chunks[0]);

    // Two-column grid for 4 agents
    let grid = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Left column: Intent Classifier + Clarifier
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(grid[0]);

    // Right column: Shallow Researcher + Deep Researcher
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(grid[1]);

    render_intent_classifier(f, left_chunks[0]);
    render_clarifier(f, left_chunks[1]);
    render_shallow_researcher(f, right_chunks[0]);
    render_deep_researcher(f, right_chunks[1]);
}

fn agent_block<'a>(title: &'a str) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(BORDER))
        .title(Span::styled(
            format!(" {} ", title),
            Style::new().fg(PURPLE).add_modifier(Modifier::BOLD),
        ))
}

fn dropdown_line<'a>(label: &'a str, value: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("{:<14}", label), Style::new().fg(TEXT_DIM)),
        Span::styled("[", Style::new().fg(TEXT_DIM)),
        Span::styled(value, Style::new().fg(ACCENT)),
        Span::styled(" ▼", Style::new().fg(ACCENT)),
        Span::styled("]", Style::new().fg(TEXT_DIM)),
    ])
}

fn input_line<'a>(label: &'a str, value: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("{:<14}", label), Style::new().fg(TEXT_DIM)),
        Span::styled(value, Style::new().fg(SUCCESS)),
    ])
}

fn render_intent_classifier(f: &mut ratatui::Frame, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        "1. INTENT CLASSIFIER",
        Style::new().fg(PURPLE).add_modifier(Modifier::BOLD),
    )));
    lines.push(dropdown_line("Model", "glm-5.2"));
    lines.push(dropdown_line("Provider", "opencode-go"));
    lines.push(input_line("Timeout (sec)", "90"));
    lines.push(Line::from(vec![
        Span::styled("Verbose Output ", Style::new().fg(TEXT_DIM)),
        Span::styled("✗", Style::new().fg(TEXT_DIM)),
    ]));

    f.render_widget(Paragraph::new(lines).block(agent_block("INTENT CLASSIFIER")), area);
}

fn render_clarifier(f: &mut ratatui::Frame, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        "2. CLARIFIER (HITL)",
        Style::new().fg(PURPLE).add_modifier(Modifier::BOLD),
    )));
    lines.push(dropdown_line("Model", "glm-5.2"));
    lines.push(dropdown_line("Provider", "opencode-go"));
    lines.push(input_line("Max turns", "3"));
    lines.push(Line::from(vec![
        Span::styled("Plan approval  ", Style::new().fg(TEXT_DIM)),
        Span::styled("✓", Style::new().fg(SUCCESS)),
    ]));
    lines.push(input_line("Max iterations", "10"));

    f.render_widget(Paragraph::new(lines).block(agent_block("CLARIFIER (HITL)")), area);
}

fn render_shallow_researcher(f: &mut ratatui::Frame, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        "3. SHALLOW RESEARCHER",
        Style::new().fg(PURPLE).add_modifier(Modifier::BOLD),
    )));
    lines.push(dropdown_line("Model", "glm-5.2"));
    lines.push(dropdown_line("Provider", "NeuralWatt"));
    lines.push(input_line("Max LLM turns", "10"));
    lines.push(input_line("Max tool iters", "5"));

    f.render_widget(Paragraph::new(lines).block(agent_block("SHALLOW RESEARCHER")), area);
}

fn render_deep_researcher(f: &mut ratatui::Frame, area: Rect) {
    let inner = area.inner(ratatui::layout::Margin::new(1, 0));
    let grid = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // title + subagent grid
            Constraint::Length(1),  // iterations
            Constraint::Length(1),  // citation verify
        ])
        .split(inner);

    // Title
    let title = Line::from(Span::styled(
        "4. DEEP RESEARCHER",
        Style::new().fg(PURPLE).add_modifier(Modifier::BOLD),
    ));
    f.render_widget(Paragraph::new(title), grid[0]);

    // Subagent grid: 4 rows (Orchestrator, Planner, Researcher, Writer)
    let sub_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(grid[1]);

    let subagent_data: &[(&str, &str, &str)] = &[
        ("Orchestrator", "glm-5.2", "opencode-go"),
        ("Planner", "glm-5.2-short", "NeuralWatt"),
        ("Researcher", "glm-5.2-flex", "NeuralWatt"),
        ("Writer", "deepseek-v4-pro", "opencode-go"),
    ];

    for (i, (name, model, provider)) in subagent_data.iter().enumerate() {
        let row = Line::from(vec![
            Span::styled(format!("  {:<13}", name), Style::new().fg(TEXT_DIM)),
            Span::styled("[", Style::new().fg(TEXT_DIM)),
            Span::styled(*model, Style::new().fg(ACCENT)),
            Span::styled(" ▼", Style::new().fg(ACCENT)),
            Span::styled("] ", Style::new().fg(TEXT_DIM)),
            Span::styled("Prov: ", Style::new().fg(TEXT_DIM)),
            Span::styled("[", Style::new().fg(TEXT_DIM)),
            Span::styled(*provider, Style::new().fg(ACCENT)),
            Span::styled(" ▼", Style::new().fg(ACCENT)),
            Span::styled("]", Style::new().fg(TEXT_DIM)),
        ]);
        f.render_widget(Paragraph::new(row), sub_rows[i]);
    }

    // Iterations + Citation verify
    let iter_line = Line::from(vec![
        Span::styled("Iterations:     ", Style::new().fg(TEXT_DIM)),
        Span::styled("[2]", Style::new().fg(SUCCESS)),
    ]);
    f.render_widget(Paragraph::new(iter_line), grid[2]);

    // Border around the whole area
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(BORDER))
        .title(Span::styled(
            " DEEP RESEARCHER ",
            Style::new().fg(PURPLE).add_modifier(Modifier::BOLD),
        ));
    f.render_widget(block, area);
}
