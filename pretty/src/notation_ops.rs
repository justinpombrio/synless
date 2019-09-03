use crate::notation::{Notation, RepeatInner};
use crate::style::Style;
use Notation::*;

pub trait NotationOps: Clone {
    fn empty() -> Self;
    fn literal(string: &str, style: Style) -> Self;
    fn text(child: &Self, style: Style) -> Self;
    fn child(children: &[Self], i: usize) -> Self;
    // TODO: add `nests()` and `verts()` with default implementations.
    fn nest(left: Self, right: Self) -> Self;
    fn vert(left: Self, right: Self) -> Self;
    fn if_flat(left: Self, right: Self) -> Self;
}

pub fn apply_notation<T: NotationOps>(
    children: &[T],
    is_empty_text: bool,
    notation: &Notation,
) -> T {
    ApplyNotation::new(children, is_empty_text).apply(&notation, None, None)
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

    fn apply(&self, notation: &Notation, in_join: Option<(&T, &T)>, in_surround: Option<&T>) -> T {
        match notation {
            Empty => T::empty(),
            Literal(string, style) => T::literal(string, *style),
            Text(style) => T::text(&self.children[0], *style),
            Child(i) => T::child(self.children, *i),
            Nest(notations) => {
                let mut lay = T::empty();
                for notation in notations {
                    lay = T::nest(lay, self.apply(notation, in_join, in_surround));
                }
                lay
            }
            Vert(notations) => {
                let mut lay = T::empty();
                for notation in notations {
                    lay = T::vert(lay, self.apply(notation, in_join, in_surround));
                }
                lay
            }
            IfFlat(notation1, notation2) => T::if_flat(
                self.apply(notation1, in_join, in_surround),
                self.apply(notation2, in_join, in_surround),
            ),
            IfEmptyText(notation1, notation2) => {
                if self.is_empty_text {
                    self.apply(notation1, in_join, in_surround)
                } else {
                    self.apply(notation2, in_join, in_surround)
                }
            }
            Repeat(box RepeatInner {
                empty,
                lone,
                join,
                surround,
            }) => match self.children.len() {
                0 => self.apply(empty, in_join, in_surround),
                1 => self.apply(lone, in_join, in_surround),
                n => {
                    let mut lay = T::empty();
                    for i in (0..(n - 1)).rev() {
                        let in_join = Some((&lay, &self.children[i]));
                        lay = self.apply(join, in_join, None);
                    }
                    let in_surround = Some(&lay);
                    self.apply(surround, None, in_surround)
                }
            },
            Left => in_join.expect("Exposed `Left`").0.clone(),
            Right => in_join.expect("Exposed `Right`").1.clone(),
            Surrounded => in_surround.expect("Exposed `Surrounded`").clone(),
        }
    }
}
