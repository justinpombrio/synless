use super::staircase::{Staircase, StaircaseIntoIter, StaircaseIter};
use crate::geometry::Bound;
use crate::geometry::Col;
use std::fmt::Debug;
use std::{iter, slice, vec};

/// A set of Bounds. If one Bound is strictly smaller than another, only the
/// smaller one will be kept. Each stair in the staircase also contains some
/// related data T.
#[derive(Clone, Debug, Default)]
pub struct BoundSet<T: Clone + Debug> {
    /// Staircases, stored by increasing indent.
    staircases: Vec<Staircase<T>>,
}

impl<T: Clone + Debug> BoundSet<T> {
    /// Construct an empty BoundSet.
    pub fn new() -> BoundSet<T> {
        BoundSet { staircases: vec![] }
    }

    pub fn staircases(&self) -> &[Staircase<T>] {
        &self.staircases
    }

    /// Pick the best (i.e., shortest) Bound that fits within the
    /// given Bound. Panics if none fit.
    pub fn fit_bound(&self, space: Bound) -> (Bound, &T) {
        let mut best = None;
        for staircase in &self.staircases {
            if let Some((bound, value)) = staircase.fit_bound(space) {
                match best {
                    None => best = Some((bound, value)),
                    Some((best_bound, _)) => {
                        if bound.height < best_bound.height {
                            best = Some((bound, value))
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
            Some((bound, value)) => (bound, value),
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
        self.unchecked_insert(bound, value);
    }

    /// Insert a bound without checking domination. Only use this if you you
    /// know it won't dominate or be dominated by any bound in the set.
    pub fn unchecked_insert(&mut self, bound: Bound, value: T) {
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

    /// Pick the best (i.e., smallest) Bound that fits within the
    /// given width. Panics if none fit.
    pub fn fit_width(&self, width: Col) -> (Bound, &T) {
        // TODO: better way?
        let bound = Bound::infinite_scroll(width);
        self.fit_bound(bound)
    }

    pub fn iter(&self) -> BoundSetIter<T> {
        self.into_iter()
    }
}

/// Iterator over Bounds in a &BoundSet.
pub struct BoundSetIter<'a, T: Debug + Clone> {
    staircases: slice::Iter<'a, Staircase<T>>,
    bounds: Option<StaircaseIter<'a, T>>,
}

impl<'a, T: Debug + Clone> Iterator for BoundSetIter<'a, T> {
    type Item = (Bound, &'a T);
    fn next(&mut self) -> Option<(Bound, &'a T)> {
        loop {
            match self.bounds.as_mut().and_then(|iter| iter.next()) {
                Some(bound) => return Some(bound),
                None => match self.staircases.next() {
                    None => return None,
                    Some(staircase) => self.bounds = Some(staircase.into_iter()),
                },
            }
        }
    }
}

impl<'a, T: Debug + Clone> iter::IntoIterator for &'a BoundSet<T> {
    type Item = (Bound, &'a T);
    type IntoIter = BoundSetIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        BoundSetIter {
            staircases: self.staircases.iter(),
            bounds: None,
        }
    }
}

/// Iterator over Bounds in a BoundSet.
pub struct BoundSetIntoIter<T: Debug + Clone> {
    staircases: vec::IntoIter<Staircase<T>>,
    bounds: Option<StaircaseIntoIter<T>>,
}

impl<T: Debug + Clone> Iterator for BoundSetIntoIter<T> {
    type Item = (Bound, T);
    fn next(&mut self) -> Option<(Bound, T)> {
        loop {
            match self.bounds.as_mut().and_then(|iter| iter.next()) {
                Some(bound) => return Some(bound),
                None => match self.staircases.next() {
                    None => return None,
                    Some(staircase) => self.bounds = Some(staircase.into_iter()),
                },
            }
        }
    }
}

impl<T: Debug + Clone> iter::IntoIterator for BoundSet<T> {
    type Item = (Bound, T);
    type IntoIter = BoundSetIntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        BoundSetIntoIter {
            staircases: self.staircases.into_iter(),
            bounds: None,
        }
    }
}

impl<T: Debug + Clone> iter::FromIterator<(Bound, T)> for BoundSet<T> {
    /// WARNING: This uses `unchecked_insert`. Don't use it if some of the
    /// bounds might dominate one another!
    fn from_iter<I>(iter: I) -> BoundSet<T>
    where
        I: iter::IntoIterator<Item = (Bound, T)>,
    {
        let mut set = BoundSet::new();
        for (bound, value) in iter.into_iter() {
            set.unchecked_insert(bound, value);
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
