use std::fmt;

use common::{Row, Col, Pos, MAX_WIDTH};



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
/// |                |   ∨
/// |      +---------+   -
/// +------+
///
/// |<---->|
///  indent
/// </pre>
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Bound {
    pub width: Col,
    pub indent: Col,
    pub height: Row
}

impl Bound {
    /// Find the sub-bound that is within this bound, but below and
    /// to the right of `pos`.
    ///
    /// # Panics
    ///
    /// Panics if `pos` is not contained within this bound.
    pub fn subbound_from(&self, pos: Pos) -> Bound {
        if self.indent >= pos.col {
            Bound{
                width:  self.width  - pos.col,
                height: self.height - pos.row,
                indent: self.indent - pos.col
            }
        } else {
            Bound{
                width:  self.width  - pos.col,
                height: self.height - pos.row - 1,
                indent: self.width  - pos.col
            }
        }
    }

    /// Find the sub-bound that is within this bound, but ends at `pos`.
    ///
    /// # Panics
    ///
    /// Panics if `pos` is not contained within this bound.
    pub fn subbound_to(&self, pos: Pos) -> Bound {
        Bound{
            width:  self.width,
            height: pos.row,
            indent: pos.col
        }
    }

    /// One Bound dominates another if it is at least as small in all
    /// dimensions.
    pub fn dominates(&self, other: Bound) -> bool {
        // self wins ties
        (self.width <= other.width) &&
            (self.height <= other.height) &&
            (self.indent <= other.indent)
    }

    /// Is this Bound wider than MAX_WIDTH?
    /// Anything wider than MAX_WIDTH will simply be ignored: no one
    /// needs more than MAX_WIDTH characters on a line.
    pub fn too_wide(&self) -> bool {
        self.width > MAX_WIDTH
    }

    /// A  Bound that has the given width and is "infinitely" tall.
    pub fn infinite_scroll(width: Col) -> Bound {
        Bound{
            width:  width,
            indent: width,
            height: Row::max_value()
        }
    }

    fn debug_print(&self, f: &mut fmt::Formatter, ch: char, indent: Col)
                              -> fmt::Result
    {
        if self.height > 30 {
            return write!(f, "[very large bound]")
        }
        for _ in 0..self.height {
            write!(f, "{}", ch.to_string().repeat(self.width as usize))?;
            write!(f, "\n")?;
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
