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
///
/// All colors are given as [terminal 256 colors](https://en.wikipedia.org/wiki/ANSI_escape_code).
pub struct ColorTheme {
    pub white:   u16,
    pub red:     u16,
    pub yellow:  u16,
    pub green:   u16,
    pub cyan:    u16,
    pub blue:    u16,
    pub magenta: u16,
    pub shade0:  u16,
    pub shade1:  u16,
    pub shade2:  u16,
    pub shade3:  u16,
    pub cursor:  u16
}

impl ColorTheme {
    /// A simple color theme, with colors uniformly distributed on a circle in color space.
    ///
    /// See (Colorful Dodecagon)[http://justinpombrio.net/random/terminal-colors.html].
    pub fn colorful_hexagon() -> ColorTheme {
        ColorTheme{
            white  : 254,
            red    : 210,
            yellow : 179,
            green  : 114,
            cyan   : 44,
            blue   : 111,
            magenta: 176,
            shade0 : 239,
            shade1 : 235,
            shade2 : 232,
            shade3 : 232,
            cursor : 255
        }
    }

    fn color(&self, color: Color) -> u16 {
        match color {
            White   => self.white,
            Red     => self.red,
            Yellow  => self.yellow,
            Green   => self.green,
            Cyan    => self.cyan,
            Blue    => self.blue,
            Magenta => self.magenta
        }
    }

    /// The background color for a given shade, in this color theme, as a terminal256-color.
    pub fn shade(&self, shade: Shade) -> u16 {
        match shade.0 {
            0 => self.shade0,
            1 => self.shade1,
            2 => self.shade2,
            3 => self.shade3,
            _ => self.shade3
        }
    }

    /// The foreground color for a given style, in this color theme, as a terminal256-color.
    pub fn foreground(&self, style: Style) -> u16 {
        if style.reversed {
            self.shade(style.shade)
        } else {
            self.color(style.color)
        }
    }

    /// The background color for a given style, in this color theme, as a terminal256-color.
    pub fn background(&self, style: Style) -> u16 {
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
