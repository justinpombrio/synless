//! Styles for text to be rendered to the terminal.

use self::Color::*;

/// The overall style to render text to the terminal.
/// If `reversed`, swap the foreground and background.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Style {
    pub color: Color,
    pub emph: Emph,
    pub shade: Shade,
    pub reversed: bool,
}

// TODO: I do not know how widespread terminal support for underlining is.
/// Bold, underlined, or both?
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Emph {
    pub bold: bool,
    pub underlined: bool,
}

/// The foreground color of some text (or if reversed the background color).
///
/// This uses the [Base16](http://chriskempson.com/projects/base16/) colortheme definitions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    /// Default Background
    Base00,
    /// Lighter Background (Used for status bars)
    Base01,
    /// Selection Background, was shade2
    Base02,
    /// Comments, Invisibles, Line Highlighting
    Base03,
    /// Dark Foreground (Used for status bars)
    Base04,
    /// Default Foreground, Caret, Delimiters, Operators
    Base05,
    /// Light Foreground (Not often used)
    Base06,
    /// Light Background (Not often used)
    Base07,
    /// Variables, XML Tags, Markup Link Text, Markup Lists, Diff Deleted
    Base08,
    /// Integers, Boolean, Constants, XML Attributes, Markup Link Url
    Base09,
    /// Classes, Markup Bold, Search Text Background
    Base0A,
    /// Strings, Inherited Class, Markup Code, Diff Inserted
    Base0B,
    /// Support, Regular Expressions, Escape Characters, Markup Quotes
    Base0C,
    /// Functions, Methods, Attribute IDs, Headings
    Base0D,
    /// Keywords, Storage, Selector, Markup Italic, Diff Changed
    Base0E,
    /// Deprecated, Opening/Closing Embedded Language Tags, e.g. <?php ?>
    Base0F,
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
        Emph {
            underlined: false,
            bold: false,
        }
    }

    /// Just underlined.
    pub fn underlined() -> Emph {
        Emph {
            underlined: true,
            bold: false,
        }
    }
}

impl Style {
    /// Typically, ordinary white on black.
    pub fn plain() -> Style {
        Style::new(Base05, Emph::plain(), Shade::background(), false)
    }

    /// Ordinary colored text.
    pub fn color(color: Color) -> Style {
        Style::new(color, Emph::plain(), Shade::background(), false)
    }

    /// Color the background. Visually very strong!
    pub fn reverse_color(color: Color) -> Style {
        Style::new(color, Emph::plain(), Shade::background(), true)
    }

    /// Fully customized style.
    pub fn new(color: Color, emph: Emph, shade: Shade, reversed: bool) -> Style {
        Style {
            color: color,
            emph: emph,
            shade: shade,
            reversed: reversed,
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Style::plain()
    }
}

impl Shade {
    /// Typically pure black, the most ordinary shade.
    pub fn background() -> Shade {
        Shade(usize::max_value())
    }
}

impl Default for Shade {
    fn default() -> Self {
        Shade::background()
    }
}

impl From<Shade> for Color {
    fn from(shade: Shade) -> Color {
        match shade {
            Shade(0) => Base03,
            Shade(1) => Base02,
            Shade(2) => Base01,
            _ => Base00,
        }
    }
}
