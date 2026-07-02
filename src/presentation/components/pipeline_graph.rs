use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::presentation::theme::{ACCENT, BORDER, SUCCESS, TEXT_DIM, TEXT_MAIN};

pub fn render_horizontal(
    f: &mut ratatui::Frame,
    area: Rect,
    clarifier: Option<fn(&mut ratatui::Frame, Rect)>,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(BORDER))
        .title(Span::styled(
            " PIPELINE ROUTING ",
            Style::new()
                .fg(TEXT_MAIN)
                .add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
        ])
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

    render_horizontal_node(f, nodes_row[0], "Intent Classifier", "✓ Complete", SUCCESS);
    render_horizontal_arrow(f, nodes_row[1], true);
    render_horizontal_node(f, nodes_row[2], "Research Pipeline", "◉ Active", ACCENT);
    render_horizontal_arrow(f, nodes_row[3], false);
    render_horizontal_node(f, nodes_row[4], "Deep Researcher", "○ Pending", TEXT_DIM);

    if let Some(clarifier_fn) = clarifier {
        clarifier_fn(f, chunks[1]);
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
        .border_style(Style::new().fg(BORDER));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let title_line = Line::from(Span::styled(
        title,
        Style::new().fg(TEXT_MAIN).add_modifier(Modifier::BOLD),
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
        Style::new().fg(ACCENT).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(BORDER)
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

pub fn render(f: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(BORDER));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    render_node(f, chunks[0], "Intent Classifier", "✓ done", SUCCESS);
    let conn1 = Paragraph::new(Line::from(Span::styled(" │", Style::new().fg(BORDER))));
    f.render_widget(conn1, chunks[1]);

    render_node(
        f,
        chunks[2],
        "Clarifier",
        "✓ 2 rounds | Plan approved",
        SUCCESS,
    );
    let conn2 = Paragraph::new(Line::from(Span::styled(" │", Style::new().fg(BORDER))));
    f.render_widget(conn2, chunks[3]);

    render_deep_researcher(f, chunks[4]);
}

fn render_node(
    f: &mut ratatui::Frame,
    area: Rect,
    title: &str,
    status: &str,
    color: ratatui::style::Color,
) {
    let node_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(BORDER));

    let inner = node_block.inner(area);
    f.render_widget(node_block, area);

    let title_line = Line::from(Span::styled(
        title,
        Style::new().fg(TEXT_MAIN).add_modifier(Modifier::BOLD),
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

fn render_deep_researcher(f: &mut ratatui::Frame, area: Rect) {
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(BORDER))
        .title(Span::styled(
            " Deep Researcher ",
            Style::new().fg(ACCENT).add_modifier(Modifier::BOLD),
        ));

    let inner = outer_block.inner(area);
    f.render_widget(outer_block, area);

    let status = Line::from(vec![
        Span::styled("  Orchestrator: ", Style::new().fg(TEXT_DIM)),
        Span::styled("Coordinating", Style::new().fg(ACCENT)),
    ]);
    f.render_widget(
        status,
        Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: 1,
        },
    );

    let subagents = [
        ("Planner", "✓", "5 queries, 4 sections", SUCCESS),
        ("Researcher [Round 1/2]", "◉", "23/47 sources", ACCENT),
        ("Researcher [Round 2/2]", "○", "pending", TEXT_DIM),
        ("Citation Verification", "○", "pending", TEXT_DIM),
        ("Final Report", "○", "pending", TEXT_DIM),
    ];

    for (i, (name, icon, detail, color)) in subagents.iter().enumerate() {
        let y = inner.y + 1 + i as u16;
        if y >= inner.y + inner.height {
            break;
        }

        let line = Line::from(vec![
            Span::styled("    ", Style::new().fg(TEXT_DIM)),
            Span::styled(format!("{} ", icon), Style::new().fg(*color)),
            Span::styled(format!("{:<22}", name), Style::new().fg(TEXT_MAIN)),
            Span::styled(*detail, Style::new().fg(TEXT_DIM)),
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
