//! This module defines coordinates used by the rest of the editor.
//! `Pos` represents points, `Range` represents lines, and `Rect` represents rectangles.

use std::ops::Add;
use std::ops::Sub;
use std::fmt;


/// Used to represent heights and row coordinates.
pub type Row = u32;
/// Used to represent widths and column coordinates.
pub type Col = u16;

/// 0-indexed positions, relative either to the screen or the document.
/// These are also sometimes used to represent offsets.
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Pos {
    pub col: Col,
    pub row: Row
}

impl Pos {
    /// The upper-left.
    pub fn zero() -> Pos {
        Pos{ col: 0, row: 0 }
    }
}

impl Add<Pos> for Pos {
    type Output = Pos;
    fn add(self, other: Pos) -> Pos {
        Pos{
            col: self.col + other.col,
            row: self.row + other.row
        }
    }
}

impl Sub<Pos> for Pos {
    type Output = Pos;
    fn sub(self, other: Pos) -> Pos {
        if self.col < other.col || self.row < other.row {
            panic!("Underflow while subtracting `Pos`s.");
        }
        Pos{
            col: self.col - other.col,
            row: self.row - other.row
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

/// A paragraph shape within which a node in the document fits.  Every
/// node fits within some Bound: it is a rectangle, except that the
/// last line may not extend all the way to the end.
///
/// <pre>
///    width
/// |<-------->|
/// +----------+  -
/// |          |  ^
/// |          |  | height
/// |          |  ∨
/// |      +---+  -
/// +------+
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
    /// to the right of `pos`. Pos must be contained in this bound.
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
    /// Pos must be contained in this bound.
    pub fn subbound_to(&self, pos: Pos) -> Bound {
        Bound{
            width:  self.width,
            height: pos.row,
            indent: pos.col
        }
    }
}



/// (Used in this file only, to simplify the implementation of Regions.)
/// A range of either rows or columns.
/// The start point is included, but not the end point, so
/// Range(2,4) means rows/columns 2&3.
/// Invariant: `Range(a,b)` implies `b-a>=1`.
#[derive(PartialEq, Eq, Clone, Copy)]
struct Range<N>(pub N, pub N);

impl<N> Range<N> where N : Add<Output=N>, N : Sub<Output=N>, N : Ord, N : Copy {

    fn overlaps(self, other: Range<N>) -> bool {
        !self.is_left_of(other) && !other.is_left_of(self)
    }

    fn contains(self, n: N) -> bool {
        self.0 <= n && n < self.1
    }

    fn covers(self, other: Range<N>) -> bool {
        self.0 <= other.0 && other.1 <= self.1
    }

    fn is_left_of(self, other: Range<N>) -> bool {
        self.1 <= other.0
    }

    fn transform(self, n: N) -> Option<N> {
        if self.contains(n) {
            Some(n - self.0)
        } else {
            None
        }
    }
}

impl<N> Add<N> for Range<N> where N : Add<N, Output=N>, N : Copy {
    type Output = Range<N>;
    fn add(self, n: N) -> Range<N> {
        Range(self.0 + n, self.1 + n)
    }
}

impl<N> fmt::Display for Range<N> where N : fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}", self.0, self.1)
    }
}

/// A rectangle, either on the screen, or on the document.
/// Includes its upper-left, but excludes its lower-right.
/// (Used in this file only, to simplify the implementation of Regions.)
#[derive(PartialEq, Eq, Clone, Copy)]
struct Rect {
    pub rows: Range<Row>,
    pub cols: Range<Col>
}

impl Rect {
    /// Does this rectangle partially overlap the other rectangle?
    fn overlaps(&self, other: Rect) -> bool {
        self.cols.overlaps(other.cols) && self.rows.overlaps(other.rows)
    }

    /// Does this rectangle completely cover the other rectangle?
    fn covers(&self, other: Rect) -> bool {
        self.cols.covers(other.cols) && self.rows.covers(other.rows)
    }

    /// Does the Pos lie on this rectangle?
    fn contains(&self, pos: Pos) -> bool {
        self.cols.contains(pos.col) && self.rows.contains(pos.row)
    }

    /// Transform a point to the coordinate system given by this rectangle.
    fn transform(&self, pos: Pos) -> Option<Pos> {
        match (self.cols.transform(pos.col), self.rows.transform(pos.row)) {
            (Some(col), Some(row)) => Some(Pos{ col: col, row: row }),
            (_, _)                 => None
        }
    }
}

impl fmt::Display for Rect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}-{}:{}",
               self.rows.0, self.cols.0,
               self.rows.1, self.cols.1)
        }
}


/// A region of the document. It has the same shape as a Bound,
/// but is positioned on the screen.
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Region {
    /// Upper left
    pub pos: Pos,
    pub bound: Bound
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{};{}", self.beginning(), self.end(), self.bound.width)
    }
}

impl fmt::Debug for Region {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Add<Pos> for Region {
    type Output = Region;
    fn add(self, pos: Pos) -> Region {
        Region{
            pos: self.pos + pos,
            bound: self.bound
        }
    }
}

impl Region {
    /// The region around a single character position.
    pub fn char_region(pos: Pos) -> Region {
        Region{
            pos: pos,
            bound: Bound{ width: 1, height: 0, indent: 1 }
        }
    }

    /// Does this region partially overlap the other region?
    pub fn overlaps(&self, other: Region) -> bool {
        self.body().overlaps(other.body())
            || self.body().overlaps(other.last_line())
            || self.last_line().overlaps(other.body())
            || self.last_line().overlaps(other.last_line())
    }

    /// Does this region completely cover the other region?
    pub fn covers(&self, other: Region) -> bool {
        self.bounding_box().covers(other.body())
            && self.bounding_box().covers(other.last_line())
            && !self.negative_space().overlaps(other.body())
            && !self.negative_space().overlaps(other.last_line())
    }

    /// Does the Pos lie on this region?
    pub fn contains(&self, pos: Pos) -> bool {
        self.body().contains(pos) || self.last_line().contains(pos)
    }

    /// Transform a point to the coordinate system given by this region.
    pub fn transform(&self, pos: Pos) -> Option<Pos> {
        match self.body().transform(pos) {
            Some(pos) => Some(pos),
            None => match self.last_line().transform(pos) {
                Some(pos) => Some(Pos{
                    row: pos.row + self.bound.height,
                    col: pos.col
                }),
                None => None
            }
        }
    }

    // TODO: untested
    /// Find the subregion that is within this region, but below and
    /// to the right of `pos`. Pos must be contained in this region.
    pub fn subregion_from(&self, pos: Pos) -> Region {
        let delta = pos - self.beginning();
        let bound = self.bound.subbound_from(delta);
        Region{
            pos:   pos,
            bound: bound
        }
    }

    pub fn width(&self) -> Col {
        self.bound.width
    }

    pub fn height(&self) -> Row {
        self.bound.height
    }

    pub fn indent(&self) -> Col {
        self.bound.indent
    }

    /// The upper-left character position (same as `.pos`)
    pub fn beginning(&self) -> Pos {
        self.pos
    }

    /// The character position just past the end of the region
    /// (just past right character of last line).
    pub fn end(&self) -> Pos {
        self.pos + Pos{ row: self.bound.height, col: self.bound.indent }
    }

    /// The difference between `end` and `beginning`.
    pub fn delta(&self) -> Pos {
        self.end() - self.beginning()
    }

    fn body(&self) -> Rect {
        Rect{
            cols: Range(self.pos.col, self.pos.col + self.bound.width),
            rows: Range(self.pos.row, self.pos.row + self.bound.height)
        }
    }

    fn last_line(&self) -> Rect {
        Rect{
            cols: Range(self.pos.col,
                        self.pos.col + self.bound.indent),
            rows: Range(self.pos.row + self.bound.height,
                        self.pos.row + self.bound.height + 1)
        }
    }

    fn negative_space(&self) -> Rect {
        Rect{
            cols: Range(self.pos.col + self.bound.indent,
                        self.pos.col + self.bound.width),
            rows: Range(self.pos.row + self.bound.height,
                        self.pos.row + self.bound.height + 1)
        }
    }

    fn bounding_box(&self) -> Rect {
        Rect{
            cols: Range(self.pos.col, self.pos.col + self.bound.width),
            rows: Range(self.pos.row, self.pos.row + self.bound.height + 1)
        }
    }
}

// Utility

/// Given two slices, find the length of their longest common prefix.
pub(crate) fn common_prefix_len<A : Eq>(p: &[A], q: &[A]) -> usize {
    match (p, q) {
        (&[ref p, ref ps..], &[ref q, ref qs..]) =>
            if p == q {
                1 + common_prefix_len(ps, qs)
            } else {
                0
            },
        _ => 0
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    static RECT: Rect = Rect{
        rows: Range(2, 4),
        cols: Range(1, 5)
    };

    static BIG: Rect = Rect{
        rows: Range(1, 4),
        cols: Range(1, 5)
    };

    static TINY: Rect = Rect{
        rows: Range(1, 2),
        cols: Range(1, 3)
    };

    static SQUARE: Rect = Rect{
        rows: Range(1, 4),
        cols: Range(2, 5)
    };

    #[test]
    fn test_rect_basics() {
        assert_eq!(&format!("{}", RECT),
                   "2:1-4:5");
    }

    #[test]
    fn test_rect_contains() {
        assert_eq!(RECT.contains(Pos{ row: 0, col: 0 }), false);
        assert_eq!(RECT.contains(Pos{ row: 1, col: 1 }), false);
        assert_eq!(RECT.contains(Pos{ row: 2, col: 0 }), false);
        assert_eq!(RECT.contains(Pos{ row: 2, col: 1 }), true);
        assert_eq!(RECT.contains(Pos{ row: 2, col: 2 }), true);
        assert_eq!(RECT.contains(Pos{ row: 2, col: 3 }), true);
        assert_eq!(RECT.contains(Pos{ row: 2, col: 4 }), true);
        assert_eq!(RECT.contains(Pos{ row: 3, col: 1 }), true);
        assert_eq!(RECT.contains(Pos{ row: 3, col: 2 }), true);
        assert_eq!(RECT.contains(Pos{ row: 3, col: 3 }), true);
        assert_eq!(RECT.contains(Pos{ row: 3, col: 4 }), true);
        assert_eq!(RECT.contains(Pos{ row: 4, col: 4 }), false);
        assert_eq!(RECT.contains(Pos{ row: 3, col: 5 }), false);
    }

    #[test]
    fn test_rect_transform() {
        assert_eq!(RECT.transform(Pos{ row: 2, col: 4 }),
                   Some(Pos{ row: 0, col: 3 }));
        assert_eq!(RECT.transform(Pos{ row: 2, col: 5 }),
                   None);
        assert_eq!(RECT.transform(Pos{ row: 2, col: 1 }),
                   Some(Pos{ row: 0, col: 0 }));
        assert_eq!(RECT.transform(Pos{ row: 1, col: 1 }), None);
        assert_eq!(RECT.transform(Pos{ row: 2, col: 0 }), None);
        assert_eq!(RECT.transform(Pos{ row: 3, col: 4 }),
                   Some(Pos{ row: 1, col: 3 }));
        assert_eq!(RECT.transform(Pos{ row: 4, col: 4 }), None);
        assert_eq!(RECT.transform(Pos{ row: 3, col: 5 }), None);
    }

    #[test]
    fn test_rect_overlaps() {
        assert_eq!(RECT.overlaps(BIG),    true);
        assert_eq!(RECT.overlaps(TINY),   false);
        assert_eq!(RECT.overlaps(SQUARE), true);
        assert_eq!(BIG.overlaps(RECT),    true);
        assert_eq!(BIG.overlaps(TINY),    true);
        assert_eq!(BIG.overlaps(SQUARE),  true);
        assert_eq!(TINY.overlaps(RECT),   false);
        assert_eq!(TINY.overlaps(BIG),    true);
        assert_eq!(TINY.overlaps(SQUARE), true);
        assert_eq!(SQUARE.overlaps(RECT), true);
        assert_eq!(SQUARE.overlaps(BIG),  true);
        assert_eq!(SQUARE.overlaps(TINY), true);
    }

    #[test]
    fn test_rect_covers() {
        assert_eq!(RECT.covers(RECT),   true);
        assert_eq!(RECT.covers(BIG),    false);
        assert_eq!(RECT.covers(TINY),   false);
        assert_eq!(RECT.covers(SQUARE), false);
        assert_eq!(BIG.covers(RECT),    true);
        assert_eq!(BIG.covers(TINY),    true);
        assert_eq!(BIG.covers(SQUARE),  true);
        assert_eq!(TINY.covers(RECT),   false);
        assert_eq!(TINY.covers(BIG),    false);
        assert_eq!(TINY.covers(SQUARE), false);
        assert_eq!(SQUARE.covers(RECT), false);
        assert_eq!(SQUARE.covers(BIG),  false);
        assert_eq!(SQUARE.covers(TINY), false);
    }

    static REGION: Region = Region{
        pos: Pos{ row: 2, col: 3},
        bound: Bound{ width: 4, height: 3, indent: 2 }
    };

    static REGION2: Region = Region{
        pos: Pos{ row: 3, col: 4},
        bound: Bound{ width: 3, height: 2, indent: 2 }
    };

    // like region1 except for the indent
    static REGION3: Region = Region{
        pos: Pos{ row: 3, col: 4},
        bound: Bound{ width: 3, height: 2, indent: 1 }
    };

    #[test]
    fn test_region_rects() {
        assert_eq!(format!("{}", REGION.body()),           "2:3-5:7");
        assert_eq!(format!("{}", REGION.last_line()),      "5:3-6:5");
        assert_eq!(format!("{}", REGION.bounding_box()),   "2:3-6:7");
        assert_eq!(format!("{}", REGION.negative_space()), "5:5-6:7");
    }

    #[test]
    fn test_region_transform() {
        assert_eq!(REGION.transform(Pos{ row: 1, col: 3 }),
                   None);
        assert_eq!(REGION.transform(Pos{ row: 2, col: 3 }),
                   Some(Pos{ row: 0, col: 0 }));
        assert_eq!(REGION.transform(Pos{ row: 2, col: 6 }),
                   Some(Pos{ row: 0, col: 3 }));
        assert_eq!(REGION.transform(Pos{ row: 2, col: 7 }),
                   None);
        assert_eq!(REGION.transform(Pos{ row: 3, col: 6 }),
                   Some(Pos{ row: 1, col: 3 }));
        assert_eq!(REGION.transform(Pos{ row: 5, col: 3 }),
                   Some(Pos{ row: 3, col: 0 }));
        assert_eq!(REGION.transform(Pos{ row: 6, col: 3 }),
                   None);
        assert_eq!(REGION.transform(Pos{ row: 4, col: 6 }),
                   Some(Pos{ row: 2, col: 3 }));
        assert_eq!(REGION.transform(Pos{ row: 5, col: 6 }),
                   None);
        assert_eq!(REGION.transform(Pos{ row: 4, col: 7 }),
                   None);
    }

    #[test]
    fn test_regions() {
        assert_eq!(REGION.overlaps(REGION2), true);
        assert_eq!(REGION.covers(REGION2), false);
        assert_eq!(REGION.covers(REGION3), true);
    }
}
