use std::fmt;

use super::pos::{Col, Pos, Row};
use super::range::Range;

/// A rectangle, either on the screen, or on the document.
/// Includes its upper-left, but excludes its lower-right.
/// (Used in this file only, to simplify the implementation of Regions.)
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Rect {
    pub rows: Range<Row>,
    pub cols: Range<Col>,
}

impl Rect {
    /// Does this rectangle partially overlap the other rectangle?
    pub fn overlaps(&self, other: Rect) -> bool {
        self.cols.overlaps(other.cols) && self.rows.overlaps(other.rows)
    }

    /// Does this rectangle completely cover the other rectangle?
    pub fn covers(&self, other: Rect) -> bool {
        self.cols.covers(other.cols) && self.rows.covers(other.rows)
    }

    /// Does the Pos lie on this rectangle?
    pub fn contains(&self, pos: Pos) -> bool {
        self.cols.contains(pos.col) && self.rows.contains(pos.row)
    }

    /// Transform a point to the coordinate system given by this rectangle.
    pub fn transform(&self, pos: Pos) -> Option<Pos> {
        match (self.cols.transform(pos.col), self.rows.transform(pos.row)) {
            (Some(col), Some(row)) => Some(Pos { col: col, row: row }),
            (_, _) => None,
        }
    }
}

impl fmt::Display for Rect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}:{}-{}:{}",
            self.rows.0, self.cols.0, self.rows.1, self.cols.1
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static RECT: Rect = Rect {
        rows: Range(2, 4),
        cols: Range(1, 5),
    };

    static BIG: Rect = Rect {
        rows: Range(1, 4),
        cols: Range(1, 5),
    };

    static TINY: Rect = Rect {
        rows: Range(1, 2),
        cols: Range(1, 3),
    };

    static SQUARE: Rect = Rect {
        rows: Range(1, 4),
        cols: Range(2, 5),
    };

    #[test]
    fn test_rect_basics() {
        assert_eq!(&format!("{}", RECT), "2:1-4:5");
    }

    #[test]
    fn test_rect_contains() {
        assert_eq!(RECT.contains(Pos { row: 0, col: 0 }), false);
        assert_eq!(RECT.contains(Pos { row: 1, col: 1 }), false);
        assert_eq!(RECT.contains(Pos { row: 2, col: 0 }), false);
        assert_eq!(RECT.contains(Pos { row: 2, col: 1 }), true);
        assert_eq!(RECT.contains(Pos { row: 2, col: 2 }), true);
        assert_eq!(RECT.contains(Pos { row: 2, col: 3 }), true);
        assert_eq!(RECT.contains(Pos { row: 2, col: 4 }), true);
        assert_eq!(RECT.contains(Pos { row: 3, col: 1 }), true);
        assert_eq!(RECT.contains(Pos { row: 3, col: 2 }), true);
        assert_eq!(RECT.contains(Pos { row: 3, col: 3 }), true);
        assert_eq!(RECT.contains(Pos { row: 3, col: 4 }), true);
        assert_eq!(RECT.contains(Pos { row: 4, col: 4 }), false);
        assert_eq!(RECT.contains(Pos { row: 3, col: 5 }), false);
    }

    #[test]
    fn test_rect_transform() {
        assert_eq!(
            RECT.transform(Pos { row: 2, col: 4 }),
            Some(Pos { row: 0, col: 3 })
        );
        assert_eq!(RECT.transform(Pos { row: 2, col: 5 }), None);
        assert_eq!(
            RECT.transform(Pos { row: 2, col: 1 }),
            Some(Pos { row: 0, col: 0 })
        );
        assert_eq!(RECT.transform(Pos { row: 1, col: 1 }), None);
        assert_eq!(RECT.transform(Pos { row: 2, col: 0 }), None);
        assert_eq!(
            RECT.transform(Pos { row: 3, col: 4 }),
            Some(Pos { row: 1, col: 3 })
        );
        assert_eq!(RECT.transform(Pos { row: 4, col: 4 }), None);
        assert_eq!(RECT.transform(Pos { row: 3, col: 5 }), None);
    }

    #[test]
    fn test_rect_overlaps() {
        assert_eq!(RECT.overlaps(BIG), true);
        assert_eq!(RECT.overlaps(TINY), false);
        assert_eq!(RECT.overlaps(SQUARE), true);
        assert_eq!(BIG.overlaps(RECT), true);
        assert_eq!(BIG.overlaps(TINY), true);
        assert_eq!(BIG.overlaps(SQUARE), true);
        assert_eq!(TINY.overlaps(RECT), false);
        assert_eq!(TINY.overlaps(BIG), true);
        assert_eq!(TINY.overlaps(SQUARE), true);
        assert_eq!(SQUARE.overlaps(RECT), true);
        assert_eq!(SQUARE.overlaps(BIG), true);
        assert_eq!(SQUARE.overlaps(TINY), true);
    }

    #[test]
    fn test_rect_covers() {
        assert_eq!(RECT.covers(RECT), true);
        assert_eq!(RECT.covers(BIG), false);
        assert_eq!(RECT.covers(TINY), false);
        assert_eq!(RECT.covers(SQUARE), false);
        assert_eq!(BIG.covers(RECT), true);
        assert_eq!(BIG.covers(TINY), true);
        assert_eq!(BIG.covers(SQUARE), true);
        assert_eq!(TINY.covers(RECT), false);
        assert_eq!(TINY.covers(BIG), false);
        assert_eq!(TINY.covers(SQUARE), false);
        assert_eq!(SQUARE.covers(RECT), false);
        assert_eq!(SQUARE.covers(BIG), false);
        assert_eq!(SQUARE.covers(TINY), false);
    }
}
