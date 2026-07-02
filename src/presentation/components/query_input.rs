use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use crate::presentation::theme::{ACCENT, BORDER, TEXT_DIM};

pub fn render(f: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" RESEARCH CONSOLE ")
        .border_style(Style::default().fg(BORDER));

    let prompt = Span::styled("> ", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD));
    let placeholder = Span::styled(
        "Economic impacts of renewable energy transition in Germany and Japan",
        Style::default().fg(TEXT_DIM),
    );
    let line = Line::from(vec![prompt, placeholder]);
    let paragraph = Paragraph::new(line);

    let inner = block.inner(area);
    f.render_widget(block, area);
    f.render_widget(paragraph, inner);
}
