use std::fmt;

use super::pos::{Col, Pos, Row};
use super::range::Range;

/// A rectangle, either on the screen, or on the document.
/// Includes its upper-left, but excludes its lower-right.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Rect {
    pub rows: Range<Row>,
    pub cols: Range<Col>,
}

impl Rect {
    /// Create a new Rect with the given position and size
    pub fn new(top_left: Pos, size: Pos) -> Rect {
        Rect {
            rows: Range(top_left.row, top_left.row + size.row),
            cols: Range(top_left.col, top_left.col + size.col),
        }
    }

    /// Get the top left corner of the rectangle
    pub fn pos(&self) -> Pos {
        Pos {
            row: self.rows.0,
            col: self.cols.0,
        }
    }

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

    /// Return the intersection of the two rectangles, or None if they don't
    /// intersect.
    pub fn intersect(&self, other: Rect) -> Option<Rect> {
        let rows = self.rows.intersect(other.rows)?;
        let cols = self.cols.intersect(other.cols)?;
        Some(Rect { rows, cols })
    }

    /// Transform a point to the coordinate system given by this rectangle.
    pub fn transform(&self, pos: Pos) -> Option<Pos> {
        match (self.cols.transform(pos.col), self.rows.transform(pos.row)) {
            (Some(col), Some(row)) => Some(Pos { col, row }),
            (_, _) => None,
        }
    }

    /// Return an iterator over every position within this rectangle.
    pub fn positions(&self) -> impl Iterator<Item = Pos> {
        let mut v = Vec::new();
        for r in self.rows {
            for c in self.cols {
                v.push(Pos { row: r, col: c });
            }
        }
        v.into_iter()
    }

    /// Get the number of columns in the rectangle
    pub fn width(&self) -> Col {
        self.cols.len()
    }

    /// Get the number of rows in the rectangle
    pub fn height(&self) -> Row {
        self.rows.len()
    }

    /// Get the size of the rectangle
    pub fn size(&self) -> Pos {
        Pos {
            row: self.rows.len(),
            col: self.cols.len(),
        }
    }
    /// True if either the width or height is zero.
    pub fn is_empty(&self) -> bool {
        self.rows.len() == 0 || self.cols.len() == 0
    }

    /// Given N `widths`, returns an iterator over N sub-rectangles with those
    /// widths, in order from left to right. `.next()` will panic if the next
    /// width is larger than the width of the remaining rectangle.
    pub fn horz_splits<'a>(&self, widths: &'a [Col]) -> impl Iterator<Item = Rect> + 'a {
        let rows = self.rows;
        self.cols.splits(widths).map(move |col_range| Rect {
            cols: col_range,
            rows,
        })
    }

    /// Given N `heights`, returns an iterator over N sub-rectangles with those
    /// heights, in order from top to bottom. `.next()` will panic if the next
    /// height is greater than the height of the remaining rectangle.
    pub fn vert_splits<'a>(&self, heights: &'a [Row]) -> impl Iterator<Item = Rect> + 'a {
        let cols = self.cols;
        self.rows.splits(heights).map(move |row_range| Rect {
            rows: row_range,
            cols,
        })
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

    #[test]
    fn test_rect_pos_iter() {
        let positions: Vec<_> = RECT.positions().map(|pos| (pos.row, pos.col)).collect();
        assert_eq!(
            positions,
            vec![
                (2, 1),
                (2, 2),
                (2, 3),
                (2, 4),
                (3, 1),
                (3, 2),
                (3, 3),
                (3, 4),
            ]
        );
    }

    #[test]
    fn test_split_horz1() {
        let mut it = RECT.horz_splits(&[1, 3]);
        assert_eq!(
            it.next(),
            Some(Rect {
                cols: Range(1, 2),
                rows: Range(2, 4),
            })
        );
        assert_eq!(
            it.next(),
            Some(Rect {
                cols: Range(2, 5),
                rows: Range(2, 4),
            })
        );
        assert_eq!(it.next(), None)
    }

    #[test]
    fn test_split_horz2() {
        let mut it = RECT.horz_splits(&[0, 1, 0, 1, 0, 1, 0, 1, 0]);
        assert_eq!(
            it.next(),
            Some(Rect {
                cols: Range(1, 1),
                rows: Range(2, 4),
            })
        );
        assert_eq!(
            it.next(),
            Some(Rect {
                cols: Range(1, 2),
                rows: Range(2, 4),
            })
        );
        assert_eq!(
            it.next(),
            Some(Rect {
                cols: Range(2, 2),
                rows: Range(2, 4),
            })
        );
        assert_eq!(
            it.next(),
            Some(Rect {
                cols: Range(2, 3),
                rows: Range(2, 4),
            })
        );
        assert_eq!(
            it.next(),
            Some(Rect {
                cols: Range(3, 3),
                rows: Range(2, 4),
            })
        );
        assert_eq!(
            it.next(),
            Some(Rect {
                cols: Range(3, 4),
                rows: Range(2, 4),
            })
        );
        assert_eq!(
            it.next(),
            Some(Rect {
                cols: Range(4, 4),
                rows: Range(2, 4),
            })
        );
        assert_eq!(
            it.next(),
            Some(Rect {
                cols: Range(4, 5),
                rows: Range(2, 4),
            })
        );
        assert_eq!(
            it.next(),
            Some(Rect {
                cols: Range(5, 5),
                rows: Range(2, 4),
            })
        );
        assert_eq!(it.next(), None)
    }

    #[test]
    #[should_panic]
    fn test_split_horz3() {
        let mut it = RECT.horz_splits(&[5, 1]);
        it.next();
    }

    #[test]
    #[should_panic]
    fn test_split_horz4() {
        let mut it = RECT.horz_splits(&[1, 5]);
        it.next();
        it.next();
    }

    #[test]
    fn test_split_horz5() {
        // It's ok to leave some leftover width
        let mut it = RECT.horz_splits(&[1, 1]);
        assert_eq!(
            it.next(),
            Some(Rect {
                cols: Range(1, 2),
                rows: Range(2, 4),
            })
        );
        assert_eq!(
            it.next(),
            Some(Rect {
                cols: Range(2, 3),
                rows: Range(2, 4),
            })
        );
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_split_vert1() {
        let mut it = BIG.vert_splits(&[1, 2]);
        assert_eq!(
            it.next(),
            Some(Rect {
                rows: Range(1, 2),
                cols: Range(1, 5),
            })
        );
        assert_eq!(
            it.next(),
            Some(Rect {
                rows: Range(2, 4),
                cols: Range(1, 5),
            })
        );
        assert_eq!(it.next(), None)
    }

    #[test]
    fn test_split_vert2() {
        let mut it = BIG.vert_splits(&[0, 1, 0, 1, 0, 1, 0]);
        assert_eq!(
            it.next(),
            Some(Rect {
                rows: Range(1, 1),
                cols: Range(1, 5),
            })
        );
        assert_eq!(
            it.next(),
            Some(Rect {
                rows: Range(1, 2),
                cols: Range(1, 5),
            })
        );
        assert_eq!(
            it.next(),
            Some(Rect {
                rows: Range(2, 2),
                cols: Range(1, 5),
            })
        );
        assert_eq!(
            it.next(),
            Some(Rect {
                rows: Range(2, 3),
                cols: Range(1, 5),
            })
        );
        assert_eq!(
            it.next(),
            Some(Rect {
                rows: Range(3, 3),
                cols: Range(1, 5),
            })
        );
        assert_eq!(
            it.next(),
            Some(Rect {
                rows: Range(3, 4),
                cols: Range(1, 5),
            })
        );
        assert_eq!(
            it.next(),
            Some(Rect {
                rows: Range(4, 4),
                cols: Range(1, 5),
            })
        );
        assert_eq!(it.next(), None)
    }

    #[test]
    #[should_panic]
    fn test_split_vert3() {
        let mut it = BIG.vert_splits(&[4, 1]);
        it.next();
    }

    #[test]
    #[should_panic]
    fn test_split_vert4() {
        let mut it = BIG.vert_splits(&[1, 4]);
        it.next();
        it.next();
    }

    #[test]
    fn test_split_vert5() {
        // It's ok to leave some leftover height
        let mut it = BIG.vert_splits(&[1, 1]);
        assert_eq!(
            it.next(),
            Some(Rect {
                rows: Range(1, 2),
                cols: Range(1, 5),
            })
        );
        assert_eq!(
            it.next(),
            Some(Rect {
                rows: Range(2, 3),
                cols: Range(1, 5),
            })
        );
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_intersect_rect() {
        // Covered:
        assert_eq!(RECT.intersect(BIG), Some(RECT));

        // Overlapping:
        assert_eq!(
            TINY.intersect(SQUARE),
            Some(Rect {
                rows: Range(1, 2),
                cols: Range(2, 3),
            })
        );

        // Too far right:
        assert_eq!(
            RECT.intersect(Rect {
                rows: Range(5, 6),
                cols: Range(2, 6),
            }),
            None
        );
        // Sharing an edge:
        assert_eq!(
            RECT.intersect(Rect {
                rows: Range(4, 6),
                cols: Range(2, 6),
            }),
            None
        );
        // Sharing two edges:
        assert_eq!(
            RECT.intersect(Rect {
                rows: Range(4, 6),
                cols: Range(0, 1),
            }),
            None
        );
        // Too far below:
        assert_eq!(
            RECT.intersect(Rect {
                rows: Range(2, 3),
                cols: Range(6, 7),
            }),
            None
        );
    }
}
