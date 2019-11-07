use super::boundset::BoundSet;
use super::staircase::Staircase;
use crate::geometry::{Bound, Col};
use crate::style::Style;
use std::cmp;
use std::fmt::Debug;

use ResolvedNotation::*;

pub trait NotationOps
where
    Self: Clone + Debug,
{
    fn empty() -> Self;
    fn literal(s: &str, style: Style) -> Self;
    // TODO: Make these into vecs
    fn follow(left: Self, right: Self) -> Self;
    fn vert(left: Self, right: Self) -> Self;
    fn text(child: Bound, style: Style) -> Self;
    fn child(i: usize, child: Bound) -> Self;
}

impl NotationOps for () {
    fn empty() {}
    fn literal(_s: &str, _style: Style) {}
    fn follow(_left: (), _right: ()) {}
    fn vert(_left: (), _right: ()) {}
    fn text(_child: Bound, _style: Style) {}
    fn child(_i: usize, _child: Bound) {}
}

impl NotationOps for Bound {
    fn empty() -> Bound {
        Bound {
            width: 0,
            height: 1,
            indent: 0,
        }
    }

    fn literal(string: &str, _style: Style) -> Bound {
        let width = string.chars().count() as Col;
        Bound {
            width: width,
            indent: width,
            height: 1,
        }
    }

    fn follow(b1: Bound, b2: Bound) -> Bound {
        Bound {
            width: cmp::max(b1.width, b1.indent + b2.width),
            height: b1.height + b2.height - 1,
            indent: b1.indent + b2.indent,
        }
    }

    fn vert(b1: Bound, b2: Bound) -> Bound {
        Bound {
            width: cmp::max(b1.width, b2.width),
            height: b1.height + b2.height,
            indent: b2.indent,
        }
    }

    fn text(child: Bound, _style: Style) -> Bound {
        child
    }

    fn child(_i: usize, child: Bound) -> Bound {
        child
    }
}

impl<T: NotationOps> NotationOps for BoundSet<T> {
    fn empty() -> BoundSet<T> {
        BoundSet::singleton(Bound::empty(), T::empty())
    }

    fn literal(string: &str, style: Style) -> BoundSet<T> {
        BoundSet::singleton(Bound::literal(string, style), T::literal(string, style))
    }

    fn follow(set1: BoundSet<T>, set2: BoundSet<T>) -> BoundSet<T> {
        let mut set = BoundSet::new();
        // Consider the case where the bound from set1 extends further to the
        // right than the bound from set2.
        for (b1, v1) in &set1 {
            let space_available = b1.width - b1.indent;
            for staircase2 in set2.staircases() {
                if staircase2.indent() > space_available {
                    break;
                }
                if let Some((b2, v2)) = staircase2.fit_width(space_available) {
                    set.insert(Bound::follow(b1, b2), T::follow(v1.clone(), v2.clone()));
                }
            }
        }
        // Consider the case where the bound from set2 extends further to the
        // right than the bound from set1.
        for (b2, v2) in set2 {
            for staircase1 in set1.staircases() {
                let space_available = staircase1.indent() + b2.width;
                if let Some((b1, v1)) = staircase1.fit_width(space_available) {
                    set.insert(Bound::follow(b1, b2), T::follow(v1.clone(), v2.clone()));
                }
            }
        }
        set
    }

    fn vert(set1: BoundSet<T>, set2: BoundSet<T>) -> BoundSet<T> {
        let mut set = BoundSet::new();
        // The indent of Bounds in set1 is irrelevant.
        let mut staircase1 = Staircase::new(0);
        for (b1, v1) in set1 {
            staircase1.insert(b1.width, b1.height, v1);
        }
        for (b2, v2) in set2 {
            for (b1, v1) in &staircase1 {
                set.insert(Bound::vert(b1, b2), T::vert(v1.clone(), v2.clone()));
            }
        }
        set
    }

    fn text(child: Bound, style: Style) -> BoundSet<T> {
        BoundSet::singleton(child, T::text(child, style))
    }

    fn child(i: usize, child: Bound) -> BoundSet<T> {
        BoundSet::singleton(child, T::child(i, child))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedNotation {
    Empty,
    Literal(String, Style, Bound),
    Follow(Box<ResolvedNotation>, Box<ResolvedNotation>),
    Vert(Box<ResolvedNotation>, Box<ResolvedNotation>),
    Text(Style, Bound),
    Child(usize, Bound),
}

impl NotationOps for ResolvedNotation {
    fn empty() -> ResolvedNotation {
        Empty
    }

    fn literal(string: &str, style: Style) -> ResolvedNotation {
        Literal(string.to_string(), style, Bound::literal(string, style))
    }

    fn follow(left: ResolvedNotation, right: ResolvedNotation) -> ResolvedNotation {
        Follow(Box::new(left), Box::new(right))
    }

    fn vert(left: ResolvedNotation, right: ResolvedNotation) -> ResolvedNotation {
        Vert(Box::new(left), Box::new(right))
    }

    fn text(bound: Bound, style: Style) -> ResolvedNotation {
        Text(style, bound)
    }

    fn child(i: usize, bound: Bound) -> ResolvedNotation {
        Child(i, bound)
    }
}
