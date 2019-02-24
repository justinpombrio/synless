//! Notations describe how to display a language.

#![feature(slice_patterns)]
#![feature(box_patterns)]

mod geometry;
mod layout;
mod notation;
mod pretty;
mod style;

pub use self::geometry::{Bound, Col, Pos, Region, Row, MAX_WIDTH};
pub use self::layout::Bounds;
pub use self::notation::{
    child, choice, concat, empty, horz, if_empty_text, literal, no_wrap, repeat, star, text, vert,
    Notation, Repeat,
};
pub use self::pretty::{PlainText, PrettyDocument, PrettyScreen};
pub use self::style::{Color, ColorTheme, Rgb, Style};
