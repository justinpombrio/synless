//! Style choices, such as color, bolding, and underlining.

mod color_theme;

// TODO rename modules to fix this for real
#[allow(clippy::module_inception)]
mod style;

pub use self::color_theme::{ColorTheme, Rgb};
pub use self::style::*;
