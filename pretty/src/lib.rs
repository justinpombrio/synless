//! Notations describe how to display a language.

#![feature(slice_patterns)]
#![feature(box_patterns)]

mod style;
mod geometry;
mod notation;
mod layout;
mod pretty;

pub use self::style::{Style, ColorTheme, Color, Rgb};
pub use self::geometry::{Row, Col, Pos, Bound, Region, MAX_WIDTH};
pub use self::notation::{Notation, Repeat,
                         empty, literal, text, no_wrap, horz, vert, concat,
                         child, repeat, if_empty_text, choice};
pub use self::layout::Bounds;
pub use self::pretty::{PrettyScreen, PrettyDocument, PlainText};


