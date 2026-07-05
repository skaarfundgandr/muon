use ratatui::layout::Rect;

use crate::presentation::views::{SettingsTab, View};

#[derive(Debug, Clone)]
pub struct ClickTarget {
    pub rect: Rect,
    pub action: ClickAction,
}

#[derive(Debug, Clone)]
pub enum ClickAction {
    ActivateField(usize),
    ToggleCheckbox(usize),
    SwitchSettingsTab(SettingsTab),
    SwitchView(View),
    ActivateQueryInput,
    FocusField(usize),
    SelectSession(usize),
    SelectDropdownOption(usize),
    ActivateClarifier,
}

pub fn is_hovering(rect: Rect, col: u16, row: u16) -> bool {
    col >= rect.x && col < rect.x.saturating_add(rect.width) && row >= rect.y && row < rect.y.saturating_add(rect.height)
}
