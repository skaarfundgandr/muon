use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::config::MuonConfig;
use crate::presentation::theme;

pub fn render(f: &mut ratatui::Frame, area: Rect, config: &MuonConfig) {
    let block = Block::default()
        .title(" DATA SOURCE REGISTRY ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::border()));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(inner);

    let sources = [
        ("Web Search", config.data_sources.web_search),
        ("Paper Search", config.data_sources.paper_search),
        ("Enterprise", config.data_sources.enterprise_systems),
        ("Knowledge Layer", config.data_sources.knowledge_layer_rag),
    ];

    for (i, (name, on)) in sources.iter().enumerate() {
        let dot = if *on { "●" } else { "○" };
        let color = if *on { theme::success() } else { theme::text_dim() };
        let line = Line::from(vec![
            Span::styled(*name, Style::default().fg(theme::text_main())),
            Span::raw(" "),
            Span::styled(dot, Style::default().fg(color)),
        ]);
        f.render_widget(Paragraph::new(line), chunks[i]);
    }
}
