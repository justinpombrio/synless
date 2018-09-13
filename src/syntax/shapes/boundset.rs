use std::iter;

use super::bound::Bound;


/// A set of Bounds. If one Bound is strictly smaller than another,
/// only the smaller one will be kept.
#[derive(Clone, Debug)]
pub struct BoundSet {
    set: Vec<Bound>
}

impl BoundSet {
    /// Construct an empty BoundSet.
    pub fn new() -> BoundSet {
        BoundSet{
            set: vec!()
        }
    }

    /// Filter out Bounds that don't fit within the given Bound.
    /// Panics if none are left.
    pub fn fit_bound(&self, space: Bound) -> Bound {
        let bound = self.into_iter().filter(|bound| {
            bound.dominates(space)
        }).nth(0);
        match bound {
            Some(bound) => bound,
            None         => panic!("No bound fits within given width {}",
                                   space.width)
        }
    }

    pub fn singleton(bound: Bound) -> BoundSet {
        let mut set = BoundSet::new();
        set.insert(bound);
        set
    }

    // TODO: efficiency (can go from O(n) to O(sqrt(n)))
    // MUST FILTER IDENTICALLY TO LayoutSet::insert
    pub fn insert(&mut self, bound: Bound) {
        if bound.too_wide() {
            return;
        }
        for &b in &self.set {
            if b.dominates(bound) {
                return;
            }
        }
        self.set.retain(|&b| !bound.dominates(b));
        self.set.push(bound);
    }

    #[cfg(test)]
    pub(crate) fn first(&self) -> Bound {
        self.set[0]
    }
}


/// Iterator over Bounds in a BoundSet.
pub struct BoundIter<'a> {
    set: &'a Vec<Bound>,
    i: usize
}

impl<'a> Iterator for BoundIter<'a> {
    type Item = Bound;
    fn next(&mut self) -> Option<Bound> {
        if self.i >= self.set.len() {
            None
        } else {
            self.i += 1;
            Some(self.set[self.i - 1])
        }
    }
}

impl<'a> iter::IntoIterator for &'a BoundSet {
    type Item = Bound;
    type IntoIter = BoundIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        BoundIter {
            set: &self.set,
            i: 0
        }
    }
}

impl iter::FromIterator<Bound> for BoundSet {
    fn from_iter<T>(iter: T) -> BoundSet
        where T: iter::IntoIterator<Item = Bound>
    {
        let mut set = BoundSet::new();
        for bound in iter.into_iter() {
            set.insert(bound);
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
