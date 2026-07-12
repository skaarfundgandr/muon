use crate::presentation::click::is_hovering;
use crate::presentation::theme;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

/// Cells reserved after the caret when scrolling so typing stays visible.
const CARET_RIGHT_PAD: usize = 3;
/// Leading ellipsis when the view is scrolled past the start of the buffer.
const SCROLL_MARK: &str = "…";

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
        self.cursor += c.len_utf8();
    }

    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            let prev = prev_boundary(&self.buffer, self.cursor);
            self.buffer.replace_range(prev..self.cursor, "");
            self.cursor = prev;
        }
    }

    pub fn delete(&mut self) {
        if self.cursor < self.buffer.len() {
            let next = next_boundary(&self.buffer, self.cursor);
            self.buffer.replace_range(self.cursor..next, "");
        }
    }

    pub fn cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor = prev_boundary(&self.buffer, self.cursor);
        }
    }

    pub fn cursor_right(&mut self) {
        if self.cursor < self.buffer.len() {
            self.cursor = next_boundary(&self.buffer, self.cursor);
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

fn prev_boundary(s: &str, idx: usize) -> usize {
    let idx = idx.min(s.len());
    if idx == 0 {
        return 0;
    }
    let mut i = idx - 1;
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

fn next_boundary(s: &str, idx: usize) -> usize {
    let idx = idx.min(s.len());
    if idx >= s.len() {
        return s.len();
    }
    let mut i = idx + 1;
    while i < s.len() && !s.is_char_boundary(i) {
        i += 1;
    }
    i
}

/// Horizontal window over `buffer` so the caret stays visible with right pad.
/// Returns `(visible_pre, visible_post, scrolled_left)`.
pub fn visible_around_caret(buffer: &str, cursor: usize, text_budget: usize) -> (String, String, bool) {
    let cursor = if cursor > buffer.len() {
        buffer.len()
    } else if !buffer.is_char_boundary(cursor) {
        prev_boundary(buffer, cursor)
    } else {
        cursor
    };

    if text_budget == 0 {
        return (String::new(), String::new(), false);
    }

    let chars: Vec<char> = buffer.chars().collect();
    let total = chars.len();
    let caret = buffer[..cursor].chars().count();

    if total <= text_budget {
        let pre: String = chars[..caret].iter().collect();
        let post: String = chars[caret..].iter().collect();
        return (pre, post, false);
    }

    // Keep caret in view with CARET_RIGHT_PAD empty-ish room on the right when possible.
    let max_caret_offset = text_budget.saturating_sub(CARET_RIGHT_PAD).max(1);
    let mut start = caret.saturating_sub(max_caret_offset);
    let max_start = total.saturating_sub(text_budget);
    start = start.min(max_start);
    let end = (start + text_budget).min(total);

    let vis_pre_end = caret.clamp(start, end);
    let pre: String = chars[start..vis_pre_end].iter().collect();
    let post: String = chars[vis_pre_end..end].iter().collect();
    (pre, post, start > 0)
}

pub fn render(f: &mut ratatui::Frame, area: Rect, query: &QueryInput) {
    let hovered = is_hovering(area, query.mouse_col, query.mouse_row);
    let border_color = if query.active {
        theme::border_focus()
    } else if hovered {
        theme::border_hover()
    } else {
        theme::border()
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" RESEARCH CONSOLE ")
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let prompt_line = if query.active {
        // "> " (2) + optional scroll mark (1) + text + caret (1) must fit in inner.width
        let prompt_w = 2usize;
        let caret_w = 1usize;
        let usable = (inner.width as usize).saturating_sub(prompt_w);
        let mut text_budget = usable.saturating_sub(caret_w);

        let (mut pre, mut post, scrolled) =
            visible_around_caret(&query.buffer, query.cursor, text_budget);

        let mut spans = vec![Span::styled(
            "> ",
            Style::default()
                .fg(theme::accent())
                .add_modifier(Modifier::BOLD),
        )];

        if scrolled && text_budget > 0 {
            // Recompute with one cell reserved for the leading scroll mark.
            text_budget = text_budget.saturating_sub(1);
            let (p, po, _) = visible_around_caret(&query.buffer, query.cursor, text_budget);
            pre = p;
            post = po;
            spans.push(Span::styled(
                SCROLL_MARK,
                Style::default().fg(theme::text_dim()),
            ));
        }

        spans.push(Span::styled(
            pre,
            Style::default().fg(theme::text_main()),
        ));
        spans.push(Span::styled(
            "█",
            Style::default()
                .fg(theme::accent())
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            post,
            Style::default().fg(theme::text_main()),
        ));
        Line::from(spans)
    } else {
        Line::from(vec![
            Span::styled(
                "> ",
                Style::default().fg(theme::accent()).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Type your research query or /new to start a fresh session...",
                Style::default().fg(theme::text_dim()),
            ),
        ])
    };

    let hint_new = Span::styled(
        "/new",
        Style::default()
            .fg(theme::accent())
            .add_modifier(Modifier::BOLD),
    );
    let hint_enter = Span::styled(
        "Enter",
        Style::default()
            .fg(theme::accent())
            .add_modifier(Modifier::BOLD),
    );
    let hint_text = Span::styled(" to start a new session | ", Style::default().fg(theme::text_dim()));
    let hint_text2 = Span::styled(" to submit", Style::default().fg(theme::text_dim()));
    let hint_line = Line::from(vec![
        Span::styled("Type ", Style::default().fg(theme::text_dim())),
        hint_new,
        hint_text,
        hint_enter,
        hint_text2,
    ]);

    let paragraph = Paragraph::new(vec![prompt_line, hint_line]);
    f.render_widget(paragraph, inner);
}
