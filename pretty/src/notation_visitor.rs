use crate::notation::{Notation, RepeatInner};
use Notation::*;

pub trait NotationVisitor {
    type Arg;
    type Result;

    fn empty(&mut self, arg: Arg) -> Self::Result;
    fn literal(&mut self, arg: Arg, s: &str) -> Self::Result;
    fn child(&mut self, arg: Arg, child_index: usize) -> Self::Result;
    fn nest(&mut self, arg: Arg, left: Self::Result, right: Self::Result) -> Self::Result;
    fn vert(&mut self, arg: Arg, left: Self::Result, right: Self::Result) -> Self::Result;
    fn if_flat(&mut self, arg: Arg, left: Self::Result, right: Self::Result) -> Self::Result;
}

struct ComputeLayout<'a> {
    layout: Layout,
    child_bounds: &'a [Bounds],
    is_empty_text: bool,
}

impl<'a> NotationVisitor for ComputeLayout<'a> {
    type Arg = (Pos, Col);
    type Result = Bound;
    
    fn empty(&mut self, _) -> Bound {
        Bound::empty()
    }

    fn literal(&mut self, _, s: &str) -> Bound {
        Bound::literal(s)
    }

    
}

impl Notation {
    fn visit<V: NotationVisitor>(&self, visitor: &mut V, arg: V::Arg) -> V::Result {
        match self {
            Empty => visitor.empty(arg),
            Literal(s, _) => visitor.literal(arg, s),
            Text(_) => visitor.child(arg, 0),
            Child(i) => visitor.child(arg, *i),
            Nest(notations) => {
                let notations = notations.iter();
                let notation = notations.next().expect("Unnormalized Notation");
                let mut result = notation.visit(visitor
            }
            
            Nest(notations) => {
                let mut lay = T::empty();
                for notation in notations {
                    lay = self.apply(notation, in_join, in_surround, &|b| T::nest(&lay, b));
                }
                func(&lay)
            }
            
        }
    }
}

pub trait NotationOps: Clone {
    fn empty() -> Self;
    fn literal(s: &str) -> Self;
    fn nest<'a>(left: &'a Self, right: &'a Self) -> Self;
    fn vert<'a>(left: &'a Self, right: &'a Self) -> Self;
    fn if_flat(left: &Self, right: &Self) -> Self;
}

pub fn apply_notation<T: NotationOps>(
    children: &[T],
    is_empty_text: bool,
    notation: &Notation,
) -> T {
    ApplyNotation::new(children, is_empty_text).apply(&notation, None, None, &|b| b.clone())
}

struct ApplyNotation<'a, T: NotationOps> {
    children: &'a [T],
    is_empty_text: bool,
    surrounded: Option<T>,
}

impl<T: NotationOps> ApplyNotation<'_, T> {
    fn new(children: &[T], is_empty_text: bool) -> ApplyNotation<T> {
        ApplyNotation {
            children,
            is_empty_text,
            surrounded: None,
        }
    }

    fn apply(&self, notation: &Notation, in_join: Option<(&T, &T)>, in_surround: Option<&T>) {
        match notation {
            Empty => func(&T::empty()),
            Literal(s, _) => func(&T::literal(s)),
            Text(_) => func(&self.children[0]),
            Child(i) => func(&self.children[*i]),
            Nest(notations) => {
                let mut lay = T::empty();
                for notation in notations {
                    lay = self.apply(notation, in_join, in_surround, &|b| T::nest(&lay, b));
                }
                func(&lay)
            }
            Vert(notations) => {
                let mut lay = T::empty();
                for notation in notations {
                    lay = self.apply(notation, in_join, in_surround, &|b| T::vert(&lay, b));
                }
                func(&lay)
            }
            IfFlat(notation1, notation2) => self.apply(notation1, in_join, in_surround, &|b1| {
                self.apply(notation2, in_join, in_surround, &|b2| {
                    func(&T::if_flat(b1, b2))
                })
            }),
            IfEmptyText(notation1, notation2) => {
                if self.is_empty_text {
                    self.apply(notation1, in_join, in_surround, func)
                } else {
                    self.apply(notation2, in_join, in_surround, func)
                }
            }
            Repeat(box RepeatInner {
                empty,
                lone,
                join,
                surround,
            }) => match self.children.len() {
                0 => self.apply(empty, in_join, in_surround, func),
                1 => self.apply(lone, in_join, in_surround, func),
                _ => self.apply_join(&T::empty(), self.children, join, &|b| {
                    let in_surround = Some(b);
                    self.apply(surround, None, in_surround, func)
                }),
            },
            Left => func(in_join.expect("Exposed `Left`").0),
            Right => func(in_join.expect("Exposed `Right`").1),
            Surrounded => func(in_surround.expect("Exposed `Surrounded`")),
        }
    }
}
