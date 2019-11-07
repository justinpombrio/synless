//! A pretty-printing library, using a peephole-efficient variant of
//! Jean-Philippe Bernardy's [Pretty But Not Greedy Printer](https://jyp.github.io/pdf/Prettiest.pdf) (ICFP'17).
//!
//! To pretty-print, you need:
//!
//! 1. A document that implements [`PrettyDocument`], and
//! 2. Something that implements [`PrettyWindow`], to print it to.

// TODO!
#![allow(dead_code)]
#![feature(box_patterns)]
#![feature(slice_patterns)]
#![feature(step_trait)]

mod geometry;
mod layout;
mod notation;
mod pane;
mod pretty;
mod style;

pub use self::geometry::{Bound, Col, Pos, Rect, Region, Row, MAX_WIDTH};
pub use self::notation::{Notation, RepeatInner};
pub use self::pretty::{DocPosSpec, PlainText, PrettyDocument, PrettyWindow};
pub use self::style::{Color, ColorTheme, Emph, Rgb, Shade, ShadedStyle, Style};
pub use pane::{CursorVis, DocLabel, Pane, PaneError, PaneNotation, PaneSize};
