use rustbox;
use rustbox::{RB_NORMAL, RB_BOLD, RB_UNDERLINE};

use style::style::*;
use style::style::Color::*;


/// A color theme.
///
/// The colors are nominally the six standard terminal colors (plus
/// white), but just like terminal colors they don't actually need to
/// match their name. (For example, all colors could be shades of
/// green or blue, though this could cause readability to suffer.)
///
/// The shades are used to shade the background of ancestors of the
/// selected node (by default in dark gray).
///
/// `cursor` is the color 
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

    fn color(&self, color: Color) -> rustbox::Color {
        let terminal_256_color = match color {
            White   => self.white,
            Red     => self.red,
            Yellow  => self.yellow,
            Green   => self.green,
            Cyan    => self.cyan,
            Blue    => self.blue,
            Magenta => self.magenta
        };
        rustbox::Color::Byte(terminal_256_color)
    }

    pub(crate) fn shade(&self, shade: Shade) -> rustbox::Color {
        let terminal_256_color = match shade.0 {
            0 => self.shade0,
            1 => self.shade1,
            2 => self.shade2,
            3 => self.shade3,
            _ => self.shade3
        };
        rustbox::Color::Byte(terminal_256_color)
    }

    pub(crate) fn foreground(&self, style: Style) -> rustbox::Color {
        if style.reversed {
            self.shade(style.shade)
        } else {
            self.color(style.color)
        }
    }

    pub(crate) fn background(&self, style: Style) -> rustbox::Color {
        if style.reversed {
            self.color(style.color)
        } else {
            self.shade(style.shade)
        }
    }

    pub(crate) fn emph(&self, style: Style) -> rustbox::Style {
        let ul = if style.emph.underline { RB_UNDERLINE } else { RB_NORMAL };
        let bd = if style.emph.bold { RB_BOLD } else { RB_NORMAL };
        ul | bd
    }
}
