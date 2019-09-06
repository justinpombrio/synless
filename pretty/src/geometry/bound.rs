use std::cmp;
use std::fmt;

use super::pos::{Col, Pos, Row, MAX_WIDTH};
use super::rect::Rect;
use crate::notation::NotationOps;

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
    // TODO: switch (height, width) -> (width, height), because that's standard.
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

    // TODO: can probably delete this now
    /// One Bound dominates another if it is at least as small in all
    /// dimensions.
    pub fn dominates(&self, other: Bound) -> bool {
        // self wins ties
        (self.width <= other.width)
            && (self.height <= other.height)
            && (self.indent <= other.indent)
    }

    /// Is this Bound wider than MAX_WIDTH?
    /// Anything wider than MAX_WIDTH will simply be ignored: no one
    /// needs more than MAX_WIDTH characters on a line.
    pub fn too_wide(&self) -> bool {
        self.width > MAX_WIDTH
    }

    // TODO: can probably delete this now
    /// A Bound that has the given width and is "infinitely" tall.
    pub fn infinite_scroll(width: Col) -> Bound {
        Bound {
            width: width,
            indent: width,
            // leave wiggle-room to avoid overflowing
            height: Row::max_value() - 1,
        }
    }

    /// Return true iff this bound is shaped like a rectangle.
    pub fn is_rectangular(&self) -> bool {
        self.indent == self.width
    }

    /// The character position just past the end of the bound.
    ///
    /// (That is: the character position just to the right of the last
    /// character of the last line of this bound.)
    pub fn end(&self) -> Pos {
        Pos {
            row: self.height - 1,
            col: self.indent,
        }
    }

    pub(crate) fn debug_print(&self, f: &mut fmt::Formatter, ch: char, indent: Col) -> fmt::Result {
        if self.height > 30 {
            return write!(f, "[very large bound]");
        }
        write!(f, "\n")?;
        for _ in 0..(self.height - 1) {
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

impl NotationOps for Bound {
    fn empty() -> Bound {
        Bound {
            width: 0,
            height: 1,
            indent: 0,
        }
    }

    fn literal(string: &str) -> Bound {
        let width = string.chars().count() as Col;
        Bound {
            width: width,
            indent: width,
            height: 1,
        }
    }

    fn nest(b1: Bound, b2: Bound) -> Bound {
        Bound {
            width: cmp::max(b1.width, b1.indent + b2.width),
            height: b1.height + b2.height - 1,
            indent: b1.indent + b2.indent,
        }
    }

    fn vert(b1: Bound, b2: Bound) -> Bound {
        Bound {
            width: cmp::max(b1.width, b2.width),
            height: b1.height + b2.height,
            indent: b2.indent,
        }
    }

    fn if_flat(bound1: Bound, bound2: Bound) -> Bound {
        if bound1.height > 1 {
            bound1.clone()
        } else {
            bound2.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    #[test]
    fn test_show_bound() {
        let r = Bound {
            width: 10,
            indent: 6,
            height: 3,
        };
        let mut s = String::new();
        write!(&mut s, "{:?}", r).unwrap();
        assert_eq!(
            s,
            "
**********
**********
******"
        );
    }

    #[test]
    fn test_show_empty_bound() {
        let r = Bound {
            width: 0,
            indent: 0,
            height: 1,
        };
        let mut s = String::new();
        write!(&mut s, "{:?}", r).unwrap();
        assert_eq!(s, "\n");
    }

    #[test]
    fn test_domination() {
        let r_best = Bound {
            width: 10,
            indent: 6,
            height: 3,
        };
        let r_1 = Bound {
            width: 11,
            indent: 6,
            height: 3,
        };
        let r_2 = Bound {
            width: 10,
            indent: 7,
            height: 3,
        };
        let r_3 = Bound {
            width: 10,
            indent: 6,
            height: 4,
        };
        let r_worst = Bound {
            width: 11,
            indent: 7,
            height: 4,
        };
        assert!(r_best.dominates(r_best));
        assert!(r_best.dominates(r_worst));
        assert!(r_best.dominates(r_1));
        assert!(r_best.dominates(r_2));
        assert!(r_best.dominates(r_3));
        assert!(r_1.dominates(r_worst));
        assert!(r_2.dominates(r_worst));
        assert!(r_3.dominates(r_worst));
        assert!(!r_1.dominates(r_2));
        assert!(!r_1.dominates(r_3));
        assert!(!r_2.dominates(r_1));
        assert!(!r_2.dominates(r_3));
        assert!(!r_3.dominates(r_1));
        assert!(!r_3.dominates(r_2));
    }
}
