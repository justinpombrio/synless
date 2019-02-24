//! This module defines coordinates and shapes used by the rest of the editor.

use std::fmt;
use std::ops::Add;
use std::ops::Sub;

/// Height, as measured in terminal characters.
pub type Row = u32;
/// Width, as measured in terminal characters.
pub type Col = u16;
/// Nothing ever needs to be wider than this.
pub const MAX_WIDTH: Col = 256;

/// A character position, typically relative to the screen or the document.
///
/// The origin is in the upper left, and is `(0, 0)`. I.e., this is 0-indexed.
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Pos {
    pub col: Col,
    pub row: Row,
}

impl Pos {
    /// The upper-left corner.
    pub fn zero() -> Pos {
        Pos { col: 0, row: 0 }
    }
}

impl Add<Pos> for Pos {
    type Output = Pos;
    fn add(self, other: Pos) -> Pos {
        Pos {
            col: self.col + other.col,
            row: self.row + other.row,
        }
    }
}

impl Sub<Pos> for Pos {
    type Output = Pos;
    fn sub(self, other: Pos) -> Pos {
        if self.col < other.col || self.row < other.row {
            panic!("Underflow while subtracting `Pos`s.");
        }
        Pos {
            col: self.col - other.col,
            row: self.row - other.row,
        }
    }
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.row, self.col)
    }
}
impl fmt::Debug for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

/*
// Stolen from rust-mustache
macro_rules! bug {
    ($msg:expr) => {
        bug!("{}", $msg);
    };
    ($fmt:expr, $($arg:tt)+) => {
        panic!(
            concat!("Bug: ",
                    $fmt,
                    " Please report this issue on Github."),
            $($arg)*);
    };
}
*/
