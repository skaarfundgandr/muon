use crate::config::{MuonConfig, SearchProviderConfig, SearchProviderType, ToolsConfig};
use crate::presentation::click::{ClickAction, ClickTarget, is_hovering};
use crate::presentation::form::{FieldDef, FormState};
use crate::presentation::theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn fields() -> &'static [FieldDef] {
    Box::leak(Box::new([
        FieldDef::button("+ Add Search Provider"),
        FieldDef::checkbox("ArXiv Search Enabled"),
    ])) as &'static [FieldDef]
}

pub fn get_field(config: &ToolsConfig, index: usize) -> String {
    match index {
        0 => String::new(),
        1 => config.arxiv_enabled.to_string(),
        _ => String::new(),
    }
}

pub fn set_field(config: &mut ToolsConfig, index: usize, value: &str) {
    if index == 1 {
        config.arxiv_enabled = value == "true";
    }
}

pub fn toggle_field(config: &mut ToolsConfig, index: usize) {
    if index == 1 {
        config.arxiv_enabled = !config.arxiv_enabled;
    }
}

fn is_focused(form: &FormState, index: usize) -> bool {
    form.focus == index
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

fn provider_type_label(t: SearchProviderType) -> &'static str {
    match t {
        SearchProviderType::Tavily => "tavily",
        SearchProviderType::Firecrawl => "firecrawl",
        SearchProviderType::Brave => "brave",
        SearchProviderType::Serper => "serper",
    }
}

const SEARCH_ROW_HEIGHT: u16 = 4;
const ARXIV_ROW_HEIGHT: u16 = 3;
const ADD_BUTTON_HEIGHT: u16 = 3;
const SCROLL_INDICATOR_HEIGHT: u16 = 1;

fn mask_key(key: &str) -> String {
    let n = key.chars().count().min(22);
    "\u{25CF}".repeat(n)
}

fn render_search_row(
    f: &mut ratatui::Frame,
    area: Rect,
    idx: usize,
    p: &SearchProviderConfig,
    focused_row: bool,
    hit_registry: &mut Vec<ClickTarget>,
) {
    let title = format!(
        "#{}  {}",
        idx + 1,
        if p.name.is_empty() {
            "<unnamed>"
        } else {
            p.name.as_str()
        }
    );
    let block = section_block(&title, focused_row, is_hovering(area, 0, 0));
    let inner = block.inner(area);
    f.render_widget(block, area);

    hit_registry.push(ClickTarget {
        rect: area,
        action: ClickAction::FocusField(idx),
    });

    let label_color = if focused_row {
        theme::border_focus()
    } else {
        theme::text_dim()
    };
    let accent_color = if focused_row {
        theme::border_focus()
    } else {
        theme::accent()
    };

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    let type_line = Line::from(vec![
        Span::styled(format!("{:<12}", "Type"), Style::new().fg(label_color)),
        Span::styled("[", Style::new().fg(accent_color)),
        Span::styled(
            provider_type_label(p.provider_type.clone()),
            Style::new().fg(accent_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled("\u{25BC}", Style::new().fg(accent_color)),
        Span::styled("]", Style::new().fg(accent_color)),
    ]);
    f.render_widget(Paragraph::new(type_line), rows[0]);

    let key_line = Line::from(vec![
        Span::styled(format!("{:<12}", "API Key"), Style::new().fg(label_color)),
        Span::styled("[", Style::new().fg(accent_color)),
        Span::styled(mask_key(&p.api_key), Style::new().fg(theme::success())),
        Span::styled("]", Style::new().fg(accent_color)),
    ]);
    f.render_widget(Paragraph::new(key_line), rows[1]);

    let actions_line = Line::from(vec![
        Span::styled(format!("{:<12}", "Options"), Style::new().fg(label_color)),
        Span::styled("[Configure]", Style::new().fg(theme::accent())),
        Span::styled("  ", Style::new().fg(theme::text_dim())),
        Span::styled("[Remove]", Style::new().fg(theme::error())),
    ]);
    f.render_widget(Paragraph::new(actions_line), rows[2]);

    let configure_button = Rect::new(rows[2].x + 12, rows[2].y, 11, 1);
    hit_registry.push(ClickTarget {
        rect: configure_button,
        action: ClickAction::ConfigureSearchOptions(idx),
    });
    let remove_button = Rect::new(rows[2].x + 25, rows[2].y, 9, 1);
    hit_registry.push(ClickTarget {
        rect: remove_button,
        action: ClickAction::RemoveSearchProvider(idx),
    });
}

fn render_arxiv_row(
    f: &mut ratatui::Frame,
    area: Rect,
    config: &ToolsConfig,
    form: &FormState,
    hit_registry: &mut Vec<ClickTarget>,
) {
    let focused = is_focused(form, 1);
    let block = section_block(
        "PAPER SEARCH",
        focused,
        is_hovering(area, form.mouse_col, form.mouse_row) && !focused,
    );
    let inner = block.inner(area);
    f.render_widget(block, area);

    hit_registry.push(ClickTarget {
        rect: area,
        action: ClickAction::FocusField(1),
    });

    let arxiv_mark = if config.arxiv_enabled {
        "[\u{2713}]"
    } else {
        "[ ]"
    };
    let line = Line::from(vec![
        Span::styled(
            "ArXiv  ",
            if focused {
                Style::new()
                    .fg(theme::border_focus())
                    .add_modifier(Modifier::BOLD)
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

fn render_add_button(f: &mut ratatui::Frame, area: Rect, hit_registry: &mut Vec<ClickTarget>) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme::border_hover()))
        .title(Span::styled(
            " ADD ",
            Style::new()
                .fg(theme::success())
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    f.render_widget(block, area);
    let line = Line::from(vec![Span::styled(
        "[+ Add Search Provider]",
        Style::new()
            .fg(theme::success())
            .add_modifier(Modifier::BOLD),
    )]);
    f.render_widget(Paragraph::new(line), inner);
    hit_registry.push(ClickTarget {
        rect: area,
        action: ClickAction::AddSearchProvider,
    });
}

#[allow(clippy::too_many_arguments)]
pub fn render(
    f: &mut ratatui::Frame,
    area: Rect,
    config: &MuonConfig,
    form: &FormState,
    hit_registry: &mut Vec<ClickTarget>,
    mouse_col: u16,
    mouse_row: u16,
) {
    let _ = (mouse_col, mouse_row);
    let chrome_height =
        ARXIV_ROW_HEIGHT + ADD_BUTTON_HEIGHT;
    let list_area_height = area.height.saturating_sub(chrome_height);
    let n = config.search.providers.len();
    let scroll_offset = form.scroll_offset;
    let max_no_indicators = (list_area_height / SEARCH_ROW_HEIGHT) as usize;
    let (max_visible_rows, show_top, show_bottom) = if n <= max_no_indicators {
        (n, false, false)
    } else {
        let worst = ((list_area_height.saturating_sub(SCROLL_INDICATOR_HEIGHT * 2))
            / SEARCH_ROW_HEIGHT) as usize;
        let worst = worst.max(1);
        if scroll_offset + worst < n {
            (worst, scroll_offset > 0, true)
        } else {
            let refined = ((list_area_height.saturating_sub(SCROLL_INDICATOR_HEIGHT))
                / SEARCH_ROW_HEIGHT) as usize;
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
        let line = Line::from(vec![Span::styled(
            "No search providers configured. Add one below to enable web search fan-out.",
            Style::new().fg(theme::text_dim()),
        )]);
        f.render_widget(Paragraph::new(line), outer[0]);
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

        let focused_row = form.focus < n;
        for (i, p) in config
            .search
            .providers
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(max_visible_rows)
        {
            render_search_row(
                f,
                rows[idx],
                i,
                p,
                focused_row && form.focus == i,
                hit_registry,
            );
            idx += 1;
        }

        if show_bottom {
            let line = Line::from(Span::styled(
                "\u{2193} more below",
                Style::new().fg(theme::text_dim()),
            ));
            f.render_widget(Paragraph::new(line), rows[idx]);
        }
    }

    render_arxiv_row(f, outer[1], &config.tools, form, hit_registry);
    render_add_button(f, outer[2], hit_registry);
}
