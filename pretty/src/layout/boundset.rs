use super::staircase::{Staircase, StaircaseIter};
use crate::geometry::Bound;
#[cfg(test)]
use crate::geometry::Col;
use std::fmt;
use std::iter;
use std::vec;

/// A set of Bounds. If one Bound is strictly smaller than another,
/// only the smaller one will be kept.
/// Each Bound may have some related data T.
#[derive(Clone, Debug)]
pub struct BoundSet<T>
where
    T: Clone + fmt::Debug,
{
    /// Staircases, stored by increasing indent.
    staircases: Vec<Staircase<T>>,
}

impl<T> BoundSet<T>
where
    T: Clone + fmt::Debug,
{
    /// Construct an empty BoundSet.
    pub fn new() -> BoundSet<T> {
        BoundSet { staircases: vec![] }
    }

    /// Pick the best (i.e., shortest) Bound that fits within the
    /// given Bound. Panics if none fit.
    pub fn fit_bound(&self, space: Bound) -> (Bound, T) {
        let mut best = None;
        for staircase in &self.staircases {
            if let Some((bound, value)) = staircase.fit_bound(space) {
                match best {
                    None => best = Some((bound, value)),
                    Some((best_bound, _)) => {
                        if bound.height < best_bound.height {
                            best = Some((bound, value));
                        }
                    }
                }
            }
        }
        match best {
            None => panic!(
                "No bound fits within the given bound {:?}.\nBoundset: {:?}",
                space, self
            ),
            Some((bound, value)) => (bound, value.to_owned()),
        }
    }

    /// Construct a BoundSet that contains only a single bound.
    pub fn singleton(bound: Bound, value: T) -> BoundSet<T> {
        let mut set = BoundSet::new();
        set.insert(bound, value);
        set
    }

    /// Insert a Bound into the BoundSet. If it is dominated by a Bound already
    /// in the set, it will not be added. If it dominates Bounds already in the
    /// set, they will be removed.
    pub fn insert(&mut self, bound: Bound, value: T) {
        if bound.too_wide() {
            return;
        }
        for staircase in &mut self.staircases {
            let indent = staircase.indent();
            // Check to see if this bound is dominated.
            if indent <= bound.indent && staircase.dominates(bound.width, bound.height) {
                return;
            }
            // Remove bounds that this bound dominates.
            if indent >= bound.indent {
                staircase.clear_dominated(bound.width, bound.height);
            }
        }
        match self
            .staircases
            .binary_search_by_key(&bound.indent, |s| s.indent())
        {
            Ok(i) => self.staircases[i].unchecked_insert(bound.width, bound.height, value),
            Err(i) => {
                let mut staircase = Staircase::new(bound.indent);
                staircase.unchecked_insert(bound.width, bound.height, value);
                self.staircases.insert(i, staircase);
            }
        }
    }

    /// Combine two boundsets. Produces a boundset whose elements are
    /// `(f(b1, b2), g(t1, t2))`
    /// for all `(b1, t1)` in `set1` and all `(b2, t2)` in `set2`.
    pub fn combine<F, G>(set1: &BoundSet<T>, set2: &BoundSet<T>, f: F, g: G) -> BoundSet<T>
    where
        F: Fn(Bound, Bound) -> Bound,
        G: Fn(T, T) -> T,
    {
        let mut set = BoundSet::new();
        for (bound1, val1) in set1.to_owned().into_iter() {
            for (bound2, val2) in set2.to_owned().into_iter() {
                let bound = f(bound1, bound2);
                let val = g(val1.clone(), val2);
                set.insert(bound, val);
            }
        }
        set
    }

    /// Pick the best (i.e., smallest) Bound that fits within the
    /// given width. Panics if none fit.
    #[cfg(test)]
    pub fn fit_width(&self, width: Col) -> (Bound, T) {
        let bound = Bound::infinite_scroll(width);
        self.fit_bound(bound)
    }
}

/// Iterator over Bounds in a BoundSet.
pub struct BoundSetIter<T>
where
    T: Clone + fmt::Debug,
{
    staircases: vec::IntoIter<Staircase<T>>,
    bounds: Option<StaircaseIter<T>>,
}

impl<T> Iterator for BoundSetIter<T>
where
    T: Clone + fmt::Debug,
{
    type Item = (Bound, T);
    fn next(&mut self) -> Option<(Bound, T)> {
        loop {
            match self.bounds.as_mut().and_then(|iter| iter.next()) {
                Some((bound, value)) => return Some((bound, value)),
                None => match self.staircases.next() {
                    None => return None,
                    Some(staircase) => self.bounds = Some(staircase.into_iter()),
                },
            }
        }
    }
}

impl<T> iter::IntoIterator for BoundSet<T>
where
    T: Clone + fmt::Debug,
{
    type Item = (Bound, T);
    type IntoIter = BoundSetIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        BoundSetIter {
            staircases: self.staircases.into_iter(),
            bounds: None,
        }
    }
}

impl<T> iter::FromIterator<(Bound, T)> for BoundSet<T>
where
    T: Clone + fmt::Debug,
{
    fn from_iter<I>(iter: I) -> BoundSet<T>
    where
        I: iter::IntoIterator<Item = (Bound, T)>,
    {
        let mut set = BoundSet::new();
        for (bound, val) in iter.into_iter() {
            set.insert(bound, val);
        }
        set
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_show_bound() {
        let r = Bound {
            width: 10,
            indent: 6,
            height: 3,
        };
        let mut s = String::new();
        r.debug_print(&mut s, '*', 2).unwrap();
        assert_eq!(
            s,
            "**********
  **********
  ******"
        );
    }

    #[test]
    fn test_show_empty_bound() {
        let r = Bound {
            width: 0,
            indent: 0,
            height: 1,
        };
        let mut s = String::new();
        r.debug_print(&mut s, '*', 2).unwrap();
        assert_eq!(s, "");
    }

    #[test]
    fn test_domination() {
        let r_best = Bound {
            width: 10,
            indent: 6,
            height: 3,
        };
        let r_1 = Bound {
            width: 11,
            indent: 6,
            height: 3,
        };
        let r_2 = Bound {
            width: 10,
            indent: 7,
            height: 3,
        };
        let r_3 = Bound {
            width: 10,
            indent: 6,
            height: 4,
        };
        let r_worst = Bound {
            width: 11,
            indent: 7,
            height: 4,
        };
        assert!(r_best.dominates(r_best));
        assert!(r_best.dominates(r_worst));
        assert!(r_best.dominates(r_1));
        assert!(r_best.dominates(r_2));
        assert!(r_best.dominates(r_3));
        assert!(r_1.dominates(r_worst));
        assert!(r_2.dominates(r_worst));
        assert!(r_3.dominates(r_worst));
        assert!(!r_1.dominates(r_2));
        assert!(!r_1.dominates(r_3));
        assert!(!r_2.dominates(r_1));
        assert!(!r_2.dominates(r_3));
        assert!(!r_3.dominates(r_1));
        assert!(!r_3.dominates(r_2));
    }

    #[test]
    fn test_boundset() {
        let r_1 = Bound {
            width: 11,
            indent: 6,
            height: 3,
        };
        let r_2 = Bound {
            width: 10,
            indent: 7,
            height: 3,
        };
        let r_3 = Bound {
            width: 10,
            indent: 6,
            height: 4,
        };
        let r_worst = Bound {
            width: 11,
            indent: 7,
            height: 4,
        };
        let mut set = BoundSet::new();
        set.insert(r_worst, 0);
        set.insert(r_1, 1);
        set.insert(r_worst, 0);
        set.insert(r_2, 2);
        set.insert(r_worst, 0);
        set.insert(r_3, 3);
        set.insert(r_worst, 0);
        let bounds: Vec<_> = set.into_iter().collect();
        assert_eq!(bounds, vec![(r_1, 1), (r_3, 3), (r_2, 2)]);
    }
}
