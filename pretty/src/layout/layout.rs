use std::cmp;
use std::fmt;

use super::boundset::BoundSet;
use crate::geometry::{Col, Pos, Bound, Region};
use crate::style::Style;
use crate::notation::Notation;

use self::Layout::*;


pub trait Lay where Self: Clone {
    fn empty() -> Self;
    fn literal(s: &str, style: Style) -> Self;
    fn concat(&self, other: Self) -> Self;
    fn horz(&self, other: Self) -> Self;
    fn vert(&self, other: Self) -> Self;
    fn text(child: Bound, style: Style) -> Self;
    fn child(i: usize, child: Bound) -> Self;
}


impl Lay for () {
    fn empty()                            {}
    fn literal(_s: &str, _style: Style)   {}
    fn concat(&self, _other: ())          {}
    fn horz(&self, _other: ())            {}
    fn vert(&self, _other: ())            {}
    fn text(_child: Bound, _style: Style) {}
    fn child(_i: usize, _child: Bound)    {}
}


impl Lay for Bound {
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
    
    fn concat(&self, other: Bound) -> Bound {
        Bound {
            width:  cmp::max(self.width,
                             self.indent + other.width),
            height: self.height + other.height,
            indent: self.indent + other.indent
        }
    }

    fn horz(&self, other: Bound) -> Bound {
        Bound {
            width: self.width + other.width,
            height: cmp::max(self.height, other.height),
            indent: if self.height > other.height {
                self.indent
            } else {
                self.width + other.indent
            }
        }
    }

    fn vert(&self, other: Bound) -> Bound {
        Bound {
            width:  cmp::max(self.width, other.width),
            height: self.height + other.height + 1,
            indent: other.indent
        }
    }

    fn text(child: Bound, _style: Style) -> Bound {
        child
    }

    fn child(_i: usize, child: Bound) -> Bound {
        child
    }
}


/// A concrete plan for how to lay out the `Notation`, once the program
/// and screen width are known.  For example, unlike in `Notation`,
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
    /// Display the standard concatenation of the two layouts.
    Concat(Box<LayoutRegion>, Box<LayoutRegion>),
    /// Display the horizontal concatenation of the two layouts.
    Horz(Box<LayoutRegion>, Box<LayoutRegion>),
    /// Display the vertical concatenation of the two layouts.
    Vert(Box<LayoutRegion>, Box<LayoutRegion>),
    /// Display a child node. Its Bound must be supplied.
    Child(usize)
}

// TODO: This is inefficient. Remove `shift_by`.
impl LayoutRegion {
    pub fn shift_by(&mut self, pos: Pos) {
        self.region.pos = self.region.pos + pos;
        self.layout.shift_by(pos);
    }
}

impl Layout {
    fn shift_by(&mut self, pos: Pos) {
        match self {
            Empty         => (),
            Literal(_, _) => (),
            Text(_)       => (),
            Concat(box lay1, box lay2) => {
                lay1.shift_by(pos);
                lay2.shift_by(pos);
            }
            Horz(box lay1, box lay2) => {
                lay1.shift_by(pos);
                lay2.shift_by(pos);
            }
            Vert(box lay1, box lay2) => {
                lay1.shift_by(pos);
                lay2.shift_by(pos);
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
            Child(index) => {
                let ch = format!("{}", index).pop().unwrap();
                bound.debug_print(f, ch, indent)
            }
            Concat(ref lay1, ref lay2) => {
                write!(f, "{:?}{:?}", lay1, lay2)
            }
            Horz(ref lay1, ref lay2) => {
                write!(f, "HORZ\n{:?}\n\n{:?}", lay1, lay2)
            }
            Vert(ref lay1, ref lay2) => {
                let indent_str = " ".repeat(indent as usize);
                write!(f, "{:?}\n{}{:?}", lay1, indent_str, lay2)
            }
        }
    }
}

impl Lay for LayoutRegion {
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

    fn concat(&self, other: LayoutRegion) -> LayoutRegion {
        let self_lay = self.clone();
        let mut other_lay = other.clone();
        other_lay.shift_by(self.region.end());
        LayoutRegion {
            region: Region {
                pos:   self.region.pos,
                bound: self.region.bound.concat(other.region.bound)
            },
            layout: Layout::Concat(Box::new(self_lay), Box::new(other_lay))
        }
    }

    fn horz(&self, other: LayoutRegion) -> LayoutRegion {
        let self_lay = self.clone();
        let mut other_lay = other.clone();
        let delta = Pos {
            row: 0,
            col: self.region.width()
        };
        other_lay.shift_by(delta);
        LayoutRegion {
            region: Region {
                pos:   self.region.pos,
                bound: self.region.bound.horz(other.region.bound)
            },
            layout: Layout::Horz(Box::new(self_lay), Box::new(other_lay))
        }
    }

    fn vert(&self, other: LayoutRegion) -> LayoutRegion {
        let self_lay = self.clone();
        let mut other_lay = other.clone();
        let delta = Pos {
            row: self.region.height() + 1,
            col: 0
        };
        other_lay.shift_by(delta);
        LayoutRegion {
            region: Region {
                pos:   self.region.pos,
                bound: self.region.bound.vert(other.region.bound)
            },
            layout: Layout::Vert(Box::new(self_lay), Box::new(other_lay))
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

#[derive(Clone)]
pub struct Bounds(BoundSet<()>);

impl Bounds {
    pub fn empty() -> Bounds {
        Bounds(BoundSet::new())
    }

    #[cfg(test)]
    pub(crate) fn first(&self) -> Bound {
        self.0.first().0
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
}

#[derive(Clone)]
pub struct Layouts(BoundSet<LayoutRegion>);

// If the node is texty, `child_bounds` should be a singleton vec of the text bounds.
pub fn compute_layouts(child_bounds: &Vec<Bounds>, notation: &Notation) -> Layouts {
    Layouts(lay(child_bounds, notation))
}

// If the node is texty, `child_bounds` should be a singleton vec of the text bounds.
pub fn compute_bounds(child_bounds: &Vec<Bounds>, notation: &Notation) -> Bounds {
    Bounds(lay(child_bounds, notation))
}

pub fn text_bounds(text: &str) -> Bounds {
    Bounds(BoundSet::singleton(Bound::literal(text, Style::plain()), ()))
}

fn lay<L: Lay>(child_bounds: &Vec<Bounds>, notation: &Notation) -> BoundSet<L> {
    match notation {
        Notation::Empty => {
            BoundSet::singleton(Bound::empty(),
                                L::empty())
        }
        Notation::Literal(s, style) => {
            BoundSet::singleton(Bound::literal(s, *style),
                                L::literal(s, *style))
        }
        Notation::Text(style) => {
            child_bounds[0].0.into_iter().map(|(bound, ())| {
                (bound, L::text(bound, *style))
            }).collect()
        }
        Notation::Child(index) => {
            child_bounds[*index].0.into_iter().map(|(bound, ())| {
                (bound, L::child(*index, bound))
            }).collect()
        }
        Notation::Concat(note1, note2) => {
            BoundSet::combine(&lay(child_bounds, note1),
                              &lay(child_bounds, note2),
                              |b1, b2| b1.concat(b2),
                              |v1, v2| v1.concat(v2))
        }
        Notation::Horz(note1, note2) => {
            BoundSet::combine(&lay(child_bounds, note1),
                              &lay(child_bounds, note2),
                              |b1, b2| b1.horz(b2),
                              |v1, v2| v1.horz(v2))
        }
        Notation::Vert(note1, note2) => {
            BoundSet::combine(&lay(child_bounds, note1),
                              &lay(child_bounds, note2),
                              |b1, b2| b1.vert(b2),
                              |v1, v2| v1.vert(v2))
        }
        Notation::NoWrap(note) => {
            let set = lay(child_bounds, note);
            set.into_iter().filter(|(bound, _)| {
                bound.height == 0
            }).collect()
        }
        Notation::Choice(note1, note2) => {
            let set1 = lay(child_bounds, note1);
            let set2 = lay(child_bounds, note2);
            set1.into_iter().chain(set2.into_iter()).collect()
        }
        Notation::IfEmptyText(_, _) => panic!("lay_out: unexpected IfEmptyText"),
        Notation::Rep(_) => panic!("lay_out: unexpected Repeat"),
        Notation::Star   => panic!("lay_out: unexpected Star")
    }
}
