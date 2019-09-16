//! A pretty-printing library, using a peephole-efficient variant of
//! Jean-Philippe Bernardy's [Pretty But Not Greedy Printer](https://jyp.github.io/pdf/Prettiest.pdf) (ICFP'17).
//!
//! To pretty-print, you need:
//!
//! 1. A document that implements [`PrettyDocument`], and
//! 2. Something that implements [`PrettyWindow`], to print it to.

#![feature(slice_patterns)]
#![feature(box_patterns)]
#![feature(step_trait)]
#![feature(non_exhaustive)]

#[macro_use]
use utility;

mod geometry;
mod layout;
mod notation;
mod pane;
mod pretty;
mod style;

pub use self::geometry::{Bound, Col, Pos, Rect, Region, Row, MAX_WIDTH};
pub use self::layout::Bounds;
pub use self::notation::{
    child, empty, if_empty_text, if_flat, left, literal, nest, repeat, right, surrounded, text,
    vert, Notation, RepeatInner,
};

//pub use self::pretty::{DocPosSpec, PlainText, PrettyDocument, PrettyWindow};
pub use self::style::{Color, ColorTheme, Emph, Rgb, Shade, Style};
//pub use pane::{CursorVis, DocLabel, Pane, PaneError, PaneNotation, PaneSize};
