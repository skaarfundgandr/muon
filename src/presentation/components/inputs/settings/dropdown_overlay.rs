use crate::presentation::click::{ClickAction, ClickTarget, is_hovering};
use crate::presentation::form::{FieldDef, FormState};
use crate::presentation::theme;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem};

/// Render a dropdown options popup overlay below the given row rect.
///
/// When `form.dropdown_open` is true and the focused field is a Dropdown,
/// settings components render all their rows, then call this with the rect of
/// the focused row to draw the options list as an overlay below the field.
pub fn render_dropdown_overlay(
    f: &mut ratatui::Frame,
    below: Rect,
    fields: &[FieldDef],
    form: &FormState,
    hit_registry: &mut Vec<ClickTarget>,
    mouse_col: u16,
    mouse_row: u16,
) {
    let Some(field) = fields.get(form.focus) else {
        return;
    };
    if field.kind != crate::presentation::form::FieldKind::Dropdown {
        return;
    }

    let options = field.options;
    if options.is_empty() {
        return;
    }

    let list_h = options.len() as u16 + 2; // borders
    let popup_h = list_h.min(6);
    let popup_y = below.y.saturating_add(below.height);
    let term_h = f.area().height;
    let popup_y = if popup_y.saturating_add(popup_h) > term_h {
        // Not enough space below — try above the field
        let above = below.y.saturating_sub(popup_h);
        // If above would go past terminal top, clamp to 0
        // and render from top (popup may still be clipped but we tried)
        if above + popup_h > term_h {
            0u16
        } else {
            above
        }
    } else {
        popup_y
    };
    let popup_w = below.width.max(20);
    let popup_x = below.x;
    let popup_area = Rect::new(popup_x, popup_y, popup_w, popup_h);

    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(theme::bg_main()))
        .border_style(Style::new().fg(theme::border_focus()))
        .title(Span::styled(
            format!(" {} ", field.label),
            Style::new().fg(theme::accent()).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let mut items: Vec<ListItem> = Vec::with_capacity(options.len());
    let option_rects: Vec<Rect> = options.iter().enumerate().map(|(i, _)| {
        Rect::new(inner.x, inner.y.saturating_add(i as u16), inner.width, 1)
    }).collect();
    for (i, opt) in options.iter().enumerate() {
        let selected = i == form.dropdown_cursor;
        let hovered = is_hovering(option_rects[i], mouse_col, mouse_row) && !selected;
        let (style, arrow) = if selected {
            (Style::new().bg(theme::bg_highlight()).fg(theme::text_main()).add_modifier(Modifier::BOLD), "\u{25B6} ")
        } else if hovered {
            (Style::new().bg(theme::bg_dark()).fg(theme::text_main()), "  ")
        } else {
            (Style::new().fg(theme::text_dim()), "  ")
        };
        items.push(ListItem::new(Line::from(vec![
            Span::styled(arrow, Style::new().fg(theme::accent())),
            Span::styled(*opt, style),
        ])));
    }
    let list = List::new(items);
    f.render_widget(list, inner);

    for (i, _) in options.iter().enumerate() {
        hit_registry.push(ClickTarget {
            rect: option_rects[i],
            action: ClickAction::SelectDropdownOption(i),
        });
    }
}
