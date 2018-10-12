use std::cmp;
use std::fmt;

use common::{Col, Pos};
use style::Style;
use super::super::{Bound, Region, Syntax};
use super::BoundSet;
use self::Layout::*;


pub trait Layable where Self: Clone {
    fn empty() -> Self;
    fn literal(s: &str, style: Style) -> Self;
    fn flush(&self) -> Self;
    fn concat(&self, other: Self) -> Self;
    fn text(child: Bound, style: Style) -> Self;
    fn child(i: usize, child: Bound) -> Self;
}


impl Layable for () {
    fn empty()                            {}
    fn literal(_s: &str, _style: Style)   {}
    fn flush(&self)                       {}
    fn concat(&self, _other: ())          {}
    fn text(_child: Bound, _style: Style) {}
    fn child(_i: usize, _child: Bound)    {}
}


impl Layable for Bound {
    fn empty() -> Bound {
        Bound {
            width: 0,
            height: 0,
            indent: 0
        }
    }

    fn literal(s: &str, _style: Style) -> Bound {
        let width = s.chars().count() as Col;
        Bound {
            width:  width,
            indent: width,
            height: 0
        }
    }

    fn flush(&self) -> Bound {
        Bound {
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

    fn text(child: Bound, _style: Style) -> Bound {
        child
    }

    fn child(_i: usize, child: Bound) -> Bound {
        child
    }
}


/// A concrete plan for how to lay out the `Syntax`, once the program
/// and screen width are known.  For example, unlike in `Syntax`,
/// there is no Choice, because the choices have been resolved.
/// The outermost region always has position zero, but inner regions
/// are relative to this.
#[derive(Clone, PartialEq, Eq)]
pub struct LayoutRegion {
    pub layout: Layout,
    pub region: Region
}

/// The enum for a LayoutRegion.
#[derive(Clone, PartialEq, Eq)]
pub enum Layout {
    /// Display nothing.
    Empty,
    /// Display a literal string with the given style.
    Literal(String, Style),
    /// Display a text node's text with the given style.
    Text(Style),
    /// Display the layout, then a newline.
    Flush(Box<LayoutRegion>),
    /// Display the concatenation of the two layouts.
    /// The `Col` is the indent on the Bound of the first Layout.
    /// (This is redundant information, but convenient to have around.)
    Concat(Box<LayoutRegion>, Box<LayoutRegion>),
    /// Display a child node. Its Bound must be supplied.
    Child(usize)
}

impl LayoutRegion {
    fn shift_to(&mut self, pos: Pos) {
        self.region.pos = pos;
        self.layout.shift_to(pos);
    }
}

impl Layout {
    fn shift_to(&mut self, pos: Pos) {
        match self {
            Empty         => (),
            Literal(_, _) => (),
            Text(_)       => (),
            Flush(box lay) => lay.shift_to(pos),
            Concat(box lay1, box lay2) => {
                let delta = lay1.region.delta();
                lay1.shift_to(pos);
                lay2.shift_to(pos + delta);
            }
            Child(_) => ()
        }
    }
}

impl fmt::Debug for LayoutRegion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let indent = self.region.pos.col;
        let bound = &self.region.bound;
        match &self.layout {
            Empty => {
                Ok(())
            }
            Literal(ref s, _) => {
                write!(f, "{}", s)
            }
            Text(_) => {
                bound.debug_print(f, 't', indent)
            }
            Flush(ref lay)  => {
                write!(f, "{:?}\n", lay)?;
                write!(f, "{}", " ".repeat(indent as usize))
            }
            Child(index) => {
                let ch = format!("{}", index).pop().unwrap();
                bound.debug_print(f, ch, indent)
            }
            Concat(ref lay1, ref lay2) => {
                write!(f, "{:?}{:?}", lay1, lay2)
            }
        }
    }
}

impl Layable for LayoutRegion {
    fn empty() -> LayoutRegion {
        LayoutRegion {
            region: Region {
                pos:   Pos::zero(),
                bound: Bound::empty()
            },
            layout: Layout::Empty
        }
    }

    fn literal(s: &str, style: Style) -> LayoutRegion {
        LayoutRegion {
            region: Region {
                pos:   Pos::zero(),
                bound: Bound::literal(&s, style)
            },
            layout: Layout::Literal(s.to_string(), style)
        }
    }

    fn flush(&self) -> LayoutRegion {
        let self_lay = self.clone();
        LayoutRegion {
            region: Region {
                pos:   self.region.pos,
                bound: self.region.bound.flush()
            },
            layout: Layout::Flush(Box::new(self_lay))
        }
    }

    fn concat(&self, other: LayoutRegion) -> LayoutRegion {
        let self_lay = self.clone();
        let mut other_lay = other.clone();
        other_lay.shift_to(self.region.end());
        LayoutRegion {
            region: Region {
                pos:   self.region.pos,
                bound: self.region.bound.concat(other.region.bound)
            },
            layout: Layout::Concat(Box::new(self_lay), Box::new(other_lay))
        }
    }

    fn text(child: Bound, style: Style) -> LayoutRegion {
        LayoutRegion {
            region: Region {
                pos:   Pos::zero(),
                bound: Bound::text(child, style)
            },
            layout: Layout::Text(style)
        }
    }

    fn child(i: usize, child: Bound) -> LayoutRegion {
        LayoutRegion {
            region: Region {
                pos:   Pos::zero(),
                bound: Bound::child(i, child)
            },
            layout: Layout::Child(i)
        }
    }
}


struct LayOuter<'a> {
    child_bounds: Vec<&'a BoundSet<()>>
}

impl<'a> LayOuter<'a> {
    fn lay_out<L>(&self, syntax: &Syntax) -> BoundSet<L>
        where L: Layable
    {
        match syntax {
            Syntax::Empty => {
                BoundSet::singleton(Bound::empty(),
                                    L::empty())
            }
            Syntax::Literal(s, style) => {
                BoundSet::singleton(Bound::literal(s, *style),
                                    L::literal(s, *style))
            }
            Syntax::Text(style) => {
                self.child_bounds[0].into_iter().map(|(bound, ())| {
                    (bound, L::text(bound, *style))
                }).collect()
            }
            Syntax::Child(index) => {
                self.child_bounds[*index].into_iter().map(|(bound, ())| {
                    (bound, L::child(*index, bound))
                }).collect()
            }
            Syntax::Flush(syn) => {
                let set = self.lay_out(syn);
                set.into_iter().map(|(bound, val): (Bound, L)| {
                    (bound.flush(), val.flush())
                }).collect()
            }
            Syntax::Concat(syn1, syn2) => {
                let set1: BoundSet<L> = self.lay_out(syn1);
                let set2: BoundSet<L> = self.lay_out(syn2);

                let mut set = BoundSet::new();
                for (bound1, val1) in set1.into_iter() {
                    for (bound2, val2) in set2.into_iter() {
                        let bound = bound1.concat(bound2);
                        let val = val1.concat(val2);
                        set.insert(bound, val);
                    }
                }
                set
            }
            Syntax::NoWrap(syn) => {
                let set = self.lay_out(syn);
                set.into_iter().filter(|(bound, _)| {
                    bound.height == 0
                }).collect()
            }
            Syntax::Choice(syn1, syn2) => {
                let set1 = self.lay_out(syn1);
                let set2 = self.lay_out(syn2);
                set1.into_iter().chain(set2.into_iter()).collect()
            }
            Syntax::IfEmptyText(_, _) => panic!("lay_out: unexpected IfEmptyText"),
            Syntax::Rep(_) => panic!("lay_out: unexpected Repeat"),
            Syntax::Star   => panic!("lay_out: unexpected Star")
        }
    }
}


impl Syntax {
    /// Compute the possible Layouts for this `Syntax`, given
    /// information about its children.
    pub fn lay_out(
        &self,
        arity: usize,
        child_bounds: Vec<&BoundSet<()>>,
        is_empty_text: bool)
        -> BoundSet<LayoutRegion>
    {
        let stx = self.expand(arity, child_bounds.len(), is_empty_text);
        let lay_outer = LayOuter {
            child_bounds: child_bounds
        };
        lay_outer.lay_out(&stx)
    }

    /// Precompute the Bounds within which this `Syntax` can be
    /// displayed, given information about its children.
    pub fn bound(
        &self,
        arity: usize,
        child_bounds: Vec<&BoundSet<()>>,
        is_empty_text: bool)
        -> BoundSet<()>
    {
        let stx = self.expand(arity, child_bounds.len(), is_empty_text);
        let lay_outer = LayOuter {
            child_bounds: child_bounds
        };
        lay_outer.lay_out(&stx)
    }
}
