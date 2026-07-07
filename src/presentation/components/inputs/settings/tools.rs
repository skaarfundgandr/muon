use crate::config::{MuonConfig, SearchProviderConfig, SearchProviderType};
use crate::presentation::views::View;
use crate::presentation::click::{is_hovering, ClickAction, ClickTarget};
use crate::presentation::form::{FieldDef, FormState};
use crate::presentation::theme;
use crate::presentation::components::inputs::settings::dropdown_overlay::PendingDropdown;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Clear};

pub fn fields(config: &MuonConfig) -> Vec<FieldDef> {
    let mut f = Vec::new();
    for _ in &config.search.providers {
        f.push(FieldDef::text("Name"));
        f.push(FieldDef::dropdown("Type", &["tavily", "firecrawl", "brave", "serper"]));
        f.push(FieldDef::text("API Key"));
        f.push(FieldDef::button("Configure"));
        f.push(FieldDef::button("Remove"));
    }
    f.push(FieldDef::button("+ Add Search Provider"));
    f.push(FieldDef::checkbox("ArXiv Search Enabled"));
    f
}

pub fn get_field(config: &MuonConfig, index: usize) -> String {
    let n = config.search.providers.len();
    if index < 5 * n {
        let provider_idx = index / 5;
        let sub_idx = index % 5;
        let p = &config.search.providers[provider_idx];
        match sub_idx {
            0 => p.name.clone(),
            1 => match p.provider_type {
                SearchProviderType::Tavily => "tavily".to_string(),
                SearchProviderType::Firecrawl => "firecrawl".to_string(),
                SearchProviderType::Brave => "brave".to_string(),
                SearchProviderType::Serper => "serper".to_string(),
            },
            2 => p.api_key.clone(),
            _ => String::new(),
        }
    } else if index == 5 * n {
        String::new()
    } else if index == 5 * n + 1 {
        config.search.papers.arxiv_enabled.to_string()
    } else {
        String::new()
    }
}

pub fn set_field(config: &mut MuonConfig, index: usize, value: &str) {
    let n = config.search.providers.len();
    if index < 5 * n {
        let provider_idx = index / 5;
        let sub_idx = index % 5;
        let p = &mut config.search.providers[provider_idx];
        match sub_idx {
            0 => p.name = value.to_string(),
            1 => p.provider_type = match value {
                "tavily" => SearchProviderType::Tavily,
                "firecrawl" => SearchProviderType::Firecrawl,
                "brave" => SearchProviderType::Brave,
                "serper" => SearchProviderType::Serper,
                _ => SearchProviderType::Tavily,
            },
            2 => p.api_key = value.to_string(),
            _ => {}
        }
    } else if index == 5 * n + 1 {
        config.search.papers.arxiv_enabled = value == "true";
    }
}

pub fn toggle_field(config: &mut MuonConfig, index: usize) {
    let n = config.search.providers.len();
    if index == 5 * n + 1 {
        config.search.papers.arxiv_enabled = !config.search.papers.arxiv_enabled;
    }
}

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

const SEARCH_ROW_HEIGHT: u16 = 6;
const ARXIV_ROW_HEIGHT: u16 = 3;
const ADD_BUTTON_HEIGHT: u16 = 3;
const SCROLL_INDICATOR_HEIGHT: u16 = 1;

#[allow(clippy::too_many_arguments)]
fn render_search_row(
    f: &mut ratatui::Frame,
    area: Rect,
    idx: usize,
    p: &SearchProviderConfig,
    focused_sub_idx: Option<usize>,
    form: &FormState,
    hovered: bool,
    hit_registry: &mut Vec<ClickTarget>,
    mouse_col: u16,
    mouse_row: u16,
) {
    let title = format!(
        "#{}  {}",
        idx + 1,
        if p.name.is_empty() { "<unnamed>" } else { p.name.as_str() }
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
        ])
        .split(inner);

    // Register row click targets
    for (sub, r) in rows.iter().enumerate() {
        hit_registry.push(ClickTarget {
            rect: *r,
            action: ClickAction::ActivateField(5 * idx + sub),
        });
    }

    // Name (0)
    let name_focused = focused_sub_idx == Some(0);
    let name_editing = name_focused && form.is_editing();
    let name_hover = is_hovering(rows[0], mouse_col, mouse_row);
    f.render_widget(
        Paragraph::new(field_line(
            "Name",
            &p.name,
            name_focused,
            name_editing,
            form.edit_cursor,
            form.edit_buffer.as_deref(),
            name_hover,
            theme::success(),
        )),
        rows[0],
    );

    // Type (1)
    let type_focused = focused_sub_idx == Some(1);
    let type_hover = is_hovering(rows[1], mouse_col, mouse_row);
    let val_color = if type_focused {
        theme::border_focus()
    } else if type_hover {
        theme::border_hover()
    } else {
        theme::cyan()
    };
    let prefix = if type_focused { "> " } else { "  " };
    let border_style = if type_focused {
        Style::new().fg(theme::border_focus())
    } else {
        Style::new().fg(theme::text_dim())
    };
    let label_style = if type_focused {
        Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(theme::text_dim())
    };
    let type_str = match p.provider_type {
        SearchProviderType::Tavily => "tavily",
        SearchProviderType::Firecrawl => "firecrawl",
        SearchProviderType::Brave => "brave",
        SearchProviderType::Serper => "serper",
    };
    let type_line = Line::from(vec![
        Span::styled(prefix, Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{:<10}", "Type"), label_style),
        Span::styled("[", border_style),
        Span::styled(type_str, Style::new().fg(val_color).add_modifier(Modifier::BOLD)),
        Span::styled("▼", border_style),
        Span::styled("]", border_style),
    ]);
    f.render_widget(Paragraph::new(type_line), rows[1]);

    // API Key (2)
    let key_focused = focused_sub_idx == Some(2);
    let key_editing = key_focused && form.is_editing();
    let key_hover = is_hovering(rows[2], mouse_col, mouse_row);
    let key_val = if key_editing {
        form.edit_buffer.clone().unwrap_or_default()
    } else if p.api_key.is_empty() {
        "".to_string()
    } else {
        mask_key(&p.api_key)
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
        rows[2],
    );

    // Actions (3: Configure, 4: Remove)
    let config_focused = focused_sub_idx == Some(3);
    let config_hover = is_hovering(rows[3], mouse_col, mouse_row);
    let conf_style = if config_focused {
        Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)
    } else if config_hover {
        Style::new().fg(theme::accent()).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(theme::accent())
    };
    
    let remove_focused = focused_sub_idx == Some(4);
    let remove_hover = is_hovering(rows[3], mouse_col, mouse_row); // wait, refine mouse hover checking for columns
    let rem_style = if remove_focused {
        Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)
    } else if remove_hover {
        Style::new().fg(theme::error()).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(theme::error())
    };

    let actions_prefix = if config_focused { "> " } else { "  " };
    let actions_line = Line::from(vec![
        Span::styled(actions_prefix, Style::new().fg(theme::border_focus())),
        Span::styled(format!("{:<10}  ", "Options"), Style::new().fg(theme::text_dim())),
        Span::styled("[Configure]", conf_style),
        Span::styled("    ", Style::new().fg(theme::text_dim())),
        Span::styled("[Remove]", rem_style),
    ]);
    f.render_widget(Paragraph::new(actions_line), rows[3]);

    let configure_button = Rect::new(rows[3].x + 14, rows[3].y, 11, 1);
    hit_registry.push(ClickTarget {
        rect: configure_button,
        action: ClickAction::ConfigureSearchOptions(idx),
    });
    let remove_button = Rect::new(rows[3].x + 29, rows[3].y, 8, 1);
    hit_registry.push(ClickTarget {
        rect: remove_button,
        action: ClickAction::RemoveSearchProvider(idx),
    });
}

fn render_arxiv_row(
    f: &mut ratatui::Frame,
    area: Rect,
    enabled: bool,
    focused: bool,
    hovered: bool,
    hit_registry: &mut Vec<ClickTarget>,
) {
    let block = section_block("PAPER SEARCH", focused, hovered);
    let inner = block.inner(area);
    f.render_widget(block, area);

    hit_registry.push(ClickTarget {
        rect: area,
        action: ClickAction::ToggleArxiv,
    });

    let arxiv_mark = if enabled {
        "[\u{2713}]"
    } else {
        "[ ]"
    };
    let prefix = if focused { "> " } else { "  " };
    let line = Line::from(vec![
        Span::styled(prefix, Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)),
        Span::styled(
            "ArXiv  ",
            if focused {
                Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(theme::text_dim())
            },
        ),
        Span::styled(
            arxiv_mark,
            if focused {
                Style::new().fg(theme::border_focus())
            } else {
                Style::new().fg(theme::success())
            },
        ),
        Span::styled(
            "  Enabled (no key required)",
            Style::new().fg(theme::text_dim()),
        ),
    ]);
    f.render_widget(Paragraph::new(line), inner);
}

fn render_add_button(
    f: &mut ratatui::Frame,
    area: Rect,
    focused: bool,
    hovered: bool,
    hit_registry: &mut Vec<ClickTarget>,
) {
    let border_color = if focused {
        theme::border_focus()
    } else if hovered {
        theme::border_hover()
    } else {
        theme::border()
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
            "[+ Add Search Provider]",
            Style::new().fg(theme::success()).add_modifier(Modifier::BOLD),
        )
    ]);
    f.render_widget(Paragraph::new(line), inner);

    hit_registry.push(ClickTarget {
        rect: area,
        action: ClickAction::AddSearchProvider,
    });
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
    let n = config.search.providers.len();
    let chrome_height = ARXIV_ROW_HEIGHT + ADD_BUTTON_HEIGHT;
    let list_area_height = area.height.saturating_sub(chrome_height);
    let scroll_offset = form.scroll_offset;
    let max_no_indicators = (list_area_height / SEARCH_ROW_HEIGHT) as usize;
    let (max_visible_rows, show_top, show_bottom) = if n <= max_no_indicators {
        (n, false, false)
    } else {
        let worst = ((list_area_height.saturating_sub(SCROLL_INDICATOR_HEIGHT * 2)) / SEARCH_ROW_HEIGHT) as usize;
        let worst = worst.max(1);
        if scroll_offset + worst < n {
            (worst, scroll_offset > 0, true)
        } else {
            let refined = ((list_area_height.saturating_sub(SCROLL_INDICATOR_HEIGHT)) / SEARCH_ROW_HEIGHT) as usize;
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
            Constraint::Length(ARXIV_ROW_HEIGHT),
            Constraint::Length(ADD_BUTTON_HEIGHT),
        ])
        .split(area);

    if n == 0 {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::new().fg(theme::border()))
            .title(Span::styled(" SEARCH PROVIDERS ", Style::new().fg(theme::purple()).add_modifier(Modifier::BOLD)));
        let inner = block.inner(outer[0]);
        f.render_widget(block, outer[0]);
        let line = Line::from(vec![Span::styled(
            "No search providers configured. Add one below to enable web search fan-out.",
            Style::new().fg(theme::text_dim()),
        )]);
        f.render_widget(Paragraph::new(line), inner);
    } else {
        let mut constraints: Vec<Constraint> = Vec::new();
        if show_top {
            constraints.push(Constraint::Length(SCROLL_INDICATOR_HEIGHT));
        }
        for _ in scroll_offset..scroll_end {
            constraints.push(Constraint::Length(SEARCH_ROW_HEIGHT));
        }
        if show_bottom {
            constraints.push(Constraint::Length(SCROLL_INDICATOR_HEIGHT));
        }
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
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

        let mut dropdown_rect = None;
        let focused_row = form.focus < 5 * n;
        for (i, p) in config
            .search
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
            if focused_sub_idx == Some(1) {
                let block = section_block("", true, false);
                let inner = block.inner(rows[idx]);
                let provider_rows = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                    ])
                    .split(inner);
                dropdown_rect = Some(provider_rows[1]);
            }
            render_search_row(
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
            );
            idx += 1;
        }

        if let Some(rect) = dropdown_rect && form.dropdown_open {
            let field_label = "Type";
            let options: Vec<String> = vec![
                "tavily".to_string(),
                "firecrawl".to_string(),
                "brave".to_string(),
                "serper".to_string(),
            ];
            *pending_dropdown = Some(PendingDropdown {
                below: rect,
                field_label: field_label.to_string(),
                options,
            });
        }

        if show_bottom {
            let line = Line::from(Span::styled(
                "\u{2193} more below",
                Style::new().fg(theme::text_dim()),
            ));
            f.render_widget(Paragraph::new(line), rows[idx]);
        }
    }

    let arxiv_focused = form.focus == 5 * n + 1;
    let arxiv_hovered = is_hovering(outer[1], mouse_col, mouse_row);
    render_arxiv_row(f, outer[1], config.search.papers.arxiv_enabled, arxiv_focused, arxiv_hovered, hit_registry);

    let add_focused = form.focus == 5 * n;
    let add_hovered = is_hovering(outer[2], mouse_col, mouse_row);
    render_add_button(f, outer[2], add_focused, add_hovered, hit_registry);
}

fn absolute_centered_rect(w: u16, h: u16, r: Rect) -> Rect {
    let x = r.x + r.width.saturating_sub(w) / 2;
    let y = r.y + r.height.saturating_sub(h) / 2;
    Rect::new(x, y, w.min(r.width), h.min(r.height))
}

#[allow(clippy::too_many_arguments)]
pub fn render_configure_popup(
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
    if provider_idx >= config.search.providers.len() {
        return;
    }
    let p = &config.search.providers[provider_idx];
    let popup_area = absolute_centered_rect(65, 12, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(theme::bg_main()))
        .border_style(Style::new().fg(theme::border_focus()))
        .title(Span::styled(
            format!(" CONFIGURE SEARCH - {} ", p.name.to_uppercase()),
            Style::new().fg(theme::purple()).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    // Row click targets
    for i in 0..3 {
        hit_registry.push(ClickTarget {
            rect: chunks[i],
            action: ClickAction::ActivateField(i),
        });
    }

    // Name (0)
    let name_focused = focus_idx == 0;
    let name_editing = name_focused && edit_buffer.is_some();
    let name_hover = is_hovering(chunks[0], mouse_col, mouse_row);
    let name_val = if name_editing {
        let buf = edit_buffer.unwrap_or("");
        let cur = edit_cursor.min(buf.len());
        format!("{}{}{}", &buf[..cur], "\u{258E}", &buf[cur..])
    } else {
        p.name.clone()
    };
    let name_style = if name_focused {
        Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)
    } else if name_hover {
        Style::new().fg(theme::border_hover())
    } else {
        Style::new().fg(theme::text_main())
    };
    f.render_widget(Paragraph::new(Line::from(vec![
        Span::styled(if name_focused { "> " } else { "  " }, Style::new().fg(theme::border_focus())),
        Span::styled("Name: [", Style::new().fg(theme::text_dim())),
        Span::styled(name_val, name_style),
        Span::styled("]", Style::new().fg(theme::text_dim())),
    ])), chunks[0]);

    // API Key (1)
    let key_focused = focus_idx == 1;
    let key_editing = key_focused && edit_buffer.is_some();
    let key_hover = is_hovering(chunks[1], mouse_col, mouse_row);
    let key_val = if key_editing {
        let buf = edit_buffer.unwrap_or("");
        let cur = edit_cursor.min(buf.len());
        format!("{}{}{}", &buf[..cur], "\u{258E}", &buf[cur..])
    } else if p.api_key.is_empty() {
        "".to_string()
    } else {
        mask_key(&p.api_key)
    };
    let key_style = if key_focused {
        Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)
    } else if key_hover {
        Style::new().fg(theme::border_hover())
    } else {
        Style::new().fg(theme::success())
    };
    f.render_widget(Paragraph::new(Line::from(vec![
        Span::styled(if key_focused { "> " } else { "  " }, Style::new().fg(theme::border_focus())),
        Span::styled("API Key: [", Style::new().fg(theme::text_dim())),
        Span::styled(key_val, key_style),
        Span::styled("]", Style::new().fg(theme::text_dim())),
    ])), chunks[1]);

    // Max Results (2)
    let max_focused = focus_idx == 2;
    let max_editing = max_focused && edit_buffer.is_some();
    let max_hover = is_hovering(chunks[2], mouse_col, mouse_row);
    let max_val = if max_editing {
        let buf = edit_buffer.unwrap_or("");
        let cur = edit_cursor.min(buf.len());
        format!("{}{}{}", &buf[..cur], "\u{258E}", &buf[cur..])
    } else {
        p.max_results.map(|x| x.to_string()).unwrap_or_default()
    };
    let max_style = if max_focused {
        Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)
    } else if max_hover {
        Style::new().fg(theme::border_hover())
    } else {
        Style::new().fg(theme::cyan())
    };
    f.render_widget(Paragraph::new(Line::from(vec![
        Span::styled(if max_focused { "> " } else { "  " }, Style::new().fg(theme::border_focus())),
        Span::styled("Max Results: [", Style::new().fg(theme::text_dim())),
        Span::styled(max_val, max_style),
        Span::styled("]", Style::new().fg(theme::text_dim())),
    ])), chunks[2]);

    // Divider
    f.render_widget(Paragraph::new(Span::styled("\u{2500}".repeat(inner.width as usize), Style::new().fg(theme::border()))), chunks[4]);

    // Bottom buttons (Save & Close, Cancel)
    let bottom_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(16),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Min(0),
        ])
        .split(chunks[5]);

    let save_focused = focus_idx == 3;
    let save_hover = is_hovering(bottom_cols[1], mouse_col, mouse_row);
    let save_style = if save_focused {
        Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)
    } else if save_hover {
        Style::new().fg(theme::success()).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(theme::success())
    };
    let save_prefix = if save_focused { "> " } else { "  " };
    f.render_widget(Paragraph::new(Line::from(vec![
        Span::styled(save_prefix, Style::new().fg(theme::border_focus())),
        Span::styled("[Save & Close]", save_style)
    ])), bottom_cols[1]);
    hit_registry.push(ClickTarget {
        rect: bottom_cols[1],
        action: ClickAction::SwitchView(View::Settings), // mapped to exit/save
    });

    let cancel_focused = focus_idx == 4;
    let cancel_hover = is_hovering(bottom_cols[3], mouse_col, mouse_row);
    let cancel_style = if cancel_focused {
        Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)
    } else if cancel_hover {
        Style::new().fg(theme::error()).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(theme::error())
    };
    let cancel_prefix = if cancel_focused { "> " } else { "  " };
    f.render_widget(Paragraph::new(Line::from(vec![
        Span::styled(cancel_prefix, Style::new().fg(theme::border_focus())),
        Span::styled("[Cancel]", cancel_style)
    ])), bottom_cols[3]);
    hit_registry.push(ClickTarget {
        rect: bottom_cols[3],
        action: ClickAction::SwitchView(View::Settings), // Cancel
    });
}
