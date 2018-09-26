use std::iter;

#[cfg(test)]
use common::Col;
use super::super::Bound;


/// A set of Bounds. If one Bound is strictly smaller than another,
/// only the smaller one will be kept.
/// Each Bound may have some related data T.
#[derive(Clone, Debug)]
pub struct BoundSet<T> where T: Clone {
    set: Vec<(Bound, T)>
}

impl<T> BoundSet<T> where T: Clone {
    /// Construct an empty BoundSet.
    pub fn new() -> BoundSet<T> {
        BoundSet {
            set: vec!()
        }
    }

    /// Pick the best (i.e., smallest) Bound that fits within the
    /// given Bound. Panics if none fit.
    pub fn fit_bound(&self, space: Bound) -> (Bound, T) {
        let bound = self.into_iter().filter(|(bound, _)| {
            bound.dominates(space)
        }).nth(0);
        match bound {
            Some(bound) => bound,
            None        => panic!("No bound fits within given width {}",
                                  space.width)
        }
    }

    /// Pick the best (i.e., smallest) Bound that fits within the
    /// given width. Panics if none fit.
    #[cfg(test)]
    pub(crate) fn fit_width(&self, width: Col) -> (Bound, T) {
        let bound = Bound::infinite_scroll(width);
        self.fit_bound(bound)
    }

    pub fn singleton(bound: Bound, val: T) -> BoundSet<T> {
        let mut set = BoundSet::new();
        set.insert(bound, val);
        set
    }

    // TODO: efficiency (can go from O(n) to O(sqrt(n)))
    // MUST FILTER IDENTICALLY TO LayoutSet::insert
    pub fn insert(&mut self, bound: Bound, val: T) {
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

    #[cfg(test)]
    pub(crate) fn first(&self) -> (Bound, T) {
        self.set[0].clone()
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
