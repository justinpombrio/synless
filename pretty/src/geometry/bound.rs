use std::fmt;

use super::pos::{Col, Row, MAX_WIDTH};
use super::rect::Rect;

/// A "paragraph" shape: it is like a rectangle, except that the last
/// line may be shorter than the rest.
///
/// Every node in the document fits within some Bound.
///
/// <pre>
///       width
/// |<-------------->|
///
/// +----------------+   -
/// |                |   ^
/// |                |   | height
/// |                |   |
/// |      +---------+   ∨
/// +------+             -
///
/// |<---->|
///  indent
/// </pre>
///
/// Valid bounds must have height at least 1, and if their height is 1 then
/// their width and indent must be equal.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Bound {
    pub width: Col,
    pub indent: Col,
    pub height: Row,
}

impl Bound {
    /// Create a new `Bound` with the given rectangular shape.
    pub fn new_rectangle(num_rows: Row, num_cols: Col) -> Bound {
        Bound {
            width: num_cols,
            height: num_rows,
            indent: num_cols,
        }
    }

    /// Create a new `Bound` with the given rectangular shape.
    pub fn from_rect(rect: Rect) -> Bound {
        Bound::new_rectangle(rect.height(), rect.width())
    }

    /// One Bound dominates another if it is at least as small in all
    /// dimensions.
    pub fn dominates(self, other: Bound) -> bool {
        // self wins ties
        (self.width <= other.width)
            && (self.height <= other.height)
            && (self.indent <= other.indent)
    }

    /// Is this Bound wider than MAX_WIDTH?
    /// Anything wider than MAX_WIDTH will simply be ignored: no one
    /// needs more than MAX_WIDTH characters on a line.
    pub fn too_wide(self) -> bool {
        self.width > MAX_WIDTH
    }

    /// A Bound that has the given width and is "infinitely" tall.
    pub fn infinite_scroll(width: Col) -> Bound {
        Bound {
            width,
            indent: width,
            // leave wiggle-room to avoid overflowing
            height: Row::max_value() - 1,
        }
    }

    /// Return true iff this bound is shaped like a rectangle.
    pub fn is_rectangular(self) -> bool {
        self.indent == self.width
    }

    pub(crate) fn debug_print(self, f: &mut fmt::Formatter, ch: char, indent: Col) -> fmt::Result {
        if self.height > 30 {
            return write!(f, "[very large bound]");
        }
        for _ in 0..(self.height - 1) {
            write!(f, "{}", ch.to_string().repeat(self.width as usize))?;
            writeln!(f)?;
            write!(f, "{}", " ".repeat(indent as usize))?;
        }
        write!(f, "{}", ch.to_string().repeat(self.indent as usize))
    }
}

impl fmt::Debug for Bound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.debug_print(f, '*', 0)
    }
}
