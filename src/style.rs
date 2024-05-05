use crate::util::SynlessBug;
use partial_pretty_printer as ppp;
use serde::{Deserialize, Serialize};

pub const HOLE_STYLE: Style = Style {
    is_hole: true,
    fg_color: Some((Base16Color::Base0F, Priority::High)),
    bold: Some((true, Priority::High)),
    ..Style::const_default()
};

pub const LEFT_CURSOR_STYLE: Style = Style {
    cursor: Some(CursorHalf::Left),
    bg_color: Some((Base16Color::Base02, Priority::High)),
    ..Style::const_default()
};

pub const RIGHT_CURSOR_STYLE: Style = Style {
    cursor: Some(CursorHalf::Right),
    bg_color: Some((Base16Color::Base00, Priority::High)),
    ..Style::const_default()
};

pub const FG_COLOR: Base16Color = Base16Color::Base05;
// NOTE: we might want to use Base00 as the default background, to follow the base16 conventions.
pub const BG_COLOR: Base16Color = Base16Color::Base01;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConcreteStyle {
    pub fg_color: Rgb,
    pub bg_color: Rgb,
    pub bold: bool,
    pub underlined: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Style {
    pub fg_color: Option<(Base16Color, Priority)>,
    pub bg_color: Option<(Base16Color, Priority)>,
    pub bold: Option<(bool, Priority)>,
    pub underlined: Option<(bool, Priority)>,
    pub cursor: Option<CursorHalf>,
    pub is_hole: bool,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Priority {
    #[default]
    Low,
    High,
}

#[derive(Debug, Clone, Copy)]
pub enum CursorHalf {
    Left,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum StyleLabel {
    Open,
    Close,
    Hole,
    Properties {
        #[serde(default)]
        fg_color: Option<Base16Color>,
        #[serde(default)]
        bg_color: Option<Base16Color>,
        #[serde(default)]
        bold: Option<bool>,
        #[serde(default)]
        underlined: Option<bool>,
        #[serde(default)]
        priority: Priority,
    },
}

// TODO: doc
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Condition {
    IsEmptyText,
    IsCommentOrWs,
    NeedsSeparator,
}

pub type Notation = ppp::Notation<StyleLabel, Condition>;
pub type ValidNotation = ppp::ValidNotation<StyleLabel, Condition>;

/// A 24-bit RGB color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Base16Color {
    /// Default Background
    Base00,
    /// Lighter Background (Used for status bars)
    Base01,
    /// Selection Background
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

/// A color theme that maps [Base16](https://github.com/chriskempson/base16) color names to RGB
/// color values.
#[derive(Debug, Clone)]
#[allow(non_snake_case)]
pub struct ColorTheme {
    /// Default Background
    pub base00: Rgb,
    /// Lighter Background (Used for status bars)
    pub base01: Rgb,
    /// Selection Background
    pub base02: Rgb,
    /// Comments, Invisibles, Line Highlighting
    pub base03: Rgb,
    /// Dark Foreground (Used for status bars)
    pub base04: Rgb,
    /// Default Foreground, Caret, Delimiters, Operators
    pub base05: Rgb,
    /// Light Foreground (Not often used)
    pub base06: Rgb,
    /// Light Background (Not often used)
    pub base07: Rgb,
    /// Variables, XML Tags, Markup Link Text, Markup Lists, Diff Deleted
    pub base08: Rgb,
    /// Integers, Boolean, Constants, XML Attributes, Markup Link Url
    pub base09: Rgb,
    /// Classes, Markup Bold, Search Text Background
    pub base0A: Rgb,
    /// Strings, Inherited Class, Markup Code, Diff Inserted
    pub base0B: Rgb,
    /// Support, Regular Expressions, Escape Characters, Markup Quotes
    pub base0C: Rgb,
    /// Functions, Methods, Attribute IDs, Headings
    pub base0D: Rgb,
    /// Keywords, Storage, Selector, Markup Italic, Diff Changed
    pub base0E: Rgb,
    /// Deprecated, Opening/Closing Embedded Language Tags, e.g. <?php ?>
    pub base0F: Rgb,
}

fn prioritize<T>(
    outer: Option<(T, Priority)>,
    inner: Option<(T, Priority)>,
) -> Option<(T, Priority)> {
    match (outer.as_ref().map(|x| x.1), inner.as_ref().map(|x| x.1)) {
        (None, _) => inner,
        (_, None) => outer,
        (Some(Priority::Low), Some(_)) => inner,
        (Some(Priority::High), Some(Priority::Low)) => outer,
        (Some(Priority::High), Some(Priority::High)) => inner,
    }
}

impl ppp::Style for Style {
    fn combine(outer: &Self, inner: &Self) -> Self {
        Style {
            cursor: outer.cursor.or(inner.cursor),
            fg_color: prioritize(outer.fg_color, inner.fg_color),
            bg_color: prioritize(outer.bg_color, inner.bg_color),
            bold: prioritize(outer.bold, inner.bold),
            underlined: prioritize(outer.underlined, inner.underlined),
            is_hole: outer.is_hole || inner.is_hole,
        }
    }
}

impl Style {
    const fn const_default() -> Style {
        Style {
            fg_color: None,
            bg_color: None,
            bold: None,
            underlined: None,
            cursor: None,
            is_hole: false,
        }
    }

    pub fn with_fg(mut self, color: Base16Color, priority: Priority) -> Style {
        self.fg_color = Some((color, priority));
        self
    }

    pub fn with_bg(mut self, color: Base16Color, priority: Priority) -> Style {
        self.bg_color = Some((color, priority));
        self
    }

    pub fn with_bold(mut self, bold: bool, priority: Priority) -> Style {
        self.bold = Some((bold, priority));
        self
    }

    pub fn with_underlined(mut self, underlined: bool, priority: Priority) -> Style {
        self.underlined = Some((underlined, priority));
        self
    }
}

impl ColorTheme {
    /// The "default dark" Base16 colorscheme, by Chris Kempson (http://chriskempson.com)
    pub fn default_dark() -> ColorTheme {
        ColorTheme {
            base00: Rgb::from_hex("#103030").bug(),
            base01: Rgb::from_hex("#111111").bug(),
            base02: Rgb::from_hex("#312121").bug(),
            base03: Rgb::from_hex("#585858").bug(),
            base04: Rgb::from_hex("#b8b8b8").bug(),
            base05: Rgb::from_hex("#d8d8d8").bug(),
            base06: Rgb::from_hex("#e8e8e8").bug(),
            base07: Rgb::from_hex("#f8f8f8").bug(),
            base08: Rgb::from_hex("#ab4642").bug(),
            base09: Rgb::from_hex("#dc9656").bug(),
            base0A: Rgb::from_hex("#f7ca88").bug(),
            base0B: Rgb::from_hex("#a1b56c").bug(),
            base0C: Rgb::from_hex("#86c1b9").bug(),
            base0D: Rgb::from_hex("#7cafc2").bug(),
            base0E: Rgb::from_hex("#ba8baf").bug(),
            base0F: Rgb::from_hex("#a16946").bug(),
        }
    }

    pub fn concrete_style(&self, style: &Style) -> ConcreteStyle {
        let unwrap_color = |pair: Option<(Base16Color, Priority)>, default: Base16Color| {
            self.color(pair.map(|(base16, _)| base16).unwrap_or(default))
        };
        let unwrap_bool = |pair: Option<(bool, Priority)>| pair.map(|(b, _)| b).unwrap_or_default();

        ConcreteStyle {
            fg_color: unwrap_color(style.fg_color, FG_COLOR),
            bg_color: unwrap_color(style.bg_color, BG_COLOR),
            bold: unwrap_bool(style.bold),
            underlined: unwrap_bool(style.underlined),
        }
    }

    pub fn color(&self, color: Base16Color) -> Rgb {
        match color {
            Base16Color::Base00 => self.base00,
            Base16Color::Base01 => self.base01,
            Base16Color::Base02 => self.base02,
            Base16Color::Base03 => self.base03,
            Base16Color::Base04 => self.base04,
            Base16Color::Base05 => self.base05,
            Base16Color::Base06 => self.base06,
            Base16Color::Base07 => self.base07,
            Base16Color::Base08 => self.base08,
            Base16Color::Base09 => self.base09,
            Base16Color::Base0A => self.base0A,
            Base16Color::Base0B => self.base0B,
            Base16Color::Base0C => self.base0C,
            Base16Color::Base0D => self.base0D,
            Base16Color::Base0E => self.base0E,
            Base16Color::Base0F => self.base0F,
        }
    }
}

impl Rgb {
    /// Construct an Rgb color from a string of the form "#FFFFFF".
    fn from_hex(hex_color: &str) -> Option<Rgb> {
        let to_int = |inclusive_range: (usize, usize)| -> Option<u8> {
            u8::from_str_radix(hex_color.get(inclusive_range.0..=inclusive_range.1)?, 16).ok()
        };

        Some(Rgb {
            red: to_int((1, 2))?,
            green: to_int((3, 4))?,
            blue: to_int((5, 6))?,
        })
    }
}
