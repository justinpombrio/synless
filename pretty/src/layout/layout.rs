use std::cmp;
use std::fmt;

use super::boundset::BoundSet;
use crate::geometry::{Bound, Col, Pos, Region};
use crate::style::Style;
use super::compile_notation::{compute_bounds, Bounds, CompiledNotation};

use self::Layout::*;

#[derive(Debug, Clone)]
/// A concrete plan for how to lay out a `Notation`, once the program
/// and screen width are known.
pub struct Layout {
    elements: Vec<LayoutElement>,
    children: Vec<Option<LayoutElement>>,
}

/// Lay out a node in preparation for rendering it. Doing this requires knowing
/// (i) the Bounds of its children, (ii) if it is a text node, whether its text
/// is empty or not, (iii) the Notation with which it is being displayed, and
/// (iv) the available screen width.
///
/// If the node is not texty, then `is_empty_text` will not be used and can have
/// any value.
pub fn compute_layout(
    child_bounds: &[Bounds],
    is_empty_text: bool,
    notation: &Notation,
    width: Col,
) -> Layout {
    unimplemented!()
}

/// Part of a [`Layout`](Layout): one thing to be written to the screen.
#[derive(Clone, PartialEq, Eq)]
pub enum LayoutElement {
    /// Display a literal string with the given style.
    Literal(Region, String, Style),
    /// Display a text node's text with the given style.
    Text(Region, Style),
    /// Display a child node. Its Bound must be supplied.
    Child(Region, usize),
}

struct ComputeLayout<'a> {
    layout: Layout,
    child_bounds: &'a [Bounds],
    is_empty_text: bool,
}

impl<'a> ComputeLayout<'a> {
    fn new(child_bounds: &'a [Bounds], is_empty_text: bool) -> ComputeLayout<'a> {
        ComputeLayout {
            layout: Layout {
                elements: vec![],
                children: unimplemented!(),
            },
            child_bounds,
            is_empty_text,
        }
    }

    fn lay(
        &mut self,
        pos: Pos,
        width: Col,
        notation: &Notation,
        in_join: Option<(Bound, Bound)>,
        in_surround: Option<Bound>,
    ) -> Bound {
        let recur = |pos, width, notation| {
            self.lay(pos, width, notation, in_join.clone(), in_surround.clone())
        };
        match notation {
            Empty => Bound::empty(),
            Literal(s, _) => Bound::literal(s),
            Text(_) => self.child_bounds[0].clone(),
            Child(i) => self.child_bounds[i].clone(),
            Nest(notations) => {
                let mut pos = pos;
                let mut width = width;
                let mut total_bound = Bound::empty();
                for notation in notations {
                    let bound = recur(pos, width, notation);
                    pos += bound.end();
                    width = width - bound.indent;
                    total_bound = Bound::nest(total_bound, bound);
                }
                total_bound
            }
            Vert(notations) => {
                let mut pos = pos;
                let mut total_bound = Bound::empty();
                for notation in notations {
                    let bound = recur(pos, width, notation);
                    pos.row += bound.height;
                    total_bound = Bound::vert(total_bound, bound);
                }
                total_bound
            }
            IfFlat(notation1, notation2) => {
                // Unfortunately, we have no pre-computed information about this
                // _particular_ Notation (which may be in the middle of a larger
                // Notation). All we need here is the flat_width, but if we
                // compute that and then recur we're doing quadratic work in the
                // worst case. So compute the full layout here, in case we need it.
                // We have to make a separate `ComputeLayout`, though, in case
                // we don't use it, to prevent `self.layout` from being contaminated.
                let mut computer = ComputeLayout::new(self.child_bounds, self.is_empty_text);
                let bound1 = computer.lay(pos, width, notation1, in_join, in_surround);
                if bound1.height > 1 {
                    self.layout.elements.append(&mut computer.layout.elements);
                    for i in 0..self.child_bounds.len() {
                        self.layout.children[i] = computer.layout.children[i];
                    }
                    self.layout.elements.append(&mut computer.layout.elements);
                } else {
                    recur(pos, width, notation2)
                }
            }
            IfEmptyText(notation1, notation2) => {
                if self.is_empty_text {
                    recur(pos, width, notation1)
                } else {
                    recur(pos, width, notation2)
                }
            }
            Repeat(box RepeatInner {
                empty,
                lone,
                join,
                surround,
            }) => match self.children.len() {
                0 => self.lay(pos, width, empty, None, None),
                1 => self.lay(pos, width, lone, None, None),
                _ => {
                    let mut total_bound = Bound::empty();
                    for child in self.child_bounds {
                        let in_join = Some((total_bound, child.clone()));
                        // THINK: how does this work?
                        // Inside out?
                        total_bound = self.lay(pos, width, join, in_join, None);
                    }
                }
            },
            Left => in_join.expect("invalid Left").0,
            Right => in_join.expect("invalid Right").1,
            Surrounded => in_surround.expect("invalid Surrounded"),

            Repeat(box RepeatInner {
                empty,
                lone,
                join,
                surround,
            }) => match self.children.len() {
                0 => self.lay(pos, width, empty, None, None),
                1 => self.lay(pos, width, lone, None, None),
                _ => {
                    self.lay(pos, width, surround, 
                    let mut lay = T::empty();
                    for child in self.children {
                        let in_join = Some((total_bound, child.clone()));
                        self.lay(pos, width, join, in_surround, None);
                        lay = self.apply(join, Some((lay, child.clone())), None);
                    }
                    self.apply(surround, None, Some(lay))
                }
            },
        }
    }
}

/*
impl<L: Lay> PerformLay<L> {
    fn new(children: &[L]) -> PerformLay<L> {
        PerformLay {
            children,
        }
    }

    fn lay(&self, notation: &Notation, in_join: Option((L, L)), in_surround: Option<L>) -> L {
        }
    }
}





impl Layout {
    pub fn child(&self, i: usize) -> Option<Region> {
        self.children[i].clone()
    }
}

fn lay_out(child_bounds: &[Bounds]

fn lay<L: Lay>(
    child_bounds: &[Bounds],
    memo: Option<&BoundSet<L>>,
    notation: &Notation,
) -> BoundSet<L> {





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
*/
