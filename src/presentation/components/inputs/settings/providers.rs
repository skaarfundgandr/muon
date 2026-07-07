use crate::config::{MuonConfig, ProviderConfig};
use crate::presentation::click::{is_hovering, ClickAction, ClickTarget};
use crate::presentation::form::{FieldDef, FormState};
use crate::presentation::theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn fields() -> &'static [FieldDef] {
    Box::leak(Box::new([FieldDef::button("+ Add Provider")])) as &'static [FieldDef]
}

pub fn get_field(_config: &MuonConfig, _index: usize) -> String {
    String::new()
}

pub fn set_field(_config: &mut MuonConfig, _index: usize, _value: &str) {}

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
    "\u{25CF}".repeat(n)
}

fn row_height(_provider: &ProviderConfig, area_width: u16) -> u16 {
    let _ = area_width;
    5
}

const ROW_HEIGHT: u16 = 5;
const ADD_BUTTON_HEIGHT: u16 = 3;
const COUNT_SUMMARY_HEIGHT: u16 = 1;
const SCROLL_INDICATOR_HEIGHT: u16 = 1;

fn render_provider_row(
    f: &mut ratatui::Frame,
    area: Rect,
    idx: usize,
    provider: &ProviderConfig,
    focused_row: bool,
    hovered: bool,
    hit_registry: &mut Vec<ClickTarget>,
) {
    let title = format!(
        "#{}  {}",
        idx + 1,
        if provider.name.is_empty() { "<unnamed>" } else { provider.name.as_str() }
    );
    let block = section_block(&title, focused_row, hovered);
    let inner = block.inner(area);
    f.render_widget(block, area);

    hit_registry.push(ClickTarget {
        rect: area,
        action: ClickAction::FocusField(idx),
    });

    let name_color = if focused_row {
        theme::border_focus()
    } else {
        theme::text_main()
    };
    let name_value_color = if focused_row {
        theme::border_focus()
    } else {
        theme::success()
    };
    let url_value_color = if focused_row {
        theme::border_focus()
    } else {
        theme::cyan()
    };

    let key_mask = mask_key(&provider.api_key);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    let label_style = if focused_row {
        Style::new().fg(theme::border_focus()).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(theme::text_dim())
    };

    let name_line = Line::from(vec![
        Span::styled(format!("{:<14}", "Name"), label_style),
        Span::styled("[", Style::new().fg(name_color)),
        Span::styled(if provider.name.is_empty() { "<click Edit to set>" } else { provider.name.as_str() }, Style::new().fg(name_value_color)),
        Span::styled("]", Style::new().fg(name_color)),
    ]);
    f.render_widget(Paragraph::new(name_line), rows[0]);

    let url_line = Line::from(vec![
        Span::styled(format!("{:<14}", "Base URL"), label_style),
        Span::styled("[", Style::new().fg(name_color)),
        Span::styled(if provider.base_url.is_empty() { "<click Edit to set>" } else { provider.base_url.as_str() }, Style::new().fg(url_value_color)),
        Span::styled("]", Style::new().fg(name_color)),
    ]);
    f.render_widget(Paragraph::new(url_line), rows[1]);

    let key_line = Line::from(vec![
        Span::styled(format!("{:<14}", "API Key"), label_style),
        Span::styled("[", Style::new().fg(name_color)),
        Span::styled(key_mask, Style::new().fg(theme::success())),
        Span::styled("]", Style::new().fg(name_color)),
    ]);
    f.render_widget(Paragraph::new(key_line), rows[2]);

    let model_count = provider.models.len();
    let model_summary = if model_count == 0 {
        "(no models)".to_string()
    } else if model_count == 1 {
        "1 model".to_string()
    } else {
        format!("{model_count} models")
    };
    let models_line = Line::from(vec![
        Span::styled(format!("{:<14}", "Models"), label_style),
        Span::styled(model_summary, Style::new().fg(theme::accent())),
        Span::styled("  ", Style::new().fg(theme::text_dim())),
        Span::styled("[Edit Models]", Style::new().fg(theme::accent())),
    ]);
    f.render_widget(Paragraph::new(models_line), rows[3]);

    let actions_row = Rect::new(
        inner.x,
        inner.y.saturating_add(inner.height.saturating_sub(1)),
        inner.width,
        1,
    );
    let _ = row_height(provider, inner.width);
    let actions_line = Line::from(vec![
        Span::styled(" ", Style::new().fg(theme::text_dim())),
        Span::styled("[Remove]", Style::new().fg(theme::error())),
    ]);
    f.render_widget(Paragraph::new(actions_line), actions_row);

    let remove_button_rect = Rect::new(
        actions_row.x,
        actions_row.y,
        actions_row.width.min(10),
        1,
    );
    hit_registry.push(ClickTarget {
        rect: remove_button_rect,
        action: ClickAction::RemoveProvider(idx),
    });
    let edit_models_rect = Rect::new(
        inner.x + 16,
        rows[3].y,
        16,
        1,
    );
    hit_registry.push(ClickTarget {
        rect: edit_models_rect,
        action: ClickAction::EditProviderModels(idx),
    });
}

fn render_add_button(
    f: &mut ratatui::Frame,
    area: Rect,
    hit_registry: &mut Vec<ClickTarget>,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme::border_hover()))
        .title(Span::styled(
            " ADD ",
            Style::new().fg(theme::success()).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let line = Line::from(vec![Span::styled(
        "[+ Add Provider]",
        Style::new().fg(theme::success()).add_modifier(Modifier::BOLD),
    )]);
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

        let focused_row = form.focus < n;
        for (i, p) in config
            .providers
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(max_visible_rows)
        {
            let hovered = is_hovering(rows[idx], mouse_col, mouse_row);
            render_provider_row(
                f,
                rows[idx],
                i,
                p,
                focused_row && form.focus == i,
                hovered,
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
            idx += 1;
        }

        let mut spans: Vec<Span> = Vec::new();
        spans.push(Span::styled(
            format!("{n} provider(s) configured"),
            Style::new().fg(theme::text_dim()),
        ));
        f.render_widget(Paragraph::new(Line::from(spans)), rows[idx]);
    }

    render_add_button(f, outer[1], hit_registry);
}
