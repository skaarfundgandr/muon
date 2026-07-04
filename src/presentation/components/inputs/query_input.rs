use crate::presentation::click::is_hovering;
use crate::presentation::theme::{ACCENT, BORDER, BORDER_FOCUS, BORDER_HOVER, TEXT_DIM, TEXT_MAIN};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

#[derive(Debug, Clone, Default)]
pub struct QueryInput {
    pub buffer: String,
    pub cursor: usize,
    pub active: bool,
    /// Last known mouse column for the query input region.
    pub mouse_col: u16,
    /// Last known mouse row for the query input region.
    pub mouse_row: u16,
}

impl QueryInput {
    pub fn insert_char(&mut self, c: char) {
        self.buffer.insert(self.cursor, c);
        self.cursor += 1;
    }

    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.buffer.remove(self.cursor);
        }
    }

    pub fn delete(&mut self) {
        if self.cursor < self.buffer.len() {
            self.buffer.remove(self.cursor);
        }
    }

    pub fn cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn cursor_right(&mut self) {
        if self.cursor < self.buffer.len() {
            self.cursor += 1;
        }
    }

    pub fn cursor_home(&mut self) {
        self.cursor = 0;
    }

    pub fn cursor_end(&mut self) {
        self.cursor = self.buffer.len();
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn is_new_command(&self) -> bool {
        self.buffer.starts_with("/new")
    }

    pub fn submit(&mut self) -> String {
        let query = if self.is_new_command() {
            self.buffer
                .trim_start_matches("/new")
                .trim()
                .to_string()
        } else {
            self.buffer.trim().to_string()
        };
        self.clear();
        query
    }
}

pub fn render(f: &mut ratatui::Frame, area: Rect, query: &QueryInput) {
    let hovered = is_hovering(area, query.mouse_col, query.mouse_row);
    let border_color = if query.active {
        BORDER_FOCUS
    } else if hovered {
        BORDER_HOVER
    } else {
        BORDER
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" RESEARCH CONSOLE ")
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let prompt_line = if query.active {
        let pre = &query.buffer[..query.cursor];
        let post = &query.buffer[query.cursor..];
        Line::from(vec![
            Span::styled("> ", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled(pre.to_string(), Style::default().fg(TEXT_MAIN)),
            Span::styled(
                "▎",
                Style::default()
                    .fg(ACCENT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(post.to_string(), Style::default().fg(TEXT_MAIN)),
        ])
    } else {
        Line::from(vec![
            Span::styled(
                "> ",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Type your research query or /new to start a fresh session...",
                Style::default().fg(TEXT_DIM),
            ),
        ])
    };

    let hint_new = Span::styled(
        "/new",
        Style::default()
            .fg(ACCENT)
            .add_modifier(Modifier::BOLD),
    );
    let hint_enter = Span::styled(
        "Enter",
        Style::default()
            .fg(ACCENT)
            .add_modifier(Modifier::BOLD),
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
