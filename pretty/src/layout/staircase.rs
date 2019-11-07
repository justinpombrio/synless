use crate::geometry::Bound;
use crate::geometry::{Col, Row};
use std::fmt::Debug;
use std::{iter, slice, vec};

/// Store a set of (width, height) pairs with the property that if one pair
/// _dominates another_ by having smaller width and height, then the dominated
/// pair need not be stored. This property allows the pairs to be stored in
/// sorted order, of simultaneously _increasing_ heights and _decreasing_
/// widths. This in turn allows for efficient (log-time) insertion and lookup.
///
/// Staircases are used by Boundsets, which stores one Staircase for each
/// indent. Each stair in the staircase also contains some related data T.
#[derive(Clone, Debug)]
pub struct Staircase<T> {
    indent: Col,                // indent
    stairs: Vec<(Col, Row, T)>, // (width, height, value)
}

impl<T: Clone + Debug> Staircase<T> {
    /// Construct an empty staircase.
    pub fn new(indent: Col) -> Staircase<T> {
        Staircase {
            indent,
            stairs: vec![],
        }
    }

    /// Get the indent shared by all Bounds in this Staircase.
    pub fn indent(&self) -> Col {
        self.indent
    }

    /// Insert a new bound into a staircase.
    pub fn insert(&mut self, width: Col, height: Row, value: T) {
        let (skip_left, skip_right, delete_left, delete_right) = self.indices(width, height);
        // If the new stair is already covered, skip it.
        if skip_left < skip_right {
            return;
        }
        // If the new stair covers existing stairs, delete them.
        if delete_left < delete_right {
            self.stairs.drain(delete_left..delete_right);
        }
        // Insert the new stair.
        self.stairs.insert(delete_left, (width, height, value));
    }

    /// Pick the best (i.e., shortest) Bound in the staircase that fits within
    /// the given Bound. Panics if none fit.
    pub fn fit_bound(&self, space: Bound) -> Option<(Bound, &T)> {
        let width = space.width;
        let height = if space.indent < self.indent {
            space.height + 1
        } else {
            space.height
        };
        let (skip_left, skip_right, _, _) = self.indices(width, height);
        if skip_left < skip_right {
            let (w, h, value) = &self.stairs[skip_left];
            let bound = Bound {
                width: *w,
                height: *h,
                indent: self.indent,
            };
            Some((bound, value))
        } else {
            None
        }
    }

    /// Pick the best (i.e., shortest) Bound in the staircase that fits within
    /// the given width.
    pub fn fit_width(&self, width: Col) -> Option<(Bound, &T)> {
        let width_index = self
            .stairs
            .binary_search_by_key(&-(width as isize), |(w, _, _)| -(*w as isize));
        let index = match width_index {
            Ok(i) => i,
            Err(i) => i,
        };
        match self.stairs.get(index) {
            None => None,
            Some((w, h, value)) => {
                let bound = Bound {
                    width: *w,
                    height: *h,
                    indent: self.indent,
                };
                Some((bound, value))
            }
        }
    }

    /// Does a bound in this staircase dominate the given bound?
    /// (I.e., is there a smaller bound in the staircase?)
    pub fn dominates(&self, width: Col, height: Row) -> bool {
        let (skip_left, skip_right, _, _) = self.indices(width, height);
        skip_left < skip_right
    }

    /// Delete all bounds in the staircase that a dominated by the given bound.
    /// (I.e., delete all stairs that are larger than it.)
    pub fn clear_dominated(&mut self, width: Col, height: Row) {
        let (_, _, delete_left, delete_right) = self.indices(width, height);
        if delete_left < delete_right {
            self.stairs.drain(delete_left..delete_right);
        }
    }

    /// Insert a bound without checking domination. Only use this if you you
    /// know it won't dominate or be dominated by any bound in the staircase.
    pub fn unchecked_insert(&mut self, width: Col, height: Row, value: T) {
        let (skip_left, _, _, _) = self.indices(width, height);
        self.stairs.insert(skip_left, (width, height, value));
    }

    fn indices(&self, width: Col, height: Row) -> (usize, usize, usize, usize) {
        let width_index = self
            .stairs
            .binary_search_by_key(&-(width as isize), |(w, _, _)| -(*w as isize));
        let height_index = self.stairs.binary_search_by_key(&height, |(_, h, _)| *h);
        let (skip_left, delete_right) = match width_index {
            Ok(i) => (i, i + 1),
            Err(i) => (i, i),
        };
        let (delete_left, skip_right) = match height_index {
            Ok(i) => (i, i + 1),
            Err(i) => (i, i),
        };
        (skip_left, skip_right, delete_left, delete_right)
    }
}

impl<'a, T: Clone + Debug> iter::IntoIterator for &'a Staircase<T> {
    type Item = (Bound, &'a T);
    type IntoIter = StaircaseIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        StaircaseIter {
            indent: self.indent,
            stairs: self.stairs.iter(),
        }
    }
}

pub struct StaircaseIter<'a, T: Clone + Debug> {
    indent: Col,
    stairs: slice::Iter<'a, (Col, Row, T)>,
}

impl<'a, T: Debug + Clone> Iterator for StaircaseIter<'a, T> {
    type Item = (Bound, &'a T);
    fn next(&mut self) -> Option<(Bound, &'a T)> {
        self.stairs.next().map(|(w, h, value)| {
            let bound = Bound {
                width: *w,
                height: *h,
                indent: self.indent,
            };
            (bound, value)
        })
    }
}

impl<T: Clone + Debug> iter::IntoIterator for Staircase<T> {
    type Item = (Bound, T);
    type IntoIter = StaircaseIntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        StaircaseIntoIter {
            indent: self.indent,
            stairs: self.stairs.into_iter(),
        }
    }
}

pub struct StaircaseIntoIter<T: Clone + Debug> {
    indent: Col,
    stairs: vec::IntoIter<(Col, Row, T)>,
}

impl<T: Debug + Clone> Iterator for StaircaseIntoIter<T> {
    type Item = (Bound, T);
    fn next(&mut self) -> Option<(Bound, T)> {
        self.stairs.next().map(|(w, h, value)| {
            let bound = Bound {
                width: w,
                height: h,
                indent: self.indent,
            };
            (bound, value)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn basic_stairs() -> Staircase<char> {
        let mut stairs = Staircase::new(0);
        stairs.insert(6, 2, 'a');
        stairs.insert(2, 6, 'a');
        stairs.insert(4, 4, 'a');
        stairs
    }

    #[test]
    fn test_dominates() {
        let stairs = basic_stairs();
        assert!(stairs.dominates(2, 6));
        assert!(stairs.dominates(4, 4));
        assert!(stairs.dominates(2, 7));
        assert!(stairs.dominates(3, 6));
        assert!(stairs.dominates(10, 10));
        assert!(!stairs.dominates(3, 5));
        assert!(!stairs.dominates(4, 3));
    }

    #[test]
    fn test_clear_dominated() {
        let mut stairs = basic_stairs();
        stairs.clear_dominated(4, 2);
        let steps: Vec<_> = stairs
            .into_iter()
            .map(|(b, v)| (b.width, b.height, v))
            .collect();
        assert_eq!(steps, vec![(2, 6, 'a')]);

        let mut stairs = basic_stairs();
        stairs.clear_dominated(4, 3);
        let steps: Vec<_> = stairs
            .into_iter()
            .map(|(b, v)| (b.width, b.height, v))
            .collect();
        assert_eq!(steps, vec![(6, 2, 'a'), (2, 6, 'a')]);

        let mut stairs = basic_stairs();
        stairs.clear_dominated(5, 2);
        let steps: Vec<_> = stairs
            .into_iter()
            .map(|(b, v)| (b.width, b.height, v))
            .collect();
        assert_eq!(steps, vec![(4, 4, 'a'), (2, 6, 'a')]);
    }

    #[test]
    fn test_empty_staircase() {
        let stairs: Staircase<char> = Staircase::new(4);
        let steps: Vec<_> = stairs.into_iter().collect();
        assert_eq!(steps, vec![]);
    }

    #[test]
    fn test_staircase() {
        let mut stairs = basic_stairs();
        stairs.insert(5, 5, 'x');
        stairs.insert(4, 4, 'b');
        stairs.insert(4, 5, 'x');
        stairs.insert(3, 5, 'b');
        stairs.insert(8, 5, 'x');
        stairs.insert(5, 3, 'b');
        stairs.insert(6, 3, 'x');
        let steps: Vec<_> = stairs
            .into_iter()
            .map(|(b, v)| (b.width, b.height, v))
            .collect();
        assert_eq!(
            steps,
            vec![
                (6, 2, 'a'),
                (5, 3, 'b'),
                (4, 4, 'a'),
                (3, 5, 'b'),
                (2, 6, 'a')
            ]
        );
    }

    #[test]
    fn test_staircase_fit_bound() {
        let stairs = basic_stairs();
        assert_eq!(
            stairs.fit_bound(Bound {
                width: 4,
                height: 7,
                indent: 0,
            }),
            Some((
                Bound {
                    width: 4,
                    height: 4,
                    indent: 0,
                },
                &'a'
            ))
        );
        assert_eq!(
            stairs.fit_bound(Bound {
                width: 1,
                height: 100,
                indent: 0,
            }),
            None
        );
        assert_eq!(
            stairs.fit_bound(Bound {
                width: 100,
                height: 100,
                indent: 100,
            }),
            Some((
                Bound {
                    width: 6,
                    height: 2,
                    indent: 0,
                },
                &'a'
            ))
        );
    }

    #[test]
    fn test_staircase_fit_width() {
        let stairs = basic_stairs();
        assert_eq!(stairs.fit_width(1), None,);
        assert_eq!(
            stairs.fit_width(2),
            Some((
                Bound {
                    width: 2,
                    height: 6,
                    indent: 0,
                },
                &'a'
            ))
        );
        assert_eq!(
            stairs.fit_width(3),
            Some((
                Bound {
                    width: 2,
                    height: 6,
                    indent: 0,
                },
                &'a'
            ))
        );
        assert_eq!(
            stairs.fit_width(4),
            Some((
                Bound {
                    width: 4,
                    height: 4,
                    indent: 0,
                },
                &'a'
            ))
        );
        assert_eq!(
            stairs.fit_width(6),
            Some((
                Bound {
                    width: 6,
                    height: 2,
                    indent: 0,
                },
                &'a'
            ))
        );
        assert_eq!(
            stairs.fit_width(7),
            Some((
                Bound {
                    width: 6,
                    height: 2,
                    indent: 0,
                },
                &'a'
            ))
        );
        assert_eq!(
            stairs.fit_width(100),
            Some((
                Bound {
                    width: 6,
                    height: 2,
                    indent: 0,
                },
                &'a'
            ))
        );
    }
}
