use std::cmp;
use std::fmt;

use super::boundset::BoundSet;
use crate::geometry::{Bound, Col, Pos, Region};
use crate::notation::Notation;
use crate::style::Style;

use self::Layout::*;

pub trait Lay
where
    Self: Clone,
{
    fn empty() -> Self;
    fn literal(s: &str) -> Self;
    fn nest(&self, other: Self) -> Self;
    fn vert(&self, other: Self) -> Self;
}

impl Lay for Bound {
    fn empty() -> Bound {
        Bound {
            width: 0,
            height: 1,
            indent: 0,
        }
    }

    fn literal(s: &str) -> Bound {
        let width = s.chars().count() as Col;
        Bound {
            width: width,
            indent: width,
            height: 1,
        }
    }

    fn nest(&self, other: Bound) -> Bound {
        Bound {
            width: cmp::max(self.width, self.indent + other.width),
            height: self.height + other.height - 1,
            indent: self.indent + other.indent,
        }
    }

    fn vert(&self, other: Bound) -> Bound {
        Bound {
            width: cmp::max(self.width, other.width),
            height: self.height + other.height,
            indent: other.indent,
        }
    }
}

/// A concrete plan for how to lay out a `Notation`, once the program
/// and screen width are known.
pub struct Layout {
    elements: Vec<(LayoutElement, Region)>,
    children: Vec<Option<Region>>,
}

/// Part of a [`Layout`](Layout): one thing to be written to the screen.
#[derive(Clone, PartialEq, Eq)]
pub enum LayoutElement {
    /// Display a literal string with the given style.
    Literal(String, Style),
    /// Display a text node's text with the given style.
    Text(Style),
    /// Display a child node. Its Bound must be supplied.
    Child(usize),
}

impl Layout {
    pub fn child(&self, i: usize) -> Option<Region> {
        self.children[i].clone()
    }
}






qqq


fn lay<L: Lay>(
    child_bounds: &[Bounds],
    memo: Option<&BoundSet<L>>,
    notation: &Notation,
) -> BoundSet<L> {
    match notation {
        Notation::Empty => BoundSet::singleton(Bound::empty(), L::empty()),
        Notation::Literal(s, style) => {
            BoundSet::singleton(Bound::literal(s, *style), L::literal(s, *style))
        }
        Notation::Text(style) => child_bounds[0]
            .0
            .into_iter()
            .map(|(bound, ())| (bound, L::text(bound, *style)))
            .collect(),
        Notation::Child(index) => child_bounds[*index]
            .0
            .into_iter()
            .map(|(bound, ())| (bound, L::child(*index, bound)))
            .collect(),
        Notation::Concat(note1, note2) => BoundSet::combine(
            &lay(child_bounds, memo, note1),
            &lay(child_bounds, memo, note2),
            |b1, b2| b1.concat(b2),
            |v1, v2| v1.concat(v2),
        ),
        Notation::Vert(note1, note2) => BoundSet::combine(
            &lay(child_bounds, memo, note1),
            &lay(child_bounds, memo, note2),
            |b1, b2| b1.vert(b2),
            |v1, v2| v1.vert(v2),
        ),
        Notation::NoWrap(note) => {
            let set = lay(child_bounds, memo, note);
            set.into_iter()
                .filter(|(bound, _)| bound.height == 1)
                .collect()
        }
        Notation::Choice(note1, note2) => {
            let set1 = lay(child_bounds, memo, note1);
            let set2 = lay(child_bounds, memo, note2);
            set1.into_iter().chain(set2.into_iter()).collect()
        }
        Notation::WithMemoized(note1, note2) => {
            let bounds = lay(child_bounds, memo, note1);
            lay(child_bounds, Some(&bounds), note2)
        }
        Notation::Memoized => match memo {
            None => panic!("lay_out: unexpected Memoized"),
            Some(boundset) => boundset.clone(),
        },
        Notation::IfEmptyText(_, _) => panic!("lay_out: unexpected IfEmptyText"),
        Notation::Rep(_) => panic!("lay_out: unexpected Repeat"),
    }
}


    
impl Lay for LayoutRegion {
    fn empty() -> LayoutRegion {
        LayoutRegion {
            region: Region {
                pos: Pos::zero(),
                bound: Bound::empty(),
            },
            layout: Layout::Empty,
        }
    }

    fn literal(s: &str, style: Style) -> LayoutRegion {
        LayoutRegion {
            region: Region {
                pos: Pos::zero(),
                bound: Bound::literal(&s, style),
            },
            layout: Layout::Literal(s.to_string(), style),
        }
    }

    fn concat(&self, other: LayoutRegion) -> LayoutRegion {
        let self_lay = self.clone();
        let mut other_lay = other.clone();
        other_lay.shift_by(self.region.end());
        LayoutRegion {
            region: Region {
                pos: self.region.pos,
                bound: self.region.bound.concat(other.region.bound),
            },
            layout: Layout::Concat(Box::new(self_lay), Box::new(other_lay)),
        }
    }

    fn vert(&self, other: LayoutRegion) -> LayoutRegion {
        let self_lay = self.clone();
        let mut other_lay = other.clone();
        let delta = Pos {
            row: self.region.height(),
            col: 0,
        };
        other_lay.shift_by(delta);
        LayoutRegion {
            region: Region {
                pos: self.region.pos,
                bound: self.region.bound.vert(other.region.bound),
            },
            layout: Layout::Vert(Box::new(self_lay), Box::new(other_lay)),
        }
    }

    fn text(child: Bound, style: Style) -> LayoutRegion {
        LayoutRegion {
            region: Region {
                pos: Pos::zero(),
                bound: Bound::text(child, style),
            },
            layout: Layout::Text(style),
        }
    }

    fn child(i: usize, child: Bound) -> LayoutRegion {
        LayoutRegion {
            region: Region {
                pos: Pos::zero(),
                bound: Bound::child(i, child),
            },
            layout: Layout::Child(i),
        }
    }
}

/// Precomputed information that helps pretty-print a document.
#[derive(Clone)]
pub struct Bounds(BoundSet<()>);

impl Bounds {
    pub fn empty() -> Bounds {
        Bounds(BoundSet::new())
    }

    #[cfg(test)]
    pub(crate) fn fit_width(&self, width: Col) -> Bound {
        self.0.fit_width(width).0
    }
}

impl Layouts {
    #[cfg(test)]
    pub(crate) fn fit_width(&self, width: Col) -> LayoutRegion {
        self.0.fit_width(width).1
    }

    pub fn fit_bound(&self, bound: Bound) -> LayoutRegion {
        self.0.fit_bound(bound).1
    }

    pub fn fit_region(&self, region: Region) -> LayoutRegion {
        let mut lay = self.0.fit_bound(region.bound).1;
        lay.shift_by(region.pos);
        lay
    }
}

#[derive(Clone)]
pub struct Layouts(BoundSet<LayoutRegion>);

// If the node is texty, `child_bounds` should be a singleton vec of the text bounds.
pub fn compute_layouts(child_bounds: &[Bounds], notation: &Notation) -> Layouts {
    Layouts(lay(child_bounds, None, notation))
}

// If the node is texty, `child_bounds` should be a singleton vec of the text bounds.
pub fn compute_bounds(child_bounds: &[Bounds], notation: &Notation) -> Bounds {
    Bounds(lay(child_bounds, None, notation))
}

pub fn text_bounds(text: &str) -> Bounds {
    Bounds(BoundSet::singleton(
        Bound::literal(text, Style::plain()),
        (),
    ))
}
