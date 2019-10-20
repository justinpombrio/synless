use crate::geometry::Bound;
use crate::geometry::{Col, Row};
use std::fmt::Debug;
use std::iter;
use std::vec;

/// Store a set of (width, height) pairs with the property that if one pair
/// _dominates another_ by having smaller width and height, then the dominated
/// pair need not be stored. This property allows the pairs to be stored in
/// sorted order, of simultaneously _increasing_ heights and _decreasing_
/// widths. This in turn allows for efficient (log-time) insertion and lookup.
///
/// Staircases are used by Boundsets, which stores one Staircase for each
/// indent.
#[derive(Clone, Debug)]
pub struct Staircase<T>
where
    T: Clone + Debug,
{
    indent: Col,                // indent
    stairs: Vec<(Col, Row, T)>, // (width, height)
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
    #[cfg(test)]
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
            let (w, h, v) = &self.stairs[skip_left];
            let bound = Bound {
                width: *w,
                height: *h,
                indent: self.indent,
            };
            Some((bound, v))
        } else {
            None
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
    /// have already called `dominates` and `clear_dominated`.
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

impl<T: Clone + Debug> iter::IntoIterator for Staircase<T> {
    type Item = (Bound, T);
    type IntoIter = StaircaseIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        StaircaseIter {
            indent: self.indent,
            stairs: self.stairs.into_iter(),
        }
    }
}

pub struct StaircaseIter<T>
where
    T: Clone + Debug,
{
    indent: Col,
    stairs: vec::IntoIter<(Col, Row, T)>,
}

impl<T> Iterator for StaircaseIter<T>
where
    T: Clone + Debug,
{
    type Item = (Bound, T);
    fn next(&mut self) -> Option<(Bound, T)> {
        self.stairs.next().map(|(width, height, value)| {
            let bound = Bound {
                width,
                height,
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
}
