use std::fmt;
use std::iter;
use std::ops;

use crate::geometry::{Bound, Col};
use utility::error;

/// A map from width (Col) to Bound. If a Notation has a particular BoundSet `BS`,
/// that means that for each width `w`, if the Notation is rendered with that
/// width, it will take up space `BS[w]`.
///
/// **Invariant after initialization:** must be non-empty, and contain an
/// element at width=1.
#[derive(Debug, Clone)]
pub struct BoundSet {
    set: Vec<Bound>,
}

impl BoundSet {
    /// Construct an empty BoundSet.
    pub fn new() -> BoundSet {
        BoundSet { set: Vec::new() }
    }

    /// Find the largest Bound that fits within the given width. Panics if none
    /// fit.
    pub fn fit_width(&self, width: Col) -> &Bound {
        match self.set.binary_search_by_key(&width, |b| b.width) {
            Ok(i) => &self.set[i],
            Err(i) => &self.set[i - 1],
        }
    }

    /// Construct a BoundSet containing a single Bound.
    pub fn singleton(bound: Bound) -> BoundSet {
        let mut set = BoundSet::new();
        set.insert(bound);
        set
    }

    /// Insert a Bound into the set. Panics if you try to insert two different
    /// Bounds that have the same width.
    pub fn insert(&mut self, bound: Bound) {
        if bound.too_wide() {
            return;
        }
        match self.set.binary_search_by_key(&bound.width, |b| b.width) {
            Ok(i) => {
                if self.set[i] != bound {
                    error!("BoundSet: duplicate bound width")
                }
            }
            Err(i) => {
                if i == 0 || self.set[i - 1] != bound {
                    self.set.insert(i, bound);
                }
            }
        }
    }
}

impl ops::Index<Col> for BoundSet {
    type Output = Bound;

    /// A shorthand for `fit_width`.
    fn index(&self, width: Col) -> &Bound {
        self.fit_width(width)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_boundset() {
        let mut set = BoundSet::new();
        let bound1 = Bound::new_rectangle(10, 3);
        let bound2 = Bound::new_rectangle(20, 1);
        set.insert(bound1);
        set.insert(bound2);
        set.insert(bound2);
        set.insert(bound1);
        assert_eq!(set.set.len(), 2);
        assert!(set[1] == bound2);
        assert!(set[2] == bound2);
        assert!(set[3] == bound1);
        assert!(set[4] == bound1);
        assert!(set[5] == bound1);
    }
}
