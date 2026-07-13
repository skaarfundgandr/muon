use crossterm::event::{KeyCode, KeyEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    Text,
    Number,
    Dropdown,
    Checkbox,
    Button,
}

/// Static definition of a field — label, kind, and dropdown options.
#[derive(Debug, Clone)]
pub struct FieldDef {
    pub label: &'static str,
    pub kind: FieldKind,
    pub options: &'static [&'static str],
}

impl FieldDef {
    pub const fn text(label: &'static str) -> Self {
        Self {
            label,
            kind: FieldKind::Text,
            options: &[],
        }
    }
    pub const fn number(label: &'static str) -> Self {
        Self {
            label,
            kind: FieldKind::Number,
            options: &[],
        }
    }
    pub const fn dropdown(label: &'static str, options: &'static [&'static str]) -> Self {
        Self {
            label,
            kind: FieldKind::Dropdown,
            options,
        }
    }
    pub const fn checkbox(label: &'static str) -> Self {
        Self {
            label,
            kind: FieldKind::Checkbox,
            options: &[],
        }
    }
    pub const fn button(label: &'static str) -> Self {
        Self {
            label,
            kind: FieldKind::Button,
            options: &[],
        }
    }
}

/// Per-tab form state — tracks which field is focused and edit mode.
#[derive(Debug, Clone, Default)]
pub struct FormState {
    /// Index of the focused field within the current tab's field list.
    pub focus: usize,
    /// When Some, a text field is being edited — this is the working buffer.
    pub edit_buffer: Option<String>,
    /// Cursor position within edit_buffer.
    pub edit_cursor: usize,
    /// Whether a dropdown is currently open for selection.
    pub dropdown_open: bool,
    /// Selected index within an open dropdown.
    pub dropdown_cursor: usize,
    /// True if there are unsaved changes (shows "UNSAVED CHANGES" in header).
    pub dirty: bool,
    /// Last known mouse column for this form's region (for hover styling).
    pub mouse_col: u16,
    /// Last known mouse row for this form's region (for hover styling).
    pub mouse_row: u16,
    /// Scroll offset for list-style tabs (Providers, Tools). Index of the first
    /// visible row. Reset to 0 when the tab is reset via `reset_edit`.
    pub scroll_offset: usize,
}

impl FormState {
    /// Reset focus and edit state — call when switching tabs.
    pub fn reset_edit(&mut self) {
        self.edit_buffer = None;
        self.edit_cursor = 0;
        self.dropdown_open = false;
        self.dropdown_cursor = 0;
        self.scroll_offset = 0;
    }

    /// Advance focus to the next field in the list.
    pub fn focus_next(&mut self, len: usize) {
        if len > 0 {
            self.focus = (self.focus + 1) % len;
            self.reset_edit();
        }
    }

    /// Move focus to the previous field in the list.
    pub fn focus_prev(&mut self, len: usize) {
        if len > 0 {
            self.focus = (self.focus + len - 1) % len;
            self.reset_edit();
        }
    }

    /// Begin editing a text/number field — seeds buffer from current value.
    pub fn begin_edit(&mut self, current_value: &str) {
        self.edit_buffer = Some(current_value.to_string());
        self.edit_cursor = current_value.len();
    }

    /// Confirm edit — returns the edited string and clears edit state.
    pub fn confirm_edit(&mut self) -> Option<String> {
        let buf = self.edit_buffer.take();
        self.edit_cursor = 0;
        self.dirty = true;
        buf
    }

    /// Cancel edit — discards buffer.
    pub fn cancel_edit(&mut self) {
        self.edit_buffer = None;
        self.edit_cursor = 0;
    }

    /// Open dropdown for the current field.
    pub fn open_dropdown(&mut self) {
        self.dropdown_open = true;
        self.dropdown_cursor = 0;
    }

    /// Returns true if currently in text edit mode.
    pub fn is_editing(&self) -> bool {
        self.edit_buffer.is_some()
    }

    /// Handle a keystroke while in text edit mode.
    /// Returns true if the key was consumed.
    pub fn handle_edit_key(&mut self, key: KeyEvent) -> bool {
        let Some(buf) = self.edit_buffer.as_mut() else {
            return false;
        };
        match key.code {
            KeyCode::Char(c) => {
                buf.insert(self.edit_cursor, c);
                self.edit_cursor += 1;
                true
            }
            KeyCode::Backspace => {
                if self.edit_cursor > 0 {
                    self.edit_cursor -= 1;
                    buf.remove(self.edit_cursor);
                }
                true
            }
            KeyCode::Delete => {
                if self.edit_cursor < buf.len() {
                    buf.remove(self.edit_cursor);
                }
                true
            }
            KeyCode::Left => {
                if self.edit_cursor > 0 {
                    self.edit_cursor -= 1;
                }
                true
            }
            KeyCode::Right => {
                if self.edit_cursor < buf.len() {
                    self.edit_cursor += 1;
                }
                true
            }
            KeyCode::Home => {
                self.edit_cursor = 0;
                true
            }
            KeyCode::End => {
                self.edit_cursor = buf.len();
                true
            }
            _ => false,
        }
    }
}
