use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::application::pipeline::PipelineStage;
use crate::presentation::theme;

/// Function-pointer signature for rendering the clarifier sub-panel inside the
/// pipeline routing graph. Carries the pending question (if any) and the
/// in-progress response buffer text.
pub type ClarifierRenderer = fn(&mut ratatui::Frame, Rect, Option<&str>, &str, u16, u16, Option<&str>, bool) -> Option<Rect>;

#[allow(clippy::too_many_arguments)]
pub fn render_horizontal(
    f: &mut ratatui::Frame,
    area: Rect,
    clarifier: Option<ClarifierRenderer>,
    clarifier_question: Option<&str>,
    clarifier_response: &str,
    mouse_col: u16,
    mouse_row: u16,
    pipeline: &crate::application::pipeline::PipelineState,
    clarifier_input_rect: &mut Option<Rect>,
    clarifier_log: Option<&str>,
    clarifier_focused: bool,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme::border()))
        .title(Span::styled(
            " PIPELINE ROUTING ",
            Style::new().fg(theme::text_main()).add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(inner);

    let nodes_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(3),
            Constraint::Percentage(30),
            Constraint::Length(3),
            Constraint::Percentage(30),
        ])
        .split(chunks[0]);

    let stage = pipeline.stage;
    let (ic_status, ic_color) = match stage {
        PipelineStage::Idle => ("○ Pending", theme::text_dim()),
        PipelineStage::IntentClassification => ("◉ Classifying", theme::accent()),
        _ => ("✓ Complete", theme::success()),
    };
    let (dr_status, dr_color) = match stage {
        PipelineStage::Idle | PipelineStage::IntentClassification => {
            ("○ Awaiting routing", theme::text_dim())
        }
        PipelineStage::Clarification => ("◉ Awaiting input", theme::warning()),
        PipelineStage::ShallowResearch => ("◉ Shallow researching", theme::accent()),
        PipelineStage::DeepResearch => ("◉ Research \u{2192} Deep", theme::accent()),
        _ => ("✓ Complete", theme::success()),
    };
    let (deep_status, deep_color) = match stage {
        PipelineStage::DeepResearch => ("◉ Deep researching", theme::accent()),
        PipelineStage::Complete => ("\u{2713} Complete", theme::success()),
        PipelineStage::Cancelled => ("\u{2717} Cancelled", theme::error()),
        PipelineStage::Failed => ("\u{2717} Failed", theme::error()),
        _ => ("○ Pending", theme::text_dim()),
    };

    let arrow1_active = !matches!(
        stage,
        PipelineStage::Idle | PipelineStage::IntentClassification
    );
    let arrow2_active = matches!(
        stage,
        PipelineStage::ShallowResearch
            | PipelineStage::DeepResearch
            | PipelineStage::Complete
            | PipelineStage::Cancelled
            | PipelineStage::Failed
    );

    render_horizontal_node(f, nodes_row[0], "Intent Classifier", ic_status, ic_color);
    render_horizontal_arrow(f, nodes_row[1], arrow1_active);
    render_horizontal_node(f, nodes_row[2], "Depth Router", dr_status, dr_color);
    render_horizontal_arrow(f, nodes_row[3], arrow2_active);
    render_horizontal_node(f, nodes_row[4], "Deep Researcher", deep_status, deep_color);

    if let Some(clarifier_fn) = clarifier {
        *clarifier_input_rect = clarifier_fn(f, chunks[1], clarifier_question, clarifier_response, mouse_col, mouse_row, clarifier_log, clarifier_focused);
    }
}

fn render_horizontal_node(
    f: &mut ratatui::Frame,
    area: Rect,
    title: &str,
    status: &str,
    color: ratatui::style::Color,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme::border()));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let title_line = Line::from(Span::styled(
        title,
        Style::new().fg(theme::text_main()).add_modifier(Modifier::BOLD),
    ));
    let status_line = Line::from(Span::styled(status, Style::new().fg(color)));

    f.render_widget(
        Paragraph::new(title_line),
        Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: 1,
        },
    );
    f.render_widget(
        Paragraph::new(status_line),
        Rect {
            x: inner.x,
            y: inner.y + 1,
            width: inner.width,
            height: 1,
        },
    );
}

fn render_horizontal_arrow(f: &mut ratatui::Frame, area: Rect, active: bool) {
    let style = if active {
        Style::new().fg(theme::accent()).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(theme::border())
    };
    let line = Line::from(Span::styled("→", style));
    let para = Paragraph::new(line).alignment(ratatui::layout::Alignment::Center);
    f.render_widget(
        para,
        Rect {
            x: area.x,
            y: area.y + 1,
            width: area.width,
            height: 1,
        },
    );
}

pub fn render(f: &mut ratatui::Frame, area: Rect, pipeline: &crate::application::pipeline::PipelineState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme::border()));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(1),
            Constraint::Length(5),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    let stage = pipeline.stage;
    let (ic_status, ic_color, ic_body) = match stage {
        PipelineStage::Idle => ("○ Pending", theme::text_dim(), "—"),
        PipelineStage::IntentClassification => ("◉ Classifying", theme::accent(), "Running classification..."),
        _ => ("✓ Complete", theme::success(), "Routed"),
    };

    render_node_with_body(
        f,
        chunks[0],
        "Intent Classifier",
        ic_status,
        ic_color,
        ic_body,
    );
    let conn1 = Paragraph::new(Line::from(Span::styled(" │", Style::new().fg(theme::border()))));
    f.render_widget(conn1, chunks[1]);

    let (clarifier_status, clarifier_color, clarifier_body) = match stage {
        PipelineStage::Idle | PipelineStage::IntentClassification => {
            ("○ Pending", theme::text_dim(), "—")
        }
        PipelineStage::Clarification => {
            ("◉ Clarifying", theme::accent(), "Awaiting clarification...")
        }
        _ => ("✓ Complete", theme::success(), "Plan approved"),
    };

    render_node_with_body(
        f,
        chunks[2],
        "Clarifier",
        clarifier_status,
        clarifier_color,
        clarifier_body,
    );
    let conn2 = Paragraph::new(Line::from(Span::styled(" │", Style::new().fg(theme::border()))));
    f.render_widget(conn2, chunks[3]);

    render_deep_researcher(f, chunks[4], pipeline);
}

fn render_node_with_body(
    f: &mut ratatui::Frame,
    area: Rect,
    title: &str,
    status: &str,
    color: ratatui::style::Color,
    body: &str,
) {
    let node_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme::border()));

    let inner = node_block.inner(area);
    f.render_widget(node_block, area);

    let title_line = Line::from(Span::styled(
        title,
        Style::new().fg(theme::text_main()).add_modifier(Modifier::BOLD),
    ));
    let status_line = Line::from(Span::styled(status, Style::new().fg(color)));
    let body_line = Line::from(Span::styled(body, Style::new().fg(color)));

    f.render_widget(
        title_line,
        Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: 1,
        },
    );
    f.render_widget(
        status_line,
        Rect {
            x: inner.x,
            y: inner.y + 1,
            width: inner.width,
            height: 1,
        },
    );
    f.render_widget(
        body_line,
        Rect {
            x: inner.x,
            y: inner.y + 2,
            width: inner.width,
            height: 1,
        },
    );
}

fn render_deep_researcher(f: &mut ratatui::Frame, area: Rect, pipeline: &crate::application::pipeline::PipelineState) {
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme::border()))
        .title(Span::styled(
            " Deep Researcher ",
            Style::new().fg(theme::accent()).add_modifier(Modifier::BOLD),
        ));

    let inner = outer_block.inner(area);
    f.render_widget(outer_block, area);

    let stage = pipeline.stage;
    let (orch_icon, orch_desc, orch_color) = match stage {
        PipelineStage::Idle | PipelineStage::IntentClassification | PipelineStage::Clarification => {
            ("○", "pending", theme::text_dim())
        }
        PipelineStage::ShallowResearch => {
            ("◉", "Shallow research in progress", theme::accent())
        }
        PipelineStage::DeepResearch => {
            ("✓", "Coordinated", theme::success())
        }
        PipelineStage::Complete => {
            ("✓", "Complete", theme::success())
        }
        PipelineStage::Cancelled => {
            ("✗", "Cancelled", theme::error())
        }
        PipelineStage::Failed => {
            ("✗", "Failed", theme::error())
        }
    };

    let (plan_icon, plan_desc, plan_color) = match stage {
        PipelineStage::DeepResearch => {
            if pipeline.current_step <= 2 {
                ("◉", "Planning", theme::accent())
            } else {
                ("✓", "Plan generated", theme::success())
            }
        }
        PipelineStage::Complete => {
            ("✓", "Plan generated", theme::success())
        }
        _ => ("○", "pending", theme::text_dim()),
    };

    let (r1_icon, r1_desc, r1_color) = match stage {
        PipelineStage::DeepResearch => {
            if pipeline.current_step == 3 {
                ("◉", "Researching", theme::accent())
            } else if pipeline.current_step > 3 {
                ("✓", "Complete", theme::success())
            } else {
                ("○", "pending", theme::text_dim())
            }
        }
        PipelineStage::Complete => {
            ("✓", "Complete", theme::success())
        }
        _ => ("○", "pending", theme::text_dim()),
    };

    let (r2_icon, r2_desc, r2_color) = match stage {
        PipelineStage::DeepResearch => {
            if pipeline.current_step == 4 {
                ("◉", "Researching", theme::accent())
            } else if pipeline.current_step > 4 {
                ("✓", "Complete", theme::success())
            } else {
                ("○", "pending", theme::text_dim())
            }
        }
        PipelineStage::Complete => {
            ("✓", "Complete", theme::success())
        }
        _ => ("○", "pending", theme::text_dim()),
    };

    let (ver_icon, ver_desc, ver_color) = match stage {
        PipelineStage::DeepResearch => {
            if pipeline.current_step >= 5 {
                ("◉", "Verifying citations", theme::accent())
            } else {
                ("○", "pending", theme::text_dim())
            }
        }
        PipelineStage::Complete => {
            ("✓", "Verified", theme::success())
        }
        _ => ("○", "pending", theme::text_dim()),
    };

    let (fin_icon, fin_desc, fin_color) = match stage {
        PipelineStage::Complete => {
            ("✓", "Report generated", theme::success())
        }
        _ => ("○", "pending", theme::text_dim()),
    };

    let subagents = [
        ("Orchestrator", orch_icon, orch_desc, orch_color),
        ("Planner", plan_icon, plan_desc, plan_color),
        ("Researcher [Round 1/2]", r1_icon, r1_desc, r1_color),
        ("Researcher [Round 2/2]", r2_icon, r2_desc, r2_color),
        ("Citation Verification", ver_icon, ver_desc, ver_color),
        ("Final Report", fin_icon, fin_desc, fin_color),
    ];

    for (i, (name, icon, detail, color)) in subagents.iter().enumerate() {
        let y = inner.y + i as u16;
        if y >= inner.y + inner.height {
            break;
        }

        let line = Line::from(vec![
            Span::styled("    ", Style::new().fg(theme::text_dim())),
            Span::styled(format!("{} ", icon), Style::new().fg(*color)),
            Span::styled(format!("{:<22}", name), Style::new().fg(theme::text_main())),
            Span::styled(*detail, Style::new().fg(theme::text_dim())),
        ]);
        f.render_widget(
            line,
            Rect {
                x: inner.x,
                y,
                width: inner.width,
                height: 1,
            },
        );
    }
}
