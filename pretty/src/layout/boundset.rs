use crate::geometry::{Bound, Col, MAX_WIDTH};
use crate::notation::NotationOps;
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

    /// Find the largest Bound that fits within the given width. Returns `None`
    /// if no Bound fits.
    pub fn fit_width(&self, width: Col) -> Option<Bound> {
        match self.set.binary_search_by_key(&width, |b| b.width) {
            Ok(i) => Some(self.set[i]),
            Err(0) => None,
            Err(i) => Some(self.set[i - 1]),
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

impl NotationOps for BoundSet {
    fn empty() -> BoundSet {
        BoundSet::singleton(Bound::empty())
    }

    fn literal(string: &str) -> BoundSet {
        BoundSet::singleton(Bound::literal(string))
    }

    fn nest(set1: BoundSet, set2: BoundSet) -> BoundSet {
        let mut set = BoundSet::new();
        for width in 0..MAX_WIDTH {
            if let Some(bound1) = set1.fit_width(width) {
                let remaining_width = width - bound1.indent;
                if let Some(bound2) = set2.fit_width(remaining_width) {
                    set.insert(Bound::nest(bound1, bound2));
                }
            }
        }
        set
    }

    fn vert(set1: BoundSet, set2: BoundSet) -> BoundSet {
        let mut set = BoundSet::new();
        for width in 0..MAX_WIDTH {
            if let Some(bound1) = set1.fit_width(width) {
                if let Some(bound2) = set2.fit_width(width) {
                    set.insert(Bound::vert(bound1, bound2));
                }
            }
        }
        set
    }

    fn if_flat(set1: BoundSet, set2: BoundSet) -> BoundSet {
        let mut set = BoundSet::new();
        for width in 0..MAX_WIDTH {
            if let Some(bound1) = set1.fit_width(width) {
                if bound1.height == 1 {
                    set.insert(bound1);
                } else if let Some(bound2) = set2.fit_width(width) {
                    set.insert(bound2);
                }
            } else if let Some(bound2) = set2.fit_width(width) {
                set.insert(bound2);
            }
        }
        set
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
        assert!(set.fit_width(1) == Some(bound2));
        assert!(set.fit_width(2) == Some(bound2));
        assert!(set.fit_width(3) == Some(bound1));
        assert!(set.fit_width(4) == Some(bound1));
        assert!(set.fit_width(5) == Some(bound1));
    }
}
