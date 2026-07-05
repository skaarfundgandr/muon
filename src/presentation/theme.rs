use ratatui::style::{Color, Modifier, Style};
use std::sync::RwLock;

// Private palette constants (Tokyo Night) — used by Theme::default() const fn.
const BG_MAIN: Color = Color::Rgb(0x1a, 0x1b, 0x26);
const BG_DARK: Color = Color::Rgb(0x16, 0x16, 0x1e);
const BG_HIGHLIGHT: Color = Color::Rgb(0x24, 0x28, 0x3b);
const BORDER: Color = Color::Rgb(0x41, 0x48, 0x68);
const BORDER_FOCUS: Color = Color::Rgb(0x7a, 0xa2, 0xf7);
const BORDER_HOVER: Color = Color::Rgb(0x56, 0x5f, 0x89);
const TEXT_MAIN: Color = Color::Rgb(0xc0, 0xca, 0xf5);
const TEXT_DIM: Color = Color::Rgb(0x56, 0x5f, 0x89);
const TEXT_DARK: Color = Color::Rgb(0x34, 0x3b, 0x58);
const ACCENT: Color = Color::Rgb(0x7a, 0xa2, 0xf7);
const SUCCESS: Color = Color::Rgb(0x9e, 0xce, 0x6a);
const WARNING: Color = Color::Rgb(0xe0, 0xaf, 0x68);
const ERROR: Color = Color::Rgb(0xf7, 0x76, 0x8e);
const PURPLE: Color = Color::Rgb(0xbb, 0x9a, 0xf7);
const CYAN: Color = Color::Rgb(0x7d, 0xcf, 0xff);

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub bg_main: Color,
    pub bg_dark: Color,
    pub bg_highlight: Color,
    pub border: Color,
    pub border_focus: Color,
    pub border_hover: Color,
    pub text_main: Color,
    pub text_dim: Color,
    pub text_dark: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub purple: Color,
    pub cyan: Color,
}

impl Theme {
    pub const fn default() -> Self {
        Self {
            bg_main: BG_MAIN,
            bg_dark: BG_DARK,
            bg_highlight: BG_HIGHLIGHT,
            border: BORDER,
            border_focus: BORDER_FOCUS,
            border_hover: BORDER_HOVER,
            text_main: TEXT_MAIN,
            text_dim: TEXT_DIM,
            text_dark: TEXT_DARK,
            accent: ACCENT,
            success: SUCCESS,
            warning: WARNING,
            error: ERROR,
            purple: PURPLE,
            cyan: CYAN,
        }
    }

    pub const fn gruvbox() -> Self {
        Self {
            bg_main: Color::Rgb(0x28, 0x28, 0x28),
            bg_dark: Color::Rgb(0x1d, 0x20, 0x21),
            bg_highlight: Color::Rgb(0x3c, 0x38, 0x36),
            border: Color::Rgb(0x66, 0x5c, 0x54),
            border_focus: Color::Rgb(0x83, 0xa5, 0x98),
            border_hover: Color::Rgb(0x7c, 0x6f, 0x64),
            text_main: Color::Rgb(0xeb, 0xdb, 0xb2),
            text_dim: Color::Rgb(0xa8, 0x99, 0x84),
            text_dark: Color::Rgb(0x92, 0x83, 0x74),
            accent: Color::Rgb(0xfa, 0xbd, 0x2f),
            success: Color::Rgb(0xb8, 0xbb, 0x26),
            warning: Color::Rgb(0xfe, 0x80, 0x19),
            error: Color::Rgb(0xfb, 0x49, 0x34),
            purple: Color::Rgb(0xd3, 0x86, 0x9e),
            cyan: Color::Rgb(0x8e, 0xc0, 0x7c),
        }
    }

    pub const fn catppuccin() -> Self {
        Self {
            bg_main: Color::Rgb(0x1e, 0x1e, 0x2e),
            bg_dark: Color::Rgb(0x18, 0x18, 0x25),
            bg_highlight: Color::Rgb(0x31, 0x32, 0x44),
            border: Color::Rgb(0x45, 0x47, 0x5a),
            border_focus: Color::Rgb(0x89, 0xb4, 0xfa),
            border_hover: Color::Rgb(0x58, 0x5b, 0x70),
            text_main: Color::Rgb(0xcd, 0xd6, 0xf4),
            text_dim: Color::Rgb(0x7f, 0x84, 0x9c),
            text_dark: Color::Rgb(0x6c, 0x70, 0x86),
            accent: Color::Rgb(0xcb, 0xa6, 0xf7),
            success: Color::Rgb(0xa6, 0xe3, 0xa1),
            warning: Color::Rgb(0xf9, 0xe2, 0xaf),
            error: Color::Rgb(0xf3, 0x8b, 0xa8),
            purple: Color::Rgb(0xc6, 0xa0, 0xf6),
            cyan: Color::Rgb(0x94, 0xe2, 0xd5),
        }
    }

    pub const fn nord() -> Self {
        Self {
            bg_main: Color::Rgb(0x2e, 0x34, 0x40),
            bg_dark: Color::Rgb(0x29, 0x2e, 0x39),
            bg_highlight: Color::Rgb(0x3b, 0x42, 0x52),
            border: Color::Rgb(0x4c, 0x56, 0x6a),
            border_focus: Color::Rgb(0x88, 0xc0, 0xd0),
            border_hover: Color::Rgb(0x61, 0x6e, 0x88),
            text_main: Color::Rgb(0xec, 0xef, 0xf4),
            text_dim: Color::Rgb(0x81, 0x8e, 0xa4),
            text_dark: Color::Rgb(0x6f, 0x7c, 0x96),
            accent: Color::Rgb(0x81, 0xa1, 0xc1),
            success: Color::Rgb(0xa3, 0xbe, 0x8c),
            warning: Color::Rgb(0xeb, 0xcb, 0x8b),
            error: Color::Rgb(0xbf, 0x61, 0x6a),
            purple: Color::Rgb(0xb4, 0x8e, 0xad),
            cyan: Color::Rgb(0x88, 0xc0, 0xd0),
        }
    }
}

static THEME: RwLock<Theme> = RwLock::new(Theme::default());

#[allow(clippy::expect_used)]
pub fn current() -> Theme {
    *THEME.read().expect("theme lock poisoned")
}

#[allow(clippy::expect_used)]
pub fn replace(theme: Theme) {
    *THEME.write().expect("theme lock poisoned") = theme;
}

pub fn for_name(name: &str) -> Option<Theme> {
    match name {
        "Tokyo Night" => Some(Theme::default()),
        "Gruvbox" => Some(Theme::gruvbox()),
        "Catppuccin" => Some(Theme::catppuccin()),
        "Nord" => Some(Theme::nord()),
        _ => None,
    }
}

// Color accessors
pub fn bg_main() -> Color { current().bg_main }
pub fn bg_dark() -> Color { current().bg_dark }
pub fn bg_highlight() -> Color { current().bg_highlight }
pub fn border() -> Color { current().border }
pub fn border_focus() -> Color { current().border_focus }
pub fn border_hover() -> Color { current().border_hover }
pub fn text_main() -> Color { current().text_main }
pub fn text_dim() -> Color { current().text_dim }
pub fn text_dark() -> Color { current().text_dark }
pub fn accent() -> Color { current().accent }
pub fn success() -> Color { current().success }
pub fn warning() -> Color { current().warning }
pub fn error() -> Color { current().error }
pub fn purple() -> Color { current().purple }
pub fn cyan() -> Color { current().cyan }

// Style accessors
pub fn border_style() -> Style { Style::new().fg(border()) }
pub fn border_focus_style() -> Style { Style::new().fg(border_focus()) }
pub fn border_hover_style() -> Style { Style::new().fg(border_hover()) }
pub fn active_style() -> Style { Style::new().fg(accent()).add_modifier(Modifier::BOLD) }
pub fn dim_style() -> Style { Style::new().fg(text_dim()) }
pub fn success_style() -> Style { Style::new().fg(success()) }
pub fn warning_style() -> Style { Style::new().fg(warning()) }
pub fn error_style() -> Style { Style::new().fg(error()) }
pub fn title_style() -> Style { Style::new().fg(purple()).add_modifier(Modifier::BOLD) }
pub fn header_style() -> Style { Style::new().bg(bg_dark()).fg(text_main()).add_modifier(Modifier::BOLD) }
