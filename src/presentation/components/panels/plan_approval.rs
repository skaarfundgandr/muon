use crate::presentation::click::{ClickAction, ClickTarget, is_hovering};
use crate::presentation::theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

#[allow(clippy::too_many_arguments)]
pub fn render(
    f: &mut ratatui::Frame,
    area: Rect,
    plan: &crate::domain::agents::clarifier::ClarifierResult,
    focus: crate::presentation::PlanApprovalFocus,
    feedback_buffer: &str,
    feedback_cursor: usize,
    hit_registry: &mut Vec<ClickTarget>,
    mouse_col: u16,
    mouse_row: u16,
) {
    // 1. Dim the backdrop
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = f.buffer_mut().cell_mut((x, y)) {
                cell.fg = theme::text_dim();
                cell.bg = theme::bg_dark();
            }
        }
    }

    // 2. Center the modal (55% width, 65% height)
    let percent_w = 55u16;
    let percent_h = 65u16;
    let w = (area.width * percent_w / 100).max(50).min(area.width);
    let h = (area.height * percent_h / 100).max(16).min(area.height);

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(area.height.saturating_sub(h) / 2),
            Constraint::Length(h),
            Constraint::Min(0),
        ])
        .split(area);

    let popup_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(area.width.saturating_sub(w) / 2),
            Constraint::Length(w),
            Constraint::Min(0),
        ])
        .split(popup_layout[1])[1];

    // Clear the popup area
    f.render_widget(Clear, popup_area);

    // 3. Render the base block
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(theme::bg_main()))
        .border_style(Style::new().fg(theme::purple()))
        .title(Span::styled(
            " RESEARCH PLAN ",
            Style::new()
                .fg(theme::accent())
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // 4. Inner Layout
    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Plan title
            Constraint::Length(1), // Space
            Constraint::Min(3),    // Sections List
            Constraint::Length(1), // Space
            Constraint::Length(1), // Buttons
            Constraint::Length(1), // Space
            Constraint::Length(3), // Feedback text input
        ])
        .split(inner);

    // Title
    let title_text = plan
        .plan_title
        .as_deref()
        .unwrap_or("Proposed Research Plan");
    f.render_widget(
        Paragraph::new(Span::styled(
            title_text,
            Style::new()
                .fg(theme::accent())
                .add_modifier(Modifier::BOLD),
        ))
        .wrap(Wrap { trim: false }),
        inner_chunks[0],
    );

    // Sections List Block
    let sections_title = Line::from(Span::styled(
        "Sections:",
        Style::new().fg(theme::text_dim()),
    ));
    let mut section_lines = vec![sections_title, Line::default()];

    for (i, sec) in plan.plan_sections.iter().enumerate() {
        section_lines.push(Line::from(vec![
            Span::styled(format!("  {}. ", i + 1), Style::new().fg(theme::accent())),
            Span::styled(sec, Style::new().fg(theme::text_main())),
        ]));
    }

    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme::purple()));

    let list_inner = list_block.inner(inner_chunks[2]);
    f.render_widget(list_block, inner_chunks[2]);
    f.render_widget(
        Paragraph::new(section_lines).wrap(Wrap { trim: false }),
        list_inner,
    );

    // Buttons
    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(12), // [Approve]
            Constraint::Length(4),  // Space
            Constraint::Length(10), // [Reject]
            Constraint::Length(4),  // Space
            Constraint::Length(17), // [Send Feedback]
            Constraint::Min(0),
        ])
        .split(inner_chunks[4]);

    // Approve Button
    let approve_focused = focus == crate::presentation::PlanApprovalFocus::Approve;
    let approve_hovered = is_hovering(button_chunks[0], mouse_col, mouse_row);
    let approve_style = if approve_focused {
        Style::new()
            .fg(theme::border_focus())
            .add_modifier(Modifier::BOLD)
    } else if approve_hovered {
        Style::new()
            .fg(theme::accent())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(theme::text_dim())
    };
    f.render_widget(
        Paragraph::new(Span::styled("[Approve]", approve_style)),
        button_chunks[0],
    );
    hit_registry.push(ClickTarget {
        rect: button_chunks[0],
        action: ClickAction::PlanApprove,
    });

    // Reject Button
    let reject_focused = focus == crate::presentation::PlanApprovalFocus::Reject;
    let reject_hovered = is_hovering(button_chunks[2], mouse_col, mouse_row);
    let reject_style = if reject_focused {
        Style::new()
            .fg(theme::border_focus())
            .add_modifier(Modifier::BOLD)
    } else if reject_hovered {
        Style::new()
            .fg(theme::accent())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(theme::text_dim())
    };
    f.render_widget(
        Paragraph::new(Span::styled("[Reject]", reject_style)),
        button_chunks[2],
    );
    hit_registry.push(ClickTarget {
        rect: button_chunks[2],
        action: ClickAction::PlanReject,
    });

    // Send Feedback Button
    let feedback_focused = focus == crate::presentation::PlanApprovalFocus::Feedback;
    let feedback_hovered = is_hovering(button_chunks[4], mouse_col, mouse_row);
    let feedback_style = if feedback_focused {
        Style::new()
            .fg(theme::border_focus())
            .add_modifier(Modifier::BOLD)
    } else if feedback_hovered {
        Style::new()
            .fg(theme::accent())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(theme::text_dim())
    };
    f.render_widget(
        Paragraph::new(Span::styled("[Send Feedback]", feedback_style)),
        button_chunks[4],
    );
    hit_registry.push(ClickTarget {
        rect: button_chunks[4],
        action: ClickAction::PlanFeedback,
    });

    // Feedback Text Input Area
    let feedback_area = inner_chunks[6];
    let input_hovered = is_hovering(feedback_area, mouse_col, mouse_row);
    let input_focused = focus == crate::presentation::PlanApprovalFocus::Feedback;
    let border_color = if input_focused {
        theme::border_focus()
    } else if input_hovered {
        theme::border_hover()
    } else {
        theme::border()
    };

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(border_color))
        .title(Span::styled(" Feedback ", Style::new().fg(theme::accent())));

    let inner_input = input_block.inner(feedback_area);
    f.render_widget(input_block, feedback_area);

    let displayed_text = feedback_buffer;
    let line = if input_focused {
        let (before, after) = displayed_text.split_at(feedback_cursor.min(displayed_text.len()));
        Line::from(vec![
            Span::styled(before.to_string(), Style::new().fg(theme::text_main())),
            Span::styled(
                "\u{2588}",
                Style::new()
                    .fg(theme::accent())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(after.to_string(), Style::new().fg(theme::text_main())),
        ])
    } else {
        Line::from(vec![Span::styled(
            displayed_text,
            Style::new().fg(theme::text_main()),
        )])
    };
    f.render_widget(Paragraph::new(line).wrap(Wrap { trim: false }), inner_input);

    hit_registry.push(ClickTarget {
        rect: feedback_area,
        action: ClickAction::PlanSelectFeedbackInput,
    });
}
