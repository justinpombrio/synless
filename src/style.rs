//! Styles for text to be rendered to the terminal.

use rustbox;
use rustbox::{RB_NORMAL, RB_BOLD, RB_UNDERLINE};
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

/// Bold, underlined, or both?
/// I do not know how widespread terminal support for underlining is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Emph {
    pub bold: bool,
    pub underline: bool
}

/// The foreground color (or if reversed the background color).
/// These are terminal colors. To change them, edit your terminal
/// color scheme.
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
/// Only 0, 1, and 2+ are distinguished (subject to change).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shade(pub usize);

impl Emph {
    /// Neither bold nor underlined.
    pub fn plain() -> Emph {
        Emph{
            underline: false,
            bold: false
        }
    }

    /// Just underlined.
    pub fn underlined() -> Emph {
        Emph{
            underline: true,
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
        Style::new(color, Emph::plain(), Shade(0), false)
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

    pub(crate) fn foreground(&self) -> rustbox::Color {
        if self.reversed {
            self.shade.into()
        } else {
            self.color.into()
        }
    }

    pub(crate) fn background(&self) -> rustbox::Color {
        if self.reversed {
            self.color.into()
        } else {
            self.shade.into()
        }
    }

    pub(crate) fn emphasis(&self) -> rustbox::Style {
        self.emph.into()
    }
}

impl Shade {
    pub fn black() -> Shade {
        Shade(5)
    }
}

impl From<Emph> for rustbox::Style {
    fn from(emph: Emph) -> rustbox::Style {
        let ul = if emph.underline { RB_UNDERLINE } else { RB_NORMAL };
        let bd = if emph.bold { RB_BOLD } else { RB_NORMAL };
        ul | bd
    }
}

impl From<Color> for rustbox::Color {
    fn from(color: Color) -> rustbox::Color {
        let terminal_256_color = match color {
            White   => 254,
            Red     => 210,
            Yellow  => 179,
            Green   => 114,
            Cyan    => 44,
            Blue    => 111,
            Magenta => 176
            /* Old colors:
            Red     => 218,
            Yellow  => 216,
            Green   => 150,
            Cyan    => 79,
            Blue    => 81,
            Magenta => 183,
            */
        };
        rustbox::Color::Byte(terminal_256_color)
    }
}

impl From<Shade> for rustbox::Color {
    fn from(shade: Shade) -> rustbox::Color {
        let Shade(shade) = shade;
        let terminal_256_color = match shade {
            0 => 239,
            1 => 235,
            _ => 232
        };
        rustbox::Color::Byte(terminal_256_color)
    }
}

