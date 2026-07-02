use ratatui::style::{Color, Modifier, Style};

pub const BG_MAIN: Color = Color::Rgb(0x1a, 0x1b, 0x26);
pub const BG_DARK: Color = Color::Rgb(0x16, 0x16, 0x1e);
pub const BG_HIGHLIGHT: Color = Color::Rgb(0x24, 0x28, 0x3b);
pub const BORDER: Color = Color::Rgb(0x41, 0x48, 0x68);
pub const BORDER_FOCUS: Color = Color::Rgb(0x7a, 0xa2, 0xf7);
pub const TEXT_MAIN: Color = Color::Rgb(0xc0, 0xca, 0xf5);
pub const TEXT_DIM: Color = Color::Rgb(0x56, 0x5f, 0x89);
pub const TEXT_DARK: Color = Color::Rgb(0x34, 0x3b, 0x58);
pub const ACCENT: Color = Color::Rgb(0x7a, 0xa2, 0xf7);
pub const SUCCESS: Color = Color::Rgb(0x9e, 0xce, 0x6a);
pub const WARNING: Color = Color::Rgb(0xe0, 0xaf, 0x68);
pub const ERROR: Color = Color::Rgb(0xf7, 0x76, 0x8e);
pub const PURPLE: Color = Color::Rgb(0xbb, 0x9a, 0xf7);
pub const CYAN: Color = Color::Rgb(0x7d, 0xcf, 0xff);

pub const HEADER_STYLE: Style = Style::new()
    .bg(BG_DARK)
    .fg(TEXT_MAIN)
    .add_modifier(Modifier::BOLD);

pub const BORDER_STYLE: Style = Style::new().fg(BORDER);

pub const BORDER_FOCUS_STYLE: Style = Style::new().fg(BORDER_FOCUS);

pub const ACTIVE_STYLE: Style = Style::new()
    .fg(ACCENT)
    .add_modifier(Modifier::BOLD);

pub const DIM_STYLE: Style = Style::new().fg(TEXT_DIM);

pub const SUCCESS_STYLE: Style = Style::new().fg(SUCCESS);

pub const WARNING_STYLE: Style = Style::new().fg(WARNING);

pub const ERROR_STYLE: Style = Style::new().fg(ERROR);

pub const TITLE_STYLE: Style = Style::new()
    .fg(PURPLE)
    .add_modifier(Modifier::BOLD);

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub bg_main: Color,
    pub bg_dark: Color,
    pub bg_highlight: Color,
    pub border: Color,
    pub border_focus: Color,
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
}

pub const THEME: Theme = Theme::default();
