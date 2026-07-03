use crate::presentation::theme::{ACCENT, BORDER, TEXT_DIM};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(f: &mut ratatui::Frame, area: Rect, placeholder: &str) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" RESEARCH CONSOLE ")
        .border_style(Style::default().fg(BORDER));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let prompt_line = Line::from(vec![
        Span::styled(
            "> ",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::styled(placeholder, Style::default().fg(TEXT_DIM)),
    ]);

    let hint_new = Span::styled(
        "/new",
        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
    );
    let hint_enter = Span::styled(
        "Enter",
        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
    );
    let hint_text = Span::styled(" to start a new session | ", Style::default().fg(TEXT_DIM));
    let hint_text2 = Span::styled(" to submit", Style::default().fg(TEXT_DIM));
    let hint_line = Line::from(vec![
        Span::styled("Type ", Style::default().fg(TEXT_DIM)),
        hint_new,
        hint_text,
        hint_enter,
        hint_text2,
    ]);

    let paragraph = Paragraph::new(vec![prompt_line, hint_line]);
    f.render_widget(paragraph, inner);
}
