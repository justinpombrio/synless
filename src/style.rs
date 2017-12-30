//! Styles for text to be rendered to the terminal.

use rustbox;
use rustbox::{RB_NORMAL, RB_BOLD, RB_UNDERLINE};
use self::Color::*;
use self::Emph::*;


/// The overall style to render text to the terminal.
/// If `reversed`, swap the foreground and background.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Style {
    pub color:    Color,
    pub emph:     Emph,
    pub shade:    Shade,
    pub reversed: bool
}

/// Normal, bold, or underlined. Your terminal may not support
/// underlining.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Emph {
    Normal,
    Bold,
    Underline
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

impl Style {
    /// Ordinary white on black.
    pub fn plain() -> Style {
        Style::new(White, Normal, Shade::black(), false)
    }

    /// Ordinary colored text.
    pub fn color(color: Color) -> Style {
        Style::new(color, Normal, Shade(0), false)
    }

    /// Color the background. Visually very strong!
    pub fn reverse_color(color: Color) -> Style {
        Style::new(color, Normal, Shade::black(), true)
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
        match emph {
            Normal    => RB_NORMAL,
            Bold      => RB_BOLD,
            Underline => RB_UNDERLINE
        }
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

