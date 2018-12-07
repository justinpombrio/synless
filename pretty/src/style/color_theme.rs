use super::style::*;
use super::Color::*;

/// A color theme.
///
/// The colors are nominally the six standard terminal colors (plus
/// white), but just like terminal colors they don't actually need to
/// match their name. (For example, all colors could be shades of
/// green or blue.)
///
/// The shades are used to shade the background of ancestors of the
/// selected node (by default in dark gray). `shade0` is the strongest
/// (i.e., lightest) shade, and `shade3` is the weakest (i.e.,
/// darkest) shade, which is used for most of the background.
///
/// `cursor` is the color of the cursor.
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

/// A 24-bit RGB color.
#[derive(Clone, Copy)]
pub struct Rgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl ColorTheme {
    /// The "default dark" Base16 colorscheme, by Chris Kempson (http://chriskempson.com)
    pub fn default_dark() -> ColorTheme {
        ColorTheme {
            base00: Rgb::from_hex("#181818").unwrap(),
            base01: Rgb::from_hex("#282828").unwrap(),
            base02: Rgb::from_hex("#383838").unwrap(),
            base03: Rgb::from_hex("#585858").unwrap(),
            base04: Rgb::from_hex("#b8b8b8").unwrap(),
            base05: Rgb::from_hex("#d8d8d8").unwrap(),
            base06: Rgb::from_hex("#e8e8e8").unwrap(),
            base07: Rgb::from_hex("#f8f8f8").unwrap(),
            base08: Rgb::from_hex("#ab4642").unwrap(),
            base09: Rgb::from_hex("#dc9656").unwrap(),
            base0A: Rgb::from_hex("#f7ca88").unwrap(),
            base0B: Rgb::from_hex("#a1b56c").unwrap(),
            base0C: Rgb::from_hex("#86c1b9").unwrap(),
            base0D: Rgb::from_hex("#7cafc2").unwrap(),
            base0E: Rgb::from_hex("#ba8baf").unwrap(),
            base0F: Rgb::from_hex("#a16946").unwrap(),
        }
    }

    fn color(&self, color: Color) -> Rgb {
        match color {
            Base00 => self.base00,
            Base01 => self.base01,
            Base02 => self.base02,
            Base03 => self.base03,
            Base04 => self.base04,
            Base05 => self.base05,
            Base06 => self.base06,
            Base07 => self.base07,
            Base08 => self.base08,
            Base09 => self.base09,
            Base0A => self.base0A,
            Base0B => self.base0B,
            Base0C => self.base0C,
            Base0D => self.base0D,
            Base0E => self.base0E,
            Base0F => self.base0F,
        }
    }

    /// The background color for a given shade, in this color theme, as a terminal256-color.
    pub fn shade(&self, shade: Shade) -> Rgb {
         self.color(shade.into())
    }

    /// The foreground color for a given style, in this color theme, as a terminal256-color.
    pub fn foreground(&self, style: Style) -> Rgb {
        if style.reversed {
            self.shade(style.shade)
        } else {
            self.color(style.color)
        }
    }

    /// The background color for a given style, in this color theme, as a terminal256-color.
    pub fn background(&self, style: Style) -> Rgb {
        if style.reversed {
            self.color(style.color)
        } else {
            self.shade(style.shade)
        }
    }

    // TODO: This belongs in the Terminal frontend.
    /*
    pub fn emph(&self, style: Style) -> rustbox::Style {
        let ul = if style.emph.underlined { RB_UNDERLINE } else { RB_NORMAL };
        let bd = if style.emph.bold { RB_BOLD } else { RB_NORMAL };
        ul | bd
    }
     */
}

impl Rgb {
    /// Construct an Rgb color from a string of the form "#FFFFFF".
    fn from_hex(hex_color: &str) -> Option<Rgb> {
        let to_int = |inclusive_range: (usize, usize)| -> Option<u8> {
            u8::from_str_radix(
                hex_color
                     .get(inclusive_range.0 ..= inclusive_range.1)?,
                16,
            ).ok()
        };

        Some(Rgb {
            red: to_int((1, 2))?,
            green: to_int((3, 4))?,
            blue: to_int((5, 6))?,
        })
    }
}
