//! Notations describe how to display a language.
//!
//! The full set of allowed notations is contained in the [`Notation`
//! enum](Notation).
//!
//! These notations obey a large set of algebraic laws. These laws are described
//! below, using `+` for [concatention](Notation::Concat), `^` for [vertical
//! concatenation](Notation::Vert), `|` for [choice](Notation::Choice), and `⊥`
//! for the impossible notation.
//!
//! ## Associativity
//!
//! ```code
//!     a + (b + c) = (a + b) + c
//!     a | (b | c) = (a | b) | c
//!     a ^ (b ^ c) = (a ^ b) ^ c
//! ```
//!
//! ## Distributivity
//!
//! ```code
//!     (a | b) + c = (a + c) | (b + c)
//!     a + (b | c) = (a + b) | (a + c)
//!    
//!     (a | b) ^ c = a ^ c | b ^ c
//!     a ^ (b | c) = a ^ b | a ^ c
//!
//!     no_wrap(a + b) = no_wrap(a) + no_wrap(b)
//!     no_wrap(a | b) = no_wrap(a) | no_wrap(b)
//! ```
//!
//! ## Identity
//!
//! ```code
//!     a + empty() = a
//!     empty() + a = a
//!
//!     ⊥ | a = a
//!     a | ⊥ = a
//! ```
//!
//! ## Idempotence
//!
//! ```code
//!     no_wrap(no_wrap(a)) = no_wrap(a)
//!     a | a = a
//! ```
//!
//! ## Annihilation
//!
//! ```code
//!     ⊥ + a = ⊥
//!     a + ⊥ = ⊥
//!     ⊥ ^ a = ⊥
//!     a ^ ⊥ = ⊥
//!     no_wrap(⊥) = ⊥
//! ```
//!
//! ## Absorbtion
//!
//! ```code
//!     a | (a + b) = a
//!     (a + b) | a = a
//!     a | (a ^ b) = a
//!     (a ^ b) | a = a
//!     a | no_wrap(a) = a
//!     no_wrap(a) | a = a
//! ```
//!
//! ## Misc
//!
//! ```code
//!     a ^ (b + c) = (a ^ b) + c
//!     no_wrap(a ^ b) = ⊥
//!     no_wrap(literal(s)) = literal(s)
//! ```

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
    child, choice, concat, empty, horz, if_empty_text, literal, no_wrap, repeat, text, vert,
    Notation, Repeat,
};
pub use self::pretty::{PlainText, PrettyDocument, PrettyScreen};
pub use self::style::{Color, ColorTheme, Emph, Rgb, Shade, Style};
