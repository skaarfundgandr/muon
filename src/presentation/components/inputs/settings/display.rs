use crate::config::DisplayConfig;
use crate::presentation::click::{ClickAction, ClickTarget};
use crate::presentation::form::{FieldDef, FormState};
use crate::presentation::theme::{BORDER, BORDER_FOCUS, CYAN, PURPLE, SUCCESS, TEXT_DIM, TEXT_MAIN, WARNING};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn fields() -> &'static [FieldDef] {
    Box::leak(Box::new([
        FieldDef::dropdown("Visual Theme", &["Tokyo Night", "Gruvbox", "Catppuccin", "Nord"]),
        FieldDef::dropdown("Font Size", &["Small 12px", "Medium 14px", "Large 16px"]),
    ])) as &'static [FieldDef]
}

pub fn get_field(config: &DisplayConfig, index: usize) -> String {
    match index {
        0 => config.visual_theme.clone(),
        1 => config.font_size.clone(),
        _ => String::new(),
    }
}

pub fn set_field(config: &mut DisplayConfig, index: usize, value: &str) {
    match index {
        0 => config.visual_theme = value.to_string(),
        1 => config.font_size = value.to_string(),
        _ => {}
    }
}

pub fn toggle_field(_config: &mut DisplayConfig, _index: usize) {}

fn is_focused(form: &FormState, index: usize) -> bool {
    form.focus == index
}

pub fn render(f: &mut ratatui::Frame, area: Rect, config: &DisplayConfig, form: &FormState, hit_registry: &mut Vec<ClickTarget>, _mouse_col: u16, _mouse_row: u16) {
    let grid = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_left(f, grid[0], config, form, hit_registry);
    render_right(f, grid[1], config);
}

fn section_block<'a>(title: &'a str) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(BORDER))
        .title(Span::styled(
            format!(" {} ", title),
            Style::new().fg(PURPLE).add_modifier(Modifier::BOLD),
        ))
}

fn dropdown_line<'a>(label: &'a str, value: &'a str, focused: bool, hovered: bool) -> Line<'a> {
    if focused {
        Line::from(vec![
            Span::styled("> ", Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:<14}", label), Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)),
            Span::styled("[", Style::new().fg(BORDER_FOCUS)),
            Span::styled(value, Style::new().fg(BORDER_FOCUS).add_modifier(Modifier::BOLD)),
            Span::styled("\u{25BC}", Style::new().fg(BORDER_FOCUS)),
            Span::styled("]", Style::new().fg(BORDER_FOCUS)),
        ])
    } else if hovered {
        Line::from(vec![
            Span::styled("  ", Style::new().fg(crate::presentation::theme::BORDER_HOVER)),
            Span::styled(format!("{:<14}", label), Style::new().fg(crate::presentation::theme::BORDER_HOVER)),
            Span::styled("[", Style::new().fg(crate::presentation::theme::BORDER_HOVER)),
            Span::styled(value, Style::new().fg(TEXT_MAIN)),
            Span::styled("\u{25BC}", Style::new().fg(crate::presentation::theme::BORDER_HOVER)),
            Span::styled("]", Style::new().fg(crate::presentation::theme::BORDER_HOVER)),
        ])
    } else {
        Line::from(vec![
            Span::styled("  ", Style::new().fg(TEXT_DIM)),
            Span::styled(format!("{:<14}", label), Style::new().fg(TEXT_DIM)),
            Span::styled("[", Style::new().fg(TEXT_DIM)),
            Span::styled(value, Style::new().fg(TEXT_MAIN)),
            Span::styled("\u{25BC}", Style::new().fg(TEXT_DIM)),
            Span::styled("]", Style::new().fg(TEXT_DIM)),
        ])
    }
}

fn info_row<'a>(label: &'a str, value: &'a str, val_style: Style) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("{:<22}", label), Style::new().fg(TEXT_DIM)),
        Span::styled(value, val_style.add_modifier(Modifier::BOLD)),
    ])
}

fn render_left(
    f: &mut ratatui::Frame,
    area: Rect,
    config: &DisplayConfig,
    form: &FormState,
    hit_registry: &mut Vec<ClickTarget>,
) {
    let block = section_block("TERMINAL DISPLAY SETTINGS");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    hit_registry.push(ClickTarget {
        rect: chunks[0],
        action: ClickAction::ActivateField(0),
    });
    hit_registry.push(ClickTarget {
        rect: chunks[1],
        action: ClickAction::ActivateField(1),
    });

    f.render_widget(
        dropdown_line("Visual Theme", &config.visual_theme, is_focused(form, 0), crate::presentation::click::is_hovering(chunks[0], form.mouse_col, form.mouse_row) && !is_focused(form, 0)),
        chunks[0],
    );
    f.render_widget(dropdown_line("Font Size", &config.font_size, is_focused(form, 1), crate::presentation::click::is_hovering(chunks[1], form.mouse_col, form.mouse_row) && !is_focused(form, 1)), chunks[1]);
    f.render_widget(
        Paragraph::new(Span::styled("Live Preview", Style::new().fg(TEXT_DIM))),
        chunks[2],
    );


    let preview_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(BORDER));
    let preview_inner = preview_block.inner(chunks[3]);
    f.render_widget(preview_block, chunks[3]);

    let preview_lines: Vec<Line> = vec![
        Line::from(Span::styled(
            "The quick brown fox jumps over the lazy dog.",
            Style::new().fg(TEXT_MAIN),
        )),
        Line::from(Span::styled(
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            Style::new().fg(TEXT_MAIN),
        )),
        Line::from(Span::styled(
            "abcdefghijklmnopqrstuvwxyz",
            Style::new().fg(TEXT_MAIN),
        )),
        Line::from(Span::styled("0123456789", Style::new().fg(TEXT_MAIN))),
    ];
    f.render_widget(Paragraph::new(preview_lines), preview_inner);

    if form.dropdown_open && (form.focus == 0 || form.focus == 1) {
        crate::presentation::components::inputs::settings::dropdown_overlay::render_dropdown_overlay(
            f, chunks[form.focus], crate::presentation::components::inputs::settings::display::fields(), form, hit_registry, form.mouse_col, form.mouse_row,
        );
    }
}

fn render_right(f: &mut ratatui::Frame, area: Rect, config: &DisplayConfig) {
    let block = section_block("STATUS BAR & ENVIRONMENT INFO (READ-ONLY)");
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Pull the numeric px out of the saved font_size option (e.g. "Medium 14px").
    let px = config
        .font_size
        .split_whitespace()
        .find(|t| t.chars().any(|c| c.is_ascii_digit()))
        .unwrap_or("14");
    let font_stack = format!("{} / {} / JetBrains Mono", config.visual_theme, px);

    let lines: Vec<Line> = vec![
        info_row(
            "Active Renderer:",
            "HTML TUI Emulator (Bex/Ratatui Mock)",
            Style::new().fg(CYAN),
        ),
        info_row("Font Stack:", &font_stack, Style::new().fg(PURPLE)),
        info_row(
            "Terminal Encoding:",
            "UTF-8 / Unicode Standard",
            Style::new().fg(SUCCESS),
        ),
        info_row(
            "Color Standard:",
            "True Color (24-bit RGB)",
            Style::new().fg(WARNING),
        ),
        info_row(
            "Window Size:",
            "1200 x 800 (Simulated Viewport)",
            Style::new().fg(CYAN),
        ),
        info_row(
            "Note:",
            "Font size is terminal-emulator controlled.",
            Style::new().fg(TEXT_DIM),
        ),
    ];
    f.render_widget(Paragraph::new(lines), inner);
}
