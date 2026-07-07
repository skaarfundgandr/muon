use crate::config::{MuonConfig, ProviderConfig};
use crate::presentation::views::View;
use crate::presentation::click::{is_hovering, ClickAction, ClickTarget};
use crate::presentation::form::{FieldDef, FormState};
use crate::presentation::theme;
use crate::presentation::components::inputs::settings::dropdown_overlay::PendingDropdown;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Clear};

pub const PROVIDER_TYPES: &[&str] = &["openai", "gemini", "anthropic", "openai_compatible"];

pub fn fields(config: &MuonConfig) -> Vec<FieldDef> {
    let mut f = Vec::new();
    for _ in &config.providers {
        f.push(FieldDef::dropdown("Type", PROVIDER_TYPES));
        f.push(FieldDef::text("Name"));
        f.push(FieldDef::text("Base URL"));
        f.push(FieldDef::text("API Key"));
        f.push(FieldDef::button("Models"));
    }
    f.push(FieldDef::button("+ Add Provider"));
    f
}

pub fn get_field(config: &MuonConfig, index: usize) -> String {
    let n = config.providers.len();
    if index < 5 * n {
        let provider_idx = index / 5;
        let sub_idx = index % 5;
        let provider = &config.providers[provider_idx];
        match sub_idx {
            0 => match provider.provider_type {
                crate::config::ProviderType::OpenAI => "openai".to_string(),
                crate::config::ProviderType::Gemini => "gemini".to_string(),
                crate::config::ProviderType::Anthropic => "anthropic".to_string(),
                crate::config::ProviderType::OpenAICompatible => "openai_compatible".to_string(),
            },
            1 => provider.name.clone(),
            2 => provider.base_url.clone(),
            3 => provider.api_key.clone(),
            _ => String::new(),
        }
    } else {
        String::new()
    }
}

pub fn set_field(config: &mut MuonConfig, index: usize, value: &str) {
    let n = config.providers.len();
    if index < 5 * n {
        let provider_idx = index / 5;
        let sub_idx = index % 5;
        let provider = &mut config.providers[provider_idx];
        match sub_idx {
            0 => {
                provider.provider_type = match value {
                    "openai" => crate::config::ProviderType::OpenAI,
                    "gemini" => crate::config::ProviderType::Gemini,
                    "anthropic" => crate::config::ProviderType::Anthropic,
                    _ => crate::config::ProviderType::OpenAICompatible,
                };
            }
            1 => provider.name = value.to_string(),
            2 => provider.base_url = value.to_string(),
            3 => provider.api_key = value.to_string(),
            _ => {}
        }
    }
}

pub fn toggle_field(_config: &mut MuonConfig, _index: usize) {}

fn section_block(title: &str, focused: bool, hovered: bool) -> Block<'_> {
    let border_color = if focused {
        theme::border_focus()
    } else if hovered {
        theme::border_hover()
    } else {
        theme::border()
    };
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(border_color))
        .title(Span::styled(
            format!(" {} ", title),
            Style::new()
                .fg(theme::purple())
                .add_modifier(Modifier::BOLD),
        ))
}

fn mask_key(key: &str) -> String {
    let n = key.chars().count().min(22);
    "\u{25C9}".repeat(n)
}

#[allow(clippy::too_many_arguments)]
fn field_line<'a>(
    label: &'a str,
    value: &'a str,
    focused: bool,
    editing: bool,
    cursor: usize,
    buffer: Option<&'a str>,
    hovered: bool,
    value_color: ratatui::style::Color,
) -> Line<'a> {
    let prefix = if focused { "> " } else { "  " };
    let label_style = if focused {
        Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(theme::text_dim())
    };
    let border_style = if focused {
        Style::new().fg(theme::border_focus())
    } else {
        Style::new().fg(theme::text_dim())
    };
    
    if editing {
        let buf = buffer.unwrap_or("");
        let cur = cursor.min(buf.len());
        let pre = &buf[..cur];
        let post = &buf[cur..];
        Line::from(vec![
            Span::styled(prefix, Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:<10}", label), label_style),
            Span::styled("[", border_style),
            Span::styled(pre.to_string(), Style::new().fg(theme::accent()).add_modifier(Modifier::BOLD)),
            Span::styled("\u{258E}", Style::new().fg(theme::border_focus())),
            Span::styled(post.to_string(), Style::new().fg(theme::accent()).add_modifier(Modifier::BOLD)),
            Span::styled("]", border_style),
        ])
    } else {
        let display_val = if value.is_empty() { "<click to edit>" } else { value };
        let fg_color = if focused { theme::border_focus() } else if hovered { theme::text_main() } else { value_color };
        Line::from(vec![
            Span::styled(prefix, Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:<10}", label), label_style),
            Span::styled("[", border_style),
            Span::styled(display_val, Style::new().fg(fg_color)),
            Span::styled("]", border_style),
        ])
    }
}

const ROW_HEIGHT: u16 = 7;
const ADD_BUTTON_HEIGHT: u16 = 3;
const COUNT_SUMMARY_HEIGHT: u16 = 1;
const SCROLL_INDICATOR_HEIGHT: u16 = 1;

fn dropdown_line<'a>(
    label: &'a str,
    value: &'a str,
    focused: bool,
    hovered: bool,
) -> Line<'a> {
    let prefix = if focused { "> " } else { "  " };
    let label_style = if focused {
        Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(theme::text_dim())
    };
    let border_style = if focused {
        Style::new().fg(theme::border_focus())
    } else {
        Style::new().fg(theme::text_dim())
    };
    let val_style = if focused {
        Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)
    } else if hovered {
        Style::new().fg(theme::text_main())
    } else {
        Style::new().fg(theme::accent())
    };
    Line::from(vec![
        Span::styled(prefix, Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{:<10}", label), label_style),
        Span::styled("[", border_style),
        Span::styled(value, val_style),
        Span::styled("\u{25BC}", val_style),
        Span::styled("]", border_style),
    ])
}

#[allow(clippy::too_many_arguments)]
fn render_provider_row(
    f: &mut ratatui::Frame,
    area: Rect,
    idx: usize,
    provider: &ProviderConfig,
    focused_sub_idx: Option<usize>,
    form: &FormState,
    hovered: bool,
    hit_registry: &mut Vec<ClickTarget>,
    mouse_col: u16,
    mouse_row: u16,
    pending_dropdown: &mut Option<PendingDropdown>,
) {
    let title = format!(
        "#{}  {}",
        idx + 1,
        if provider.name.is_empty() { "<unnamed>" } else { provider.name.as_str() }
    );
    let block = section_block(&title, focused_sub_idx.is_some(), hovered && focused_sub_idx.is_none());
    let inner = block.inner(area);
    f.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    // Click targets for rows
    for (sub, r) in rows.iter().enumerate() {
        hit_registry.push(ClickTarget {
            rect: *r,
            action: ClickAction::ActivateField(5 * idx + sub),
        });
    }

    // Type (0)
    let type_focused = focused_sub_idx == Some(0);
    let type_hover = is_hovering(rows[0], mouse_col, mouse_row);
    let type_str = match provider.provider_type {
        crate::config::ProviderType::OpenAI => "openai",
        crate::config::ProviderType::Gemini => "gemini",
        crate::config::ProviderType::Anthropic => "anthropic",
        crate::config::ProviderType::OpenAICompatible => "openai_compatible",
    };
    f.render_widget(
        Paragraph::new(dropdown_line("Type", type_str, type_focused, type_hover)),
        rows[0],
    );

    // Name (1)
    let name_focused = focused_sub_idx == Some(1);
    let name_editing = name_focused && form.is_editing();
    let name_hover = is_hovering(rows[1], mouse_col, mouse_row);
    f.render_widget(
        Paragraph::new(field_line(
            "Name",
            &provider.name,
            name_focused,
            name_editing,
            form.edit_cursor,
            form.edit_buffer.as_deref(),
            name_hover,
            theme::success(),
        )),
        rows[1],
    );

    // Base URL (2)
    let url_focused = focused_sub_idx == Some(2);
    let url_editing = url_focused && form.is_editing();
    let url_hover = is_hovering(rows[2], mouse_col, mouse_row);
    f.render_widget(
        Paragraph::new(field_line(
            "Base URL",
            &provider.base_url,
            url_focused,
            url_editing,
            form.edit_cursor,
            form.edit_buffer.as_deref(),
            url_hover,
            theme::cyan(),
        )),
        rows[2],
    );

    // API Key (3)
    let key_focused = focused_sub_idx == Some(3);
    let key_editing = key_focused && form.is_editing();
    let key_hover = is_hovering(rows[3], mouse_col, mouse_row);
    let key_val = if key_editing {
        form.edit_buffer.clone().unwrap_or_default()
    } else if provider.api_key.is_empty() {
        "".to_string()
    } else {
        mask_key(&provider.api_key)
    };
    f.render_widget(
        Paragraph::new(field_line(
            "API Key",
            &key_val,
            key_focused,
            key_editing,
            form.edit_cursor,
            form.edit_buffer.as_deref(),
            key_hover,
            theme::success(),
        )),
        rows[3],
    );

    // Models (4)
    let models_focused = focused_sub_idx == Some(4);
    let model_count = provider.models.len();
    let model_summary = if model_count == 0 {
        "(no models)".to_string()
    } else if model_count == 1 {
        "1 model".to_string()
    } else {
        format!("{model_count} models")
    };

    let edit_btn_x = rows[4].x + 28;
    let edit_btn_rect = Rect::new(edit_btn_x, rows[4].y, 13, 1);
    let edit_hover = is_hovering(edit_btn_rect, mouse_col, mouse_row);

    let fetch_btn_x = edit_btn_x + 13 + 2;
    let fetch_btn_rect = Rect::new(fetch_btn_x, rows[4].y, 14, 1);
    let fetch_hover = is_hovering(fetch_btn_rect, mouse_col, mouse_row);

    let edit_style = if edit_hover {
        Style::new().fg(theme::accent()).add_modifier(Modifier::BOLD)
    } else if models_focused {
        Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(theme::accent())
    };

    let fetch_style = if fetch_hover {
        Style::new().fg(theme::accent()).add_modifier(Modifier::BOLD)
    } else if models_focused {
        Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(theme::accent())
    };

    let btn_prefix = if models_focused { "> " } else { "  " };
    let models_line = Line::from(vec![
        Span::styled(btn_prefix, Style::new().fg(theme::border_focus())),
        Span::styled(format!("{:<10}  {:<12}  ", "Models", model_summary), Style::new().fg(theme::text_dim())),
        Span::styled("[Edit Models]", edit_style),
        Span::styled("  ", Style::new().fg(theme::text_dim())),
        Span::styled("[Fetch Models]", fetch_style),
    ]);
    f.render_widget(Paragraph::new(models_line), rows[4]);

    hit_registry.push(ClickTarget {
        rect: edit_btn_rect,
        action: ClickAction::EditProviderModels(idx),
    });

    hit_registry.push(ClickTarget {
        rect: fetch_btn_rect,
        action: ClickAction::FetchProviderModels(idx),
    });

    // Set Pending dropdown overlay if open
    if form.dropdown_open && focused_sub_idx == Some(0) {
        let options: Vec<String> = PROVIDER_TYPES.iter().map(|s| s.to_string()).collect();
        *pending_dropdown = Some(PendingDropdown {
            below: rows[0],
            field_label: "Type".to_string(),
            options,
        });
    }
}

fn render_add_button(
    f: &mut ratatui::Frame,
    area: Rect,
    focused: bool,
    hit_registry: &mut Vec<ClickTarget>,
) {
    let border_color = if focused {
        theme::border_focus()
    } else {
        theme::border_hover()
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(border_color))
        .title(Span::styled(
            " ADD ",
            Style::new().fg(theme::success()).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let prefix = if focused { "> " } else { "  " };
    let line = Line::from(vec![
        Span::styled(prefix, Style::new().fg(theme::border_focus())),
        Span::styled(
            "[+ Add Provider]",
            Style::new().fg(theme::success()).add_modifier(Modifier::BOLD),
        )
    ]);
    f.render_widget(Paragraph::new(line), inner);

    hit_registry.push(ClickTarget {
        rect: area,
        action: ClickAction::AddProvider,
    });
}

fn render_empty_state(f: &mut ratatui::Frame, area: Rect) {
    let block = section_block("LLM PROVIDERS", false, is_hovering(area, 0, 0));
    let inner = block.inner(area);
    f.render_widget(block, area);
    let line = Line::from(vec![Span::styled(
        "No providers configured. Add one below or set [providers] in your config.toml.",
        Style::new().fg(theme::text_dim()),
    )]);
    f.render_widget(Paragraph::new(line), inner);
}

pub fn render(
    f: &mut ratatui::Frame,
    area: Rect,
    config: &MuonConfig,
    form: &FormState,
    hit_registry: &mut Vec<ClickTarget>,
    mouse_col: u16,
    mouse_row: u16,
    pending_dropdown: &mut Option<PendingDropdown>,
) {
    let n = config.providers.len();
    let list_area_height = area.height.saturating_sub(ADD_BUTTON_HEIGHT);
    let available = list_area_height.saturating_sub(COUNT_SUMMARY_HEIGHT);
    let scroll_offset = form.scroll_offset;
    let max_no_indicators = (available / ROW_HEIGHT) as usize;
    let (max_visible_rows, show_top, show_bottom) = if n <= max_no_indicators {
        (n, false, false)
    } else {
        let worst = ((available.saturating_sub(SCROLL_INDICATOR_HEIGHT * 2)) / ROW_HEIGHT)
            as usize;
        let worst = worst.max(1);
        if scroll_offset + worst < n {
            (worst, scroll_offset > 0, true)
        } else {
            let refined =
                ((available.saturating_sub(SCROLL_INDICATOR_HEIGHT)) / ROW_HEIGHT) as usize;
            (refined.max(1), true, false)
        }
    };
    let max_offset = n.saturating_sub(max_visible_rows);
    let scroll_offset = scroll_offset.min(max_offset);
    let scroll_end = (scroll_offset + max_visible_rows).min(n);

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(ADD_BUTTON_HEIGHT),
        ])
        .split(area);

    if n == 0 {
        render_empty_state(f, outer[0]);
    } else {
        let mut row_constraints: Vec<Constraint> = Vec::new();
        if show_top {
            row_constraints.push(Constraint::Length(SCROLL_INDICATOR_HEIGHT));
        }
        for _ in scroll_offset..scroll_end {
            row_constraints.push(Constraint::Length(ROW_HEIGHT));
        }
        if show_bottom {
            row_constraints.push(Constraint::Length(SCROLL_INDICATOR_HEIGHT));
        }
        row_constraints.push(Constraint::Length(COUNT_SUMMARY_HEIGHT));
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(row_constraints)
            .split(outer[0]);

        let mut idx = 0;
        if show_top {
            let line = Line::from(Span::styled(
                "\u{2191} more above",
                Style::new().fg(theme::text_dim()),
            ));
            f.render_widget(Paragraph::new(line), rows[idx]);
            idx += 1;
        }

        let focused_row = form.focus < 5 * n;
        for (i, p) in config
            .providers
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(max_visible_rows)
        {
            let hovered = is_hovering(rows[idx], mouse_col, mouse_row);
            let focused_sub_idx = if focused_row && form.focus / 5 == i {
                Some(form.focus % 5)
            } else {
                None
            };
            render_provider_row(
                f,
                rows[idx],
                i,
                p,
                focused_sub_idx,
                form,
                hovered,
                hit_registry,
                mouse_col,
                mouse_row,
                pending_dropdown,
            );
            idx += 1;
        }

        if show_bottom {
            let line = Line::from(Span::styled(
                "\u{2193} more below",
                Style::new().fg(theme::text_dim()),
            ));
            f.render_widget(Paragraph::new(line), rows[idx]);
            idx += 1;
        }

        let mut spans: Vec<Span> = Vec::new();
        spans.push(Span::styled(
            format!("{n} provider(s) configured"),
            Style::new().fg(theme::text_dim()),
        ));
        f.render_widget(Paragraph::new(Line::from(spans)), rows[idx]);
    }

    let add_focused = form.focus == 5 * n;
    render_add_button(f, outer[1], add_focused, hit_registry);
}

fn absolute_centered_rect(w: u16, h: u16, r: Rect) -> Rect {
    let x = r.x + r.width.saturating_sub(w) / 2;
    let y = r.y + r.height.saturating_sub(h) / 2;
    Rect::new(x, y, w.min(r.width), h.min(r.height))
}

#[allow(clippy::too_many_arguments)]
pub fn render_models_popup(
    f: &mut ratatui::Frame,
    area: Rect,
    config: &MuonConfig,
    provider_idx: usize,
    focus_idx: usize,
    edit_buffer: Option<&str>,
    edit_cursor: usize,
    hit_registry: &mut Vec<ClickTarget>,
    mouse_col: u16,
    mouse_row: u16,
) {
    if provider_idx >= config.providers.len() {
        return;
    }
    let provider = &config.providers[provider_idx];
    let popup_area = absolute_centered_rect(75, 18, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(theme::bg_main()))
        .border_style(Style::new().fg(theme::border_focus()))
        .title(Span::styled(
            format!(" EDIT MODELS - {} ", provider.name.to_uppercase()),
            Style::new().fg(theme::purple()).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    f.render_widget(Paragraph::new(Span::styled("\u{2500}".repeat(inner.width as usize), Style::new().fg(theme::border()))), chunks[1]);

    let bottom_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(16),
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Min(0),
        ])
        .split(chunks[2]);

    let m = provider.models.len();

    let add_focused = focus_idx == 3 * m;
    let add_hovered = is_hovering(bottom_cols[1], mouse_col, mouse_row);
    let add_style = if add_focused {
        Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)
    } else if add_hovered {
        Style::new().fg(theme::success()).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(theme::success())
    };
    let add_prefix = if add_focused { "> " } else { "  " };
    f.render_widget(Paragraph::new(Line::from(vec![
        Span::styled(add_prefix, Style::new().fg(theme::border_focus())),
        Span::styled("[+ Add Model]", add_style)
    ])), bottom_cols[1]);
    hit_registry.push(ClickTarget {
        rect: bottom_cols[1],
        action: ClickAction::AddModel,
    });

    let close_focused = focus_idx == 3 * m + 1;
    let close_hovered = is_hovering(bottom_cols[3], mouse_col, mouse_row);
    let close_style = if close_focused {
        Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)
    } else if close_hovered {
        Style::new().fg(theme::accent()).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(theme::accent())
    };
    let close_prefix = if close_focused { "> " } else { "  " };
    f.render_widget(Paragraph::new(Line::from(vec![
        Span::styled(close_prefix, Style::new().fg(theme::border_focus())),
        Span::styled("[Close]", close_style)
    ])), bottom_cols[3]);
    hit_registry.push(ClickTarget {
        rect: bottom_cols[3],
        action: ClickAction::SwitchView(View::Settings),
    });

    let list_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(2); m])
        .split(chunks[0]);

    for i in 0..m {
        let model = &provider.models[i];
        let row_area = list_chunks[i];

        let row_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(45),
                Constraint::Percentage(45),
                Constraint::Length(11),
            ])
            .split(row_area);

        // Name (3 * i)
        let name_focused = focus_idx == 3 * i;
        let name_editing = name_focused && edit_buffer.is_some();
        let name_hover = is_hovering(row_cols[0], mouse_col, mouse_row);
        let name_val = if name_editing {
            let buf = edit_buffer.unwrap_or("");
            let cur = edit_cursor.min(buf.len());
            format!("{}{}{}", &buf[..cur], "\u{258E}", &buf[cur..])
        } else {
            model.name.clone()
        };
        let name_style = if name_focused {
            Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)
        } else if name_hover {
            Style::new().fg(theme::border_hover())
        } else {
            Style::new().fg(theme::text_main())
        };
        let name_line = Line::from(vec![
            Span::styled(if name_focused { "> " } else { "  " }, Style::new().fg(theme::border_focus())),
            Span::styled("Name: [", Style::new().fg(theme::text_dim())),
            Span::styled(name_val, name_style),
            Span::styled("]", Style::new().fg(theme::text_dim())),
        ]);
        f.render_widget(Paragraph::new(name_line), row_cols[0]);
        hit_registry.push(ClickTarget {
            rect: row_cols[0],
            action: ClickAction::ActivateField(3 * i),
        });

        // Model ID (3 * i + 1)
        let id_focused = focus_idx == 3 * i + 1;
        let id_editing = id_focused && edit_buffer.is_some();
        let id_hover = is_hovering(row_cols[1], mouse_col, mouse_row);
        let id_val = if id_editing {
            let buf = edit_buffer.unwrap_or("");
            let cur = edit_cursor.min(buf.len());
            format!("{}{}{}", &buf[..cur], "\u{258E}", &buf[cur..])
        } else {
            model.model_id.clone()
        };
        let id_style = if id_focused {
            Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)
        } else if id_hover {
            Style::new().fg(theme::border_hover())
        } else {
            Style::new().fg(theme::text_main())
        };
        let id_line = Line::from(vec![
            Span::styled(if id_focused { "> " } else { "  " }, Style::new().fg(theme::border_focus())),
            Span::styled("ID: [", Style::new().fg(theme::text_dim())),
            Span::styled(id_val, id_style),
            Span::styled("]", Style::new().fg(theme::text_dim())),
        ]);
        f.render_widget(Paragraph::new(id_line), row_cols[1]);
        hit_registry.push(ClickTarget {
            rect: row_cols[1],
            action: ClickAction::ActivateField(3 * i + 1),
        });

        // Remove (3 * i + 2)
        let rem_focused = focus_idx == 3 * i + 2;
        let rem_hover = is_hovering(row_cols[2], mouse_col, mouse_row);
        let rem_style = if rem_focused {
            Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)
        } else if rem_hover {
            Style::new().fg(theme::error()).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(theme::error())
        };
        let rem_line = Line::from(vec![
            Span::styled(if rem_focused { "> " } else { "  " }, Style::new().fg(theme::border_focus())),
            Span::styled("[Rem]", rem_style),
        ]);
        f.render_widget(Paragraph::new(rem_line), row_cols[2]);
        hit_registry.push(ClickTarget {
            rect: row_cols[2],
            action: ClickAction::RemoveModel(i),
        });
    }
}
