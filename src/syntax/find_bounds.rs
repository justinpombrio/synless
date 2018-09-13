use common::{Row, Col};
use super::shapes::{Bound, BoundSet};
use super::syntax::Syntax;
use super::syntax::Syntax::*;

use std::cmp;


impl Bound {
    // TODO: make private?
    // TODO: text wrapping
    pub fn literal(s: &str) -> Bound {
        let width = s.chars().count() as Col;
        Bound{
            width:  width,
            indent: width,
            height: 0
        }
    }

    fn flush(&self) -> Bound {
        Bound{
            width:  self.width,
            indent: 0,
            height: self.height + 1
        }
    }

    fn concat(&self, other: Bound) -> Bound {
        Bound{
            width:  cmp::max(self.width,
                             self.indent + other.width),
            height: self.height + other.height,
            indent: self.indent + other.indent
        }
    }
}


impl BoundSet {
    // TODO: Why is this public?
    fn literal(s: &str) -> BoundSet {
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

    fn no_wrap(&self) -> BoundSet {
        self.into_iter().filter(|bound| {
            bound.height == 0
        }).collect()
    }

    fn choice(&self, other: &BoundSet) -> BoundSet {
        self.into_iter().chain(other.into_iter()).collect()
    }
}


impl Syntax {
    /// Find the Bound within which this `Syntax` can be displayed,
    /// when that its children have the given bounds.
    pub fn find_bounds(&self,
                       arity: usize,
                       child_bounds: Vec<&BoundSet>,
                       empty_text: bool)
                       -> BoundSet
    {
        self.expand(arity, child_bounds.len(), empty_text)
            .bound(&child_bounds)
    }
    
    fn bound(&self, child_bounds: &Vec<&BoundSet>) -> BoundSet {
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
                let set = syn.bound(child_bounds);
                set.flush()
            }
            &Concat(ref syn1, ref syn2) => {
                let set1 = syn1.bound(child_bounds);
                let set2 = syn2.bound(child_bounds);
                set1.concat(&set2)
            }
            &NoWrap(ref syn) => {
                syn.bound(child_bounds).no_wrap()
            }
            &Choice(ref syn1, ref syn2) => {
                let set1 = syn1.bound(child_bounds);
                let set2 = syn2.bound(child_bounds);
                set1.choice(&set2)
            }
            &IfEmptyText(_, _) => panic!("find_bounds: unexpected IfEmptyText"),
            &Rep(_) => panic!("find_bounds: unexpected Repeat"),
            &Star   => panic!("find_bounds: unexpected Star")
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::syntax::*;
    use super::super::syntax::Syntax::*;
    use style::Style;

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
        let actual = example_syntax().find_bounds(0, vec!(), false).first();
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
            .find_bounds(0, vec!(), false).first();
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
            .find_bounds(0, vec!(), true).first();
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
            .find_bounds(0, vec!(), false).first();
        let expected = Bound{
            width: 2,
            indent: 2,
            height: 0
        };
        assert_eq!(actual, expected);
    }
}
