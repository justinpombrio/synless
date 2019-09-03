use crate::geometry::{Bound, Col, MAX_WIDTH};
use crate::notation_ops::NotationOps;
use crate::style::Style;
use utility::error;

use std::ops;

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

impl NotationOps for BoundSet {
    fn empty() -> BoundSet {
        BoundSet::singleton(Bound::empty())
    }

    fn literal(string: &str, style: Style) -> BoundSet {
        BoundSet::singleton(Bound::literal(string, style))
    }

    fn text(child: &BoundSet, _style: Style) -> BoundSet {
        child.clone()
    }

    fn child(children: &[BoundSet], i: usize) -> BoundSet {
        children[i].clone()
    }

    fn nest(set_1: BoundSet, set_2: BoundSet) -> BoundSet {
        let mut set = BoundSet::new();
        for width in 0..MAX_WIDTH {
            let bound_1 = set_1[width];
            let remaining_width = width - bound_1.indent;
            let bound_2 = set_2[remaining_width];
            set.insert(Bound::nest(bound_1, bound_2));
        }
        set
    }

    fn vert(set_1: BoundSet, set_2: BoundSet) -> BoundSet {
        let mut set = BoundSet::new();
        for width in 0..MAX_WIDTH {
            let bound_1 = set_1[width];
            let bound_2 = set_2[width];
            set.insert(Bound::vert(bound_1, bound_2));
        }
        set
    }

    fn if_flat(set_1: BoundSet, set_2: BoundSet) -> BoundSet {
        let mut set = BoundSet::new();
        for width in 0..MAX_WIDTH {
            let bound_1 = set_1[width];
            if bound_1.height > 1 {
                set.insert(bound_1);
            } else {
                set.insert(set_2[width]);
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
        assert!(set[1] == bound2);
        assert!(set[2] == bound2);
        assert!(set[3] == bound1);
        assert!(set[4] == bound1);
        assert!(set[5] == bound1);
    }
}
