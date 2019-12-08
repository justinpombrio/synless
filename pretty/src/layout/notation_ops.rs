use super::boundset::BoundSet;
use super::staircase::Staircase;
use crate::geometry::{Bound, Col};
use crate::style::Style;
use std::cmp;
use std::fmt::Debug;

use typed_arena::Arena;

pub trait NotationOps
where
    Self: Clone + Debug,
{
    /// The trait method implementations can optionally use a custom allocator
    /// (eg. an Arena) to create new instances of Self, to improve performance.
    type Allocator: Copy;

    fn empty(allocator: Self::Allocator) -> Self;
    fn literal(s: &str, style: Style, allocator: Self::Allocator) -> Self;
    // TODO: Make these into vecs
    fn follow(left: Self, right: Self, allocator: Self::Allocator) -> Self;
    fn vert(left: Self, right: Self, allocator: Self::Allocator) -> Self;
    fn text(child: Bound, style: Style, allocator: Self::Allocator) -> Self;
    fn child(i: usize, child: Bound, allocator: Self::Allocator) -> Self;
}

impl NotationOps for () {
    type Allocator = ();

    fn empty(_allocator: ()) {}
    fn literal(_s: &str, _style: Style, _allocator: ()) {}
    fn follow(_left: (), _right: (), _allocator: ()) {}
    fn vert(_left: (), _right: (), _allocator: ()) {}
    fn text(_child: Bound, _style: Style, _allocator: ()) {}
    fn child(_i: usize, _child: Bound, _allocator: ()) {}
}

impl NotationOps for Bound {
    type Allocator = ();

    fn empty(_allocator: Self::Allocator) -> Bound {
        Bound {
            width: 0,
            height: 1,
            indent: 0,
        }
    }

    fn literal(string: &str, _style: Style, _allocator: Self::Allocator) -> Bound {
        let width = string.chars().count() as Col;
        Bound {
            width,
            indent: width,
            height: 1,
        }
    }

    fn follow(b1: Bound, b2: Bound, _allocator: Self::Allocator) -> Bound {
        Bound {
            width: cmp::max(b1.width, b1.indent + b2.width),
            height: b1.height + b2.height - 1,
            indent: b1.indent + b2.indent,
        }
    }

    fn vert(b1: Bound, b2: Bound, _allocator: Self::Allocator) -> Bound {
        Bound {
            width: cmp::max(b1.width, b2.width),
            height: b1.height + b2.height,
            indent: b2.indent,
        }
    }

    fn text(child: Bound, _style: Style, _allocator: Self::Allocator) -> Bound {
        child
    }

    fn child(_i: usize, child: Bound, _allocator: Self::Allocator) -> Bound {
        child
    }
}

impl<T: NotationOps> NotationOps for BoundSet<T> {
    /// We'll need to allocate new copies of whatever data is stored inside the
    /// BoundSet.
    type Allocator = T::Allocator;

    fn empty(allocator: Self::Allocator) -> BoundSet<T> {
        BoundSet::singleton(Bound::empty(()), T::empty(allocator))
    }

    fn literal(string: &str, style: Style, allocator: Self::Allocator) -> BoundSet<T> {
        BoundSet::singleton(
            Bound::literal(string, style, ()),
            T::literal(string, style, allocator),
        )
    }

    fn follow(set1: BoundSet<T>, set2: BoundSet<T>, allocator: Self::Allocator) -> BoundSet<T> {
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
                    set.insert(
                        Bound::follow(b1, b2, ()),
                        T::follow(v1.clone(), v2.clone(), allocator),
                    );
                }
            }
        }
        // Consider the case where the bound from set2 extends further to the
        // right than the bound from set1.
        for (b2, v2) in set2 {
            for staircase1 in set1.staircases() {
                let space_available = staircase1.indent() + b2.width;
                if let Some((b1, v1)) = staircase1.fit_width(space_available) {
                    set.insert(
                        Bound::follow(b1, b2, ()),
                        T::follow(v1.clone(), v2.clone(), allocator),
                    );
                }
            }
        }
        set
    }

    fn vert(set1: BoundSet<T>, set2: BoundSet<T>, allocator: Self::Allocator) -> BoundSet<T> {
        let mut set = BoundSet::new();
        // The indent of Bounds in set1 is irrelevant.
        let mut staircase1 = Staircase::new(0);
        for (b1, v1) in set1 {
            staircase1.insert(b1.width, b1.height, v1);
        }
        for (b2, v2) in set2 {
            for (b1, v1) in &staircase1 {
                set.insert(
                    Bound::vert(b1, b2, ()),
                    T::vert(v1.clone(), v2.clone(), allocator),
                );
            }
        }
        set
    }

    fn text(child: Bound, style: Style, allocator: Self::Allocator) -> BoundSet<T> {
        BoundSet::singleton(child, T::text(child, style, allocator))
    }

    fn child(i: usize, child: Bound, allocator: Self::Allocator) -> BoundSet<T> {
        BoundSet::singleton(child, T::child(i, child, allocator))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedNotation<'a> {
    Empty,
    Literal(String, Style, Bound),
    Follow(&'a ResolvedNotation<'a>, &'a ResolvedNotation<'a>),
    Vert(&'a ResolvedNotation<'a>, &'a ResolvedNotation<'a>),
    Text(Style, Bound),
    Child(usize, Bound),
}

impl<'a> NotationOps for &'a ResolvedNotation<'a> {
    type Allocator = &'a Arena<ResolvedNotation<'a>>;

    fn empty(allocator: Self::Allocator) -> Self {
        allocator.alloc(ResolvedNotation::Empty)
    }

    fn literal(string: &str, style: Style, allocator: Self::Allocator) -> &'a ResolvedNotation<'a> {
        allocator.alloc(ResolvedNotation::Literal(
            string.to_string(),
            style,
            Bound::literal(string, style, ()),
        ))
    }

    fn follow(
        left: &'a ResolvedNotation<'a>,
        right: &'a ResolvedNotation<'a>,
        allocator: Self::Allocator,
    ) -> &'a ResolvedNotation<'a> {
        allocator.alloc(ResolvedNotation::Follow(left, right))
    }

    fn vert(
        left: &'a ResolvedNotation<'a>,
        right: &'a ResolvedNotation<'a>,
        allocator: Self::Allocator,
    ) -> &'a ResolvedNotation<'a> {
        allocator.alloc(ResolvedNotation::Vert(left, right))
    }

    fn text(bound: Bound, style: Style, allocator: Self::Allocator) -> &'a ResolvedNotation<'a> {
        allocator.alloc(ResolvedNotation::Text(style, bound))
    }

    fn child(i: usize, bound: Bound, allocator: Self::Allocator) -> &'a ResolvedNotation<'a> {
        allocator.alloc(ResolvedNotation::Child(i, bound))
    }
}
