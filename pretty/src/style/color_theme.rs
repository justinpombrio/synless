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
pub struct ColorTheme {
    pub white: Rgb,
    pub red: Rgb,
    pub yellow: Rgb,
    pub green: Rgb,
    pub cyan: Rgb,
    pub blue: Rgb,
    pub magenta: Rgb,
    pub shade0: Rgb,
    pub shade1: Rgb,
    pub shade2: Rgb,
    pub shade3: Rgb,
    pub cursor: Rgb,
}

/// A 24-bit RGB color.
#[derive(Clone, Copy)]
pub struct Rgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl ColorTheme {
    /// A simple color theme, with colors uniformly distributed on a circle in color space.
    ///
    /// See (Colorful Dodecagon)[http://justinpombrio.net/random/terminal-colors.html].
    pub fn colorful_hexagon() -> ColorTheme {
        ColorTheme {
            white: Rgb::from_hex("#e2e2e2"),
            red: Rgb::from_hex("#fc9c93"),
            yellow: Rgb::from_hex("#cdb36b"),
            green: Rgb::from_hex("#7ac68f"),
            cyan: Rgb::from_hex("#01c8d9"),
            blue: Rgb::from_hex("#80b9fe"),
            magenta: Rgb::from_hex("#e49fdb"),
            shade0: Rgb::from_hex("#4e4e4e"),
            shade1: Rgb::from_hex("#262626"),
            shade2: Rgb::from_hex("#080808"),
            shade3: Rgb::from_hex("#080808"),
            cursor: Rgb::from_hex("#eeeeee"),
        }
    }

    fn color(&self, color: Color) -> Rgb {
        match color {
            White => self.white,
            Red => self.red,
            Yellow => self.yellow,
            Green => self.green,
            Cyan => self.cyan,
            Blue => self.blue,
            Magenta => self.magenta,
        }
    }

    /// The background color for a given shade, in this color theme, as a terminal256-color.
    pub fn shade(&self, shade: Shade) -> Rgb {
        match shade.0 {
            0 => self.shade0,
            1 => self.shade1,
            2 => self.shade2,
            3 => self.shade3,
            _ => self.shade3,
        }
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
    // TODO return a result instead of panicking
    fn from_hex(hex_color: &str) -> Rgb {
        let to_int = |inclusive_range: (usize, usize)| {
            u8::from_str_radix(
                hex_color
                    .get(inclusive_range.0..=inclusive_range.1)
                    .expect("invalid hex color string: wrong length"),
                16,
            )
            .expect("invalid hex color string: not an int")
        };

        Rgb {
            red: to_int((1, 2)),
            green: to_int((3, 4)),
            blue: to_int((5, 6)),
        }
    }
}
