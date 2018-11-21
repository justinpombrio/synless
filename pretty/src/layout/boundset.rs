use std::iter;
use std::fmt;

use crate::geometry::Bound;
#[cfg(test)]
use crate::geometry::Col;


/// A set of Bounds. If one Bound is strictly smaller than another,
/// only the smaller one will be kept.
/// Each Bound may have some related data T.
#[derive(Clone)]
pub struct BoundSet<T> where T: Clone {
    set: Vec<(Bound, T)>
}

impl<T> BoundSet<T> where T: Clone {
    /// Construct an empty BoundSet.
    pub(super) fn new() -> BoundSet<T> {
        BoundSet {
            set: vec!()
        }
    }

    /// Pick the best (i.e., smallest) Bound that fits within the
    /// given Bound. Panics if none fit.
    pub(super) fn fit_bound(&self, space: Bound) -> (Bound, T) {
        let bound = self.into_iter().filter(|(bound, _)| {
            bound.dominates(space)
        }).nth(0);
        match bound {
            Some(bound) => bound,
            None        => panic!("No bound fits within given width {}.\nBoundset: {:?}",
                                  space.width, self)
        }
    }

    pub(super) fn singleton(bound: Bound, val: T) -> BoundSet<T> {
        let mut set = BoundSet::new();
        set.insert(bound, val);
        set
    }

    // TODO: efficiency (can go from O(n) to O(sqrt(n)))
    // MUST FILTER IDENTICALLY TO LayoutSet::insert
    pub(super) fn insert(&mut self, bound: Bound, val: T) {
        if bound.too_wide() {
            return;
        }
        for (b, _) in &self.set {
            if b.dominates(bound) {
                return;
            }
        }
        self.set.retain(|&(b, _)| !bound.dominates(b));
        self.set.push((bound, val));
    }

    /// Combine two boundsets. Produces a boundset whose elements are
    /// `(f(b1, b2), g(t1, t2))`
    /// for all `(b1, t1)` in `set1` and all `(b2, t2)` in `set2`.
    pub(super) fn combine<F, G>(set1: &BoundSet<T>, set2: &BoundSet<T>, f: F, g: G)
                                -> BoundSet<T>
        where F: Fn(Bound, Bound) -> Bound, G: Fn(T, T) -> T
    {
        let mut set = BoundSet::new();
        for (bound1, val1) in set1.into_iter() {
            for (bound2, val2) in set2.into_iter() {
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
    pub(super) fn fit_width(&self, width: Col) -> (Bound, T) {
        let bound = Bound::infinite_scroll(width);
        self.fit_bound(bound)
    }

    #[cfg(test)]
    pub(super) fn first(&self) -> (Bound, T) {
        self.set[0].clone()
    }
}

impl<T> fmt::Debug for BoundSet<T> where T: Clone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let set: Vec<&Bound> = self.set.iter().map(|(bound, _)| bound).collect();
        write!(f, "{:?}", set)
    }
}

/// Iterator over Bounds in a BoundSet.
pub struct BoundIter<'a, T: 'a> where T: Clone {
    set: &'a Vec<(Bound, T)>,
    i: usize
}

impl<'a, T> Iterator for BoundIter<'a, T> where T: Clone {
    type Item = (Bound, T);
    fn next(&mut self) -> Option<(Bound, T)> {
        if self.i >= self.set.len() {
            None
        } else {
            self.i += 1;
            Some(self.set[self.i - 1].clone())
        }
    }
}

impl<'a, T> iter::IntoIterator for &'a BoundSet<T> where T: Clone {
    type Item = (Bound, T);
    type IntoIter = BoundIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        BoundIter {
            set: &self.set,
            i: 0
        }
    }
}

impl<T> iter::FromIterator<(Bound, T)> for BoundSet<T>
    where T: Clone
{
    fn from_iter<I>(iter: I) -> BoundSet<T>
        where I: iter::IntoIterator<Item = (Bound, T)>
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
    use std::fmt::Write;
    use super::*;


    #[test]
    fn test_show_bound() {
        let r = Bound{
            width:  10,
            indent: 6,
            height: 2
        };
        let mut s = String::new();
        write!(&mut s, "{:?}", r).unwrap();
        assert_eq!(s,
"**********
**********
******");
    }

    #[test]
    fn test_show_empty_bound() {
        let r = Bound {
            width: 0,
            indent: 0,
            height: 0
        };
        let mut s = String::new();
        write!(&mut s, "{:?}", r).unwrap();
        assert_eq!(s, "");
    }

    #[test]
    fn test_domination() {
        let r_best  = Bound{ width: 10, indent: 6, height: 2 };
        let r_1     = Bound{ width: 11, indent: 6, height: 2 };
        let r_2     = Bound{ width: 10, indent: 7, height: 2 };
        let r_3     = Bound{ width: 10, indent: 6, height: 3 };
        let r_worst = Bound{ width: 11, indent: 7, height: 3 };
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
}
