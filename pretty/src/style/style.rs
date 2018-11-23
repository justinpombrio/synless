//! Styles for text to be rendered to the terminal.

use self::Color::*;


/// The overall style to render text to the terminal.
/// If `reversed`, swap the foreground and background.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Style {
    pub color:    Color,
    pub emph:     Emph,
    pub shade:    Shade,
    pub reversed: bool
}

// TODO: I do not know how widespread terminal support for underlining is.
/// Bold, underlined, or both?
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Emph {
    pub bold: bool,
    pub underlined: bool
}

/// The foreground color of some text (or if reversed the background color).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    White,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan
}

/// How dark the background is, or if reversed how dark the foreground is.
///
/// Only 0, 1, and 2+ are distinguished (subject to change).
/// 0 is brightest (most highlighted), and 2+ is black (least highlighted).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shade(pub usize);

impl Emph {
    /// Neither bold nor underlined.
    pub fn plain() -> Emph {
        Emph{
            underlined: false,
            bold: false
        }
    }

    /// Just underlined.
    pub fn underlined() -> Emph {
        Emph{
            underlined: true,
            bold: false
        }
    }
}

impl Style {
    /// Ordinary white on black.
    pub fn plain() -> Style {
        Style::new(White, Emph::plain(), Shade::black(), false)
    }

    /// Ordinary colored text.
    pub fn color(color: Color) -> Style {
        Style::new(color, Emph::plain(), Shade::black(), false)
    }

    /// Color the background. Visually very strong!
    pub fn reverse_color(color: Color) -> Style {
        Style::new(color, Emph::plain(), Shade::black(), true)
    }

    /// Fully customized style.
    pub fn new(color: Color, emph: Emph, shade: Shade, reversed: bool) -> Style {
        Style{
            color: color,
            emph: emph,
            shade: shade,
            reversed: reversed
        }
    }
}

impl Shade {
    /// Pure black, the most ordinary shade.
    pub fn black() -> Shade {
        Shade(5)
    }
}
