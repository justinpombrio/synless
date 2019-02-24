use std::fmt;
use std::ops::Add;

use super::bound::Bound;
use super::pos::{Col, Pos, Row};
use super::range::Range;
use super::rect::Rect;

/// A region of the document. It has the same shape as a Bound,
/// but is positioned on the screen.
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Region {
    /// The position of the upper left corner
    pub pos: Pos,
    pub bound: Bound,
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}-{};{}",
            self.beginning(),
            self.end(),
            self.bound.width
        )
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
        Region {
            pos: self.pos + pos,
            bound: self.bound,
        }
    }
}

impl Region {
    /// The empty region at a particular location.
    pub fn empty_region(pos: Pos) -> Region {
        Region {
            pos: pos,
            bound: Bound {
                width: 0,
                height: 0,
                indent: 0,
            },
        }
    }

    /// The region around a single character position.
    pub fn char_region(pos: Pos) -> Region {
        Region {
            pos: pos,
            bound: Bound {
                width: 1,
                height: 0,
                indent: 1,
            },
        }
    }

    /// Does this region partially overlap the other region?
    pub fn overlaps(&self, other: Region) -> bool {
        self.body().overlaps(other.body())
            || self.body().overlaps(other.last_line())
            || self.last_line().overlaps(other.body())
            || self.last_line().overlaps(other.last_line())
    }

    pub fn covers(&self, other: Region) -> bool {
        self.bounding_box().covers(other.body())
            && self.bounding_box().covers(other.last_line())
            && !self.negative_space().overlaps(other.body())
            && !self.negative_space().overlaps(other.last_line())
    }

    /// Does `pos` lie on this region?
    pub fn contains(&self, pos: Pos) -> bool {
        self.body().contains(pos) || self.last_line().contains(pos)
    }

    /// Transform a point to the coordinate system given by this region.
    ///
    /// Returns `None` if the point is not contained in this region.
    pub fn transform(&self, pos: Pos) -> Option<Pos> {
        match self.body().transform(pos) {
            Some(pos) => Some(pos),
            None => match self.last_line().transform(pos) {
                Some(pos) => Some(Pos {
                    row: pos.row + self.bound.height,
                    col: pos.col,
                }),
                None => None,
            },
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

    /// The character position just past the end of the region.
    ///
    /// (That is: the character position just to the right of the last
    /// character of the last line of this region.)
    pub fn end(&self) -> Pos {
        self.pos
            + Pos {
                row: self.bound.height,
                col: self.bound.indent,
            }
    }

    /// The difference between `end` and `beginning`.
    pub fn delta(&self) -> Pos {
        self.end() - self.beginning()
    }

    fn body(&self) -> Rect {
        Rect {
            cols: Range(self.pos.col, self.pos.col + self.bound.width),
            rows: Range(self.pos.row, self.pos.row + self.bound.height),
        }
    }

    fn last_line(&self) -> Rect {
        Rect {
            cols: Range(self.pos.col, self.pos.col + self.bound.indent),
            rows: Range(
                self.pos.row + self.bound.height,
                self.pos.row + self.bound.height + 1,
            ),
        }
    }

    fn negative_space(&self) -> Rect {
        Rect {
            cols: Range(
                self.pos.col + self.bound.indent,
                self.pos.col + self.bound.width,
            ),
            rows: Range(
                self.pos.row + self.bound.height,
                self.pos.row + self.bound.height + 1,
            ),
        }
    }

    fn bounding_box(&self) -> Rect {
        Rect {
            cols: Range(self.pos.col, self.pos.col + self.bound.width),
            rows: Range(self.pos.row, self.pos.row + self.bound.height + 1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static REGION: Region = Region {
        pos: Pos { row: 2, col: 3 },
        bound: Bound {
            width: 4,
            height: 3,
            indent: 2,
        },
    };

    static REGION2: Region = Region {
        pos: Pos { row: 3, col: 4 },
        bound: Bound {
            width: 3,
            height: 2,
            indent: 2,
        },
    };

    // like region1 except for the indent
    static REGION3: Region = Region {
        pos: Pos { row: 3, col: 4 },
        bound: Bound {
            width: 3,
            height: 2,
            indent: 1,
        },
    };

    #[test]
    fn test_region_rects() {
        assert_eq!(format!("{}", REGION.body()), "2:3-5:7");
        assert_eq!(format!("{}", REGION.last_line()), "5:3-6:5");
        assert_eq!(format!("{}", REGION.bounding_box()), "2:3-6:7");
        assert_eq!(format!("{}", REGION.negative_space()), "5:5-6:7");
    }

    #[test]
    fn test_region_transform() {
        assert_eq!(REGION.transform(Pos { row: 1, col: 3 }), None);
        assert_eq!(
            REGION.transform(Pos { row: 2, col: 3 }),
            Some(Pos { row: 0, col: 0 })
        );
        assert_eq!(
            REGION.transform(Pos { row: 2, col: 6 }),
            Some(Pos { row: 0, col: 3 })
        );
        assert_eq!(REGION.transform(Pos { row: 2, col: 7 }), None);
        assert_eq!(
            REGION.transform(Pos { row: 3, col: 6 }),
            Some(Pos { row: 1, col: 3 })
        );
        assert_eq!(
            REGION.transform(Pos { row: 5, col: 3 }),
            Some(Pos { row: 3, col: 0 })
        );
        assert_eq!(REGION.transform(Pos { row: 6, col: 3 }), None);
        assert_eq!(
            REGION.transform(Pos { row: 4, col: 6 }),
            Some(Pos { row: 2, col: 3 })
        );
        assert_eq!(REGION.transform(Pos { row: 5, col: 6 }), None);
        assert_eq!(REGION.transform(Pos { row: 4, col: 7 }), None);
    }

    #[test]
    fn test_regions() {
        assert_eq!(REGION.overlaps(REGION2), true);
        assert_eq!(REGION.covers(REGION2), false);
        assert_eq!(REGION.covers(REGION3), true);
    }
}
