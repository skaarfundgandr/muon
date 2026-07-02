use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use crate::presentation::theme::{BORDER, PURPLE, SUCCESS, TEXT_DIM, TEXT_MAIN, WARNING};

pub fn render(f: &mut ratatui::Frame<'_>, area: Rect) {
    let items = [
        ("IEA Germany 2024 Energy Policy Review", 96u16, true),
        ("Japan Green Growth Strategy & Industrial Price Premiums", 89, true),
        ("Comparative Macroeconomic Modeling: FIT vs PPA", 78, true),
        ("Grid Storage Infrastructure Analysis 2024", 71, false),
    ];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    for (i, (title, pct, verified)) in items.iter().enumerate() {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER));

        let index_style = Style::default().fg(PURPLE).add_modifier(Modifier::BOLD);
        let index_span = Span::styled(format!("[{}] ", i + 1), index_style);
        let title_span = Span::styled(*title, Style::default().fg(TEXT_MAIN));

        let filled_count = (*pct / 20) as usize;
        let empty_count = 5usize.saturating_sub(filled_count);
        let filled: String = "█".repeat(filled_count);
        let empty: String = "░".repeat(empty_count);

        let pct_color = if *pct >= 85 {
            SUCCESS
        } else if *pct >= 60 {
            WARNING
        } else {
            TEXT_DIM
        };

        let bar_filled_style = if *pct >= 85 {
            SUCCESS
        } else if *pct >= 60 {
            WARNING
        } else {
            TEXT_DIM
        };

        let badge = if *verified {
            Span::styled(" ✓", Style::default().fg(SUCCESS))
        } else {
            Span::styled(" ✗", Style::default().fg(WARNING))
        };

        let line2 = Line::from(vec![
            Span::styled("Relevance: ", Style::default().fg(TEXT_DIM)),
            Span::styled(format!("{}%", *pct), Style::default().fg(pct_color)),
            Span::styled(" ", Style::default().fg(TEXT_DIM)),
            Span::styled(filled, Style::default().fg(bar_filled_style)),
            Span::styled(empty, Style::default().fg(TEXT_DIM)),
            badge,
        ]);

        let paragraph = Paragraph::new(vec![
            Line::from(vec![index_span, title_span]),
            line2,
        ]);

        let inner = block.inner(chunks[i]);
        f.render_widget(block, chunks[i]);
        f.render_widget(paragraph, inner);
    }
}
