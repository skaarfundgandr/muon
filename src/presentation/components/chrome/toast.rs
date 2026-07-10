use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::presentation::theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Error,
}

pub fn render(f: &mut ratatui::Frame, area: Rect, message: &str, kind: ToastKind) {
    if message.is_empty() || area.width < 12 || area.height < 3 {
        return;
    }

    let max_w = area.width.saturating_sub(2).clamp(20, 64);
    let text_w = (message.chars().count() as u16).saturating_add(4).min(max_w);
    let lines_est =
        ((message.chars().count() as u16) / text_w.saturating_sub(2).max(1) + 1).min(4);
    let height = lines_est
        .saturating_add(2)
        .min(area.height.saturating_sub(1))
        .max(3);

    let x = area
        .x
        .saturating_add(area.width.saturating_sub(text_w).saturating_sub(1));
    let y = area.y.saturating_add(1);
    let toast_area = Rect::new(x, y, text_w, height);

    f.render_widget(Clear, toast_area);

    let (border_fg, text_fg) = match kind {
        ToastKind::Success => (theme::success(), theme::success()),
        ToastKind::Error => (theme::error(), theme::error()),
        ToastKind::Info => (theme::accent(), theme::text_main()),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_fg))
        .style(
            Style::default()
                .bg(theme::bg_highlight())
                .fg(theme::text_main()),
        );

    let inner = block.inner(toast_area);
    f.render_widget(block, toast_area);
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            message.to_string(),
            Style::default()
                .fg(text_fg)
                .bg(theme::bg_highlight())
                .add_modifier(Modifier::BOLD),
        )))
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Left),
        inner,
    );
}
