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

mod style;
mod geometry;
mod notation;
mod layout;
mod pretty;
pub mod examples;

pub use self::style::{Style, ColorTheme, Color, Rgb, Emph, Shade};
pub use self::geometry::{Row, Col, Pos, Bound, Region, MAX_WIDTH};
pub use self::notation::{Notation, Repeat,
                         empty, literal, text, no_wrap, horz, vert, concat,
                         child, repeat, if_empty_text, choice};
pub use self::layout::Bounds;
pub use self::pretty::{PrettyScreen, PrettyDocument, PlainText};


