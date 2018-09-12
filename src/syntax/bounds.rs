use std::{cmp, fmt, iter};

use geometry::*;
use syntax::syntax::Syntax;
use syntax::syntax::Syntax::*;
use syntax::syntax::MAX_WIDTH;


impl Bound {
    /// A  Bound that has the given width and is "infinitely" tall.
    pub fn infinite_scroll(width: Col) -> Bound {
        Bound{
            width:  width,
            indent: width,
            height: Row::max_value()
        }
    }

    /// Is this Bound wider than MAX_WIDTH?
    /// Anything wider than MAX_WIDTH will simply be ignored: no one
    /// needs more than MAX_WIDTH characters on a line.
    pub fn too_wide(&self) -> bool {
        self.width > MAX_WIDTH
    }

    /// One Bound dominates another if it is completely covered by it.
    pub fn dominates(&self, other: Bound) -> bool {
        // self wins ties
        (self.width <= other.width) &&
            (self.height <= other.height) &&
            (self.indent <= other.indent)
    }

    // TODO: text wrapping
    /// Bound around a syntax Literal.
    pub fn literal(s: &str) -> Bound {
        let width = s.chars().count() as Col;
        Bound{
            width:  width,
            indent: width,
            height: 0
        }
    }

    /// Add a newline to this Bound.
    pub fn flush(&self) -> Bound {
        Bound{
            width:  self.width,
            indent: 0,
            height: self.height + 1
        }
    }

    /// Concatenate two Bound.
    /// The second starts at the end of the indent of the first.
    pub fn concat(&self, other: Bound) -> Bound {
        Bound{
            width:  cmp::max(self.width,
                             self.indent + other.width),
            height: self.height + other.height,
            indent: self.indent + other.indent
        }
    }

    pub(crate) fn debug_print(&self, f: &mut fmt::Formatter, ch: char, indent: Col)
                              -> fmt::Result
    {
        if self.height > 30 {
            return write!(f, "[very large bound]")
        }
        for _ in 0..self.height {
            write!(f, "{}", ch.to_string().repeat(self.width as usize))?;
            write!(f, "\n")?;
            write!(f, "{}", " ".repeat(indent as usize))?;
        }
        write!(f, "{}", ch.to_string().repeat(self.indent as usize))
    }
}

impl fmt::Debug for Bound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.debug_print(f, '*', 0)
    }
}


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

    fn singleton(bound: Bound) -> BoundSet {
        let mut set = BoundSet::new();
        set.insert(bound);
        set
    }

    // TODO: efficiency (can go from O(n) to O(sqrt(n)))
    // MUST FILTER IDENTICALLY TO LayoutSet::insert
    fn insert(&mut self, bound: Bound) {
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

    // TODO: Why is this public?
    pub fn literal(s: &str) -> BoundSet {
        BoundSet::singleton(Bound::literal(s))
    }

    fn flush(&self) -> BoundSet {
        self.into_iter().map(|bound| bound.flush()).collect()
    }

    fn concat(&self, other: &BoundSet) -> BoundSet {
        let mut set = BoundSet::new();
        for bound1 in self {
            for bound2 in other {
                set.insert(bound1.concat(bound2))
            }
        }
        set
    }

    fn choice(&self, other: &BoundSet) -> BoundSet {
        self.into_iter().chain(other.into_iter()).collect()
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


impl Syntax {
    /// Find the Bound within which this `Syntax` can be displayed,
    /// given that its children have the bound given.
    pub fn bound(&self,
                 arity: usize,
                 child_bounds: Vec<&BoundSet>,
                 empty_text: bool)
                 -> BoundSet
    {
        self.expand(arity, child_bounds.len(), empty_text)
            .bound_rec(&child_bounds)
    }
    
    fn bound_rec(&self, child_bounds: &Vec<&BoundSet>) -> BoundSet {
        match self {
            &Literal(ref s, _) => {
                BoundSet::literal(s)
            }
            &Text(_) => {
                child_bounds[0].clone()
            }
            &Child(index) => {
                child_bounds[index].clone()
            }
            &Flush(ref syn) => {
                let set = syn.bound_rec(child_bounds);
                set.flush()
            }
            &Concat(ref syn1, ref syn2) => {
                let set1 = syn1.bound_rec(child_bounds);
                let set2 = syn2.bound_rec(child_bounds);
                set1.concat(&set2)
            }
            &NoWrap(ref syn) => {
                let mut set = syn.bound_rec(child_bounds);
                set.set.retain(|bound| {
                    bound.height == 0
                });
                set
            }
            &Choice(ref syn1, ref syn2) => {
                let set1 = syn1.bound_rec(child_bounds);
                let set2 = syn2.bound_rec(child_bounds);
                set1.choice(&set2)
            }
            &IfEmptyText(_, _) => panic!("bound_rec: unexpected IfEmptyText"),
            &Rep(_) => panic!("bound_rec: unexpected Repeat"),
            &Star   => panic!("bound_rec: unexpected Star")
        }
    }
}


#[cfg(test)]
mod tests {
    use std::fmt::Write;
    use super::*;
    use syntax::*;
    use style::Style;

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
        let r = Bound::literal("");
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

    #[test]
    fn test_bound_construction() {
        let actual = Bound::literal("abc").flush().concat(
            Bound::literal("Schrödinger").concat(
                Bound::literal("I").concat(
                    Bound::literal(" am indented")).flush().concat(
                    Bound::literal("me too"))));
        let expected = Bound{
            width:  24,
            indent: 17,
            height: 2
        };
        assert_eq!(actual, expected);
    }

    fn lit(s: &str) -> Syntax {
        literal(s, Style::plain())
    }

    fn example_syntax() -> Syntax {
        flush(lit("if ") + lit("true"))
            + flush(lit("  ")
                + lit("* ")
                + flush(lit("bulleted"))
                + lit("list"))
            + lit("end")
    }

    #[test]
    fn test_bound() {
        let actual = example_syntax().bound(0, vec!(), false).set[0];
        let expected = Bound{
            width:  12,
            indent: 3,
            height: 3
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_bound_2() {
        let actual = (flush(lit("abc")) + lit("de"))
            .bound(0, vec!(), false).set[0];
        let expected = Bound{
            width:  3,
            indent: 2,
            height: 1
        };
        assert_eq!(actual, expected);
        assert_eq!(format!("{:?}", actual), "***\n**");
    }

    #[test]
    fn test_bound_3() {
        let actual = if_empty_text(lit("a"), lit("bc"))
            .bound(0, vec!(), true).set[0];
        let expected = Bound{
            width: 1,
            indent: 1,
            height: 0
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_bound_4() {
        let actual = if_empty_text(lit("a"), lit("bc"))
            .bound(0, vec!(), false).set[0];
        let expected = Bound{
            width: 2,
            indent: 2,
            height: 0
        };
        assert_eq!(actual, expected);
    }
}
