use std::fmt;
use std::ops::Add;

use super::bound::Bound;
use super::pos::{Col, Pos, Row};
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
    pub fn new_rectangle(pos: Pos, size: Pos) -> Region {
        Region {
            pos,
            bound: Bound::new_rectangle(size.row, size.col),
        }
    }

    /// The empty region at a particular location.
    pub fn empty_region(pos: Pos) -> Region {
        Region {
            pos,
            bound: Bound {
                width: 0,
                height: 1,
                indent: 0,
            },
        }
    }

    /// The region around a single character position.
    pub fn char_region(pos: Pos) -> Region {
        Region {
            pos,
            bound: Bound {
                width: 1,
                height: 1,
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

    /// Does this region partially overlap the given rectangle?
    pub fn overlaps_rect(&self, rect: Rect) -> bool {
        self.body().overlaps(rect) || self.last_line().overlaps(rect)
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

    /// Return the intersection between the region and the rectangle, or None if they don't intersect.
    pub fn crop(&self, frame: Rect) -> Option<Region> {
        match (
            self.body().intersect(frame),
            self.last_line().intersect(frame),
        ) {
            (None, None) => None,
            (Some(body), None) => Some(Region::from(body)),
            (None, Some(last_line)) => Some(Region::from(last_line)),
            (Some(body), Some(last_line)) => Some(Region {
                pos: body.pos(),
                bound: Bound {
                    width: body.width(),
                    height: body.height() + last_line.height(),
                    indent: last_line.width(),
                },
            }),
        }
    }

    /// Transform a point to the coordinate system given by this region.
    ///
    /// Returns `None` if the point is not contained in this region.
    pub fn transform(&self, pos: Pos) -> Option<Pos> {
        match self.body().transform(pos) {
            Some(pos) => Some(pos),
            None => match self.last_line().transform(pos) {
                Some(pos) => Some(Pos {
                    row: pos.row + self.bound.height - 1,
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
        self.pos + self.bound.end()
    }

    /// The difference between `end` and `beginning`.
    pub fn delta(&self) -> Pos {
        self.end() - self.beginning()
    }

    /// Return true iff this region is shaped like a rectangle.
    pub fn is_rectangular(&self) -> bool {
        self.bound.is_rectangular()
    }

    fn body(&self) -> Rect {
        let size = Pos {
            col: self.bound.width,
            row: self.bound.height - 1,
        };
        Rect::new(self.pos, size)
    }

    fn last_line(&self) -> Rect {
        // There should probably never be a region with height 0, since even empty regions have
        // height 1. If there was, this method would return a very wrong row range.
        assert!(self.bound.height != 0);
        let pos = Pos {
            col: self.pos.col,
            row: self.pos.row + self.bound.height - 1,
        };

        let size = Pos {
            col: self.bound.indent,
            row: 1,
        };
        Rect::new(pos, size)
    }

    /// Return an iterator over every position within this region.
    pub fn positions(&self) -> impl Iterator<Item = Pos> {
        self.body().positions().chain(self.last_line().positions())
    }

    fn negative_space(&self) -> Rect {
        let size = Pos {
            col: self.bound.width - self.bound.indent,
            row: 1,
        };
        Rect::new(self.end(), size)
    }

    fn bounding_box(&self) -> Rect {
        let size = Pos {
            col: self.bound.width,
            row: self.bound.height,
        };
        Rect::new(self.pos, size)
    }
}

impl From<Rect> for Region {
    fn from(rect: Rect) -> Region {
        Region::new_rectangle(rect.pos(), rect.size())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::range::Range;

    static REGION: Region = Region {
        pos: Pos { row: 2, col: 3 },
        bound: Bound {
            width: 4,
            height: 4,
            indent: 2,
        },
    };

    static REGION2: Region = Region {
        pos: Pos { row: 3, col: 4 },
        bound: Bound {
            width: 3,
            height: 3,
            indent: 2,
        },
    };

    // like region1 except for the indent
    static REGION3: Region = Region {
        pos: Pos { row: 3, col: 4 },
        bound: Bound {
            width: 3,
            height: 3,
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

    #[test]
    fn test_region_rectangular() {
        assert!(Region::char_region(Pos::zero()).is_rectangular());

        // Not sure if an empty region *should* be considered rectangular, but now we'll notice if it changes
        assert!(Region::empty_region(Pos::zero()).is_rectangular());

        let a = Region {
            pos: Pos::zero(),
            bound: Bound {
                width: 5,
                height: 2,
                indent: 5,
            },
        };
        assert!(a.is_rectangular());

        let b = Region {
            pos: Pos::zero(),
            bound: Bound {
                width: 5,
                height: 2,
                indent: 4,
            },
        };
        assert!(!b.is_rectangular());

        let c = Region {
            pos: Pos::zero(),
            bound: Bound::new_rectangle(4, 5),
        };
        assert!(c.is_rectangular());

        let d = Region::new_rectangle(Pos::zero(), Pos { row: 4, col: 5 });
        assert!(d.is_rectangular());
        assert_eq!(c, d);
    }

    #[test]
    fn test_region_pos_iter() {
        assert!(Region::empty_region(Pos::zero())
            .positions()
            .next()
            .is_none());

        assert_eq!(
            Region::char_region(Pos::zero())
                .positions()
                .map(|pos| (pos.row, pos.col))
                .collect::<Vec<_>>(),
            vec![(0, 0),]
        );

        let rect_region = Region {
            pos: Pos::zero(),
            bound: Bound {
                width: 3,
                height: 2,
                indent: 3,
            },
        };

        assert_eq!(
            rect_region
                .positions()
                .map(|pos| (pos.row, pos.col))
                .collect::<Vec<_>>(),
            vec![(0, 0), (0, 1), (0, 2), (1, 0), (1, 1), (1, 2),]
        );

        assert_eq!(
            REGION2
                .positions()
                .map(|pos| (pos.row, pos.col))
                .collect::<Vec<_>>(),
            vec![
                (3, 4),
                (3, 5),
                (3, 6),
                (4, 4),
                (4, 5),
                (4, 6),
                (5, 4),
                (5, 5),
            ]
        );
    }

    #[test]
    fn test_crop_region() {
        assert_eq!(REGION.crop(REGION.bounding_box()), Some(REGION));
        assert_eq!(
            REGION.crop(Rect {
                rows: Range(1, 7),
                cols: Range(2, 8)
            }),
            Some(REGION)
        );
        assert_eq!(
            REGION.crop(Rect {
                rows: Range(1, 4),
                cols: Range(2, 8)
            }),
            Some(Region {
                pos: Pos { row: 2, col: 3 },
                bound: Bound {
                    width: 4,
                    height: 2,
                    indent: 4,
                },
            })
        );
        assert_eq!(
            REGION.crop(Rect {
                rows: Range(3, 7),
                cols: Range(2, 8)
            }),
            Some(Region {
                pos: Pos { row: 3, col: 3 },
                bound: Bound {
                    width: 4,
                    height: 3,
                    indent: 2,
                },
            })
        );
        assert_eq!(
            REGION.crop(Rect {
                rows: Range(3, 6),
                cols: Range(3, 5)
            }),
            Some(Region {
                pos: Pos { row: 3, col: 3 },
                bound: Bound {
                    width: 2,
                    height: 3,
                    indent: 2,
                },
            })
        );
        assert_eq!(
            REGION.crop(Rect {
                rows: Range(2, 6),
                cols: Range(4, 6)
            }),
            Some(Region {
                pos: Pos { row: 2, col: 4 },
                bound: Bound {
                    width: 2,
                    height: 4,
                    indent: 1,
                },
            })
        );
        assert_eq!(
            REGION.crop(Rect {
                rows: Range(2, 6),
                cols: Range(5, 7)
            }),
            Some(Region {
                pos: Pos { row: 2, col: 5 },
                bound: Bound {
                    width: 2,
                    height: 3,
                    indent: 2,
                },
            })
        );
        assert_eq!(
            REGION.crop(Rect {
                rows: Range(5, 6),
                cols: Range(3, 5)
            }),
            Some(Region {
                pos: Pos { row: 5, col: 3 },
                bound: Bound {
                    width: 2,
                    height: 1,
                    indent: 2,
                },
            })
        );
    }
}
