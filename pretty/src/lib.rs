//! A pretty-printing library, using a peephole-efficient variant of
//! Jean-Philippe Bernardy's [Pretty But Not Greedy Printer](https://jyp.github.io/pdf/Prettiest.pdf) (ICFP'17).
//!
//! To pretty-print, you need:
//!
//! 1. A document that implements [`PrettyDocument`], and
//! 2. A screen that implements [`PrettyScreen`].

#![feature(slice_patterns)]
#![feature(box_patterns)]
#![feature(step_trait)]

mod geometry;
mod layout;
mod notation;
mod pretty;
mod style;

pub use self::geometry::{Bound, Col, Pos, Region, Row, MAX_WIDTH};
pub use self::layout::Bounds;
pub use self::notation::{
    child, choice, concat, empty, horz, if_empty_text, literal, no_wrap, repeat, text, vert,
    Notation, Repeat,
};

pub use self::pretty::{PlainText, PrettyDocument, PrettyScreen};
pub use self::style::{Color, ColorTheme, Emph, Rgb, Shade, Style};
