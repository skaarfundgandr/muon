use crate::presentation::theme;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(f: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(theme::border()));

    let context_pct: u16 = 62;
    let filled_count = (context_pct / 10) as usize;
    let empty_count = 10 - filled_count;

    let bar_color = if context_pct >= 80 {
        theme::error()
    } else if context_pct >= 60 {
        theme::warning()
    } else {
        theme::accent()
    };

    let filled_bar: String = "█".repeat(filled_count);
    let empty_bar: String = "░".repeat(empty_count);

    let line = Line::from(vec![
        Span::styled("Context: ", Style::default().fg(theme::text_dim())),
        Span::styled(filled_bar, Style::default().fg(bar_color)),
        Span::styled(empty_bar, Style::default().fg(theme::text_dim())),
        Span::styled(
            format!(" {}%", context_pct),
            Style::default().fg(bar_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  |  ", Style::default().fg(theme::text_dim())),
        Span::styled("Tokens: ", Style::default().fg(theme::text_dim())),
        Span::styled("↓8,421", Style::default().fg(theme::success())),
        Span::styled(" ", Style::default().fg(theme::text_main())),
        Span::styled("↑4,426", Style::default().fg(theme::accent())),
        Span::styled("  |  ", Style::default().fg(theme::text_dim())),
        Span::styled("Cost: ", Style::default().fg(theme::text_dim())),
        Span::styled(
            "$0.034",
            Style::default().fg(theme::success()).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  |  ", Style::default().fg(theme::text_dim())),
        Span::styled("Mem: ", Style::default().fg(theme::text_dim())),
        Span::styled("142MB", Style::default().fg(theme::text_main())),
    ]);

    let paragraph = Paragraph::new(line);
    f.render_widget(block.clone(), area);
    f.render_widget(paragraph, block.inner(area));
}
