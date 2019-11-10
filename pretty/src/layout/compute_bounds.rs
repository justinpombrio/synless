use super::boundset::BoundSet;
use super::notation_ops::NotationOps;
use crate::notation::{Notation, RepeatInner};
use crate::style::Style;
use Notation::*;

/*
impl Bounds {
    /// Find the largest Bound that fits within the given width. Panics if none
    /// fits.
    pub(crate) fn fit_width(&self, width: Col) -> Bound {
        match self.bound_set.fit_width(width) {
            None => panic!("No bound fit within width {}", width),
            Some(bound) => bound,
        }
    }
}
*/

/// Compute the [`Bounds`](Bounds) of a node, given (i) the Notation with which
/// it is being displayed, (ii) the Bounds of its children, and (iii) if it is a
/// text node, whether its text is empty.
///
/// If the node is texty, then `child_bounds` should contain exactly one
/// `Bounds`, computed by [`text_bounds()`](text_bounds). If the node is not
/// texty, then `is_empty_text` will not be used (but should be false).
pub fn compute_bounds<T: NotationOps>(
    notation: &Notation,
    child_bounds: &[&BoundSet<()>],
    is_empty_text: bool,
    allocator: T::Allocator,
) -> BoundSet<T> {
    ComputeBounds::new(child_bounds, is_empty_text, allocator).compute(notation, None, None)
}

struct ComputeBounds<'a, T: NotationOps> {
    child_bounds: &'a [&'a BoundSet<()>],
    is_empty_text: bool,
    allocator: T::Allocator,
}

impl<'a, T: NotationOps> ComputeBounds<'a, T> {
    fn new(
        child_bounds: &'a [&'a BoundSet<()>],
        is_empty_text: bool,
        allocator: T::Allocator,
    ) -> ComputeBounds<'a, T> {
        ComputeBounds {
            child_bounds,
            is_empty_text,
            allocator,
        }
    }

    fn compute(
        &self,
        notation: &Notation,
        in_join: Option<(&BoundSet<T>, &BoundSet<T>)>,
        in_surround: Option<&BoundSet<T>>,
    ) -> BoundSet<T> {
        match notation {
            Empty => BoundSet::empty(self.allocator),
            Literal(string, style) => BoundSet::literal(string, *style, self.allocator),
            Text(style) => self.get_text_bounds(*style),
            Child(i) => self.get_child_bounds(*i),
            Follow(notations) => {
                let mut notations = notations.iter();
                // Assume that the Notation has been normalized, so `notations.len()` >= 2.
                let first_notation = notations.next().expect("non-normal Notation");
                let mut total_bounds = self.compute(first_notation, in_join, in_surround);
                for notation in notations {
                    let bounds = self.compute(notation, in_join, in_surround);
                    total_bounds = BoundSet::follow(total_bounds, bounds, self.allocator);
                }
                total_bounds
            }
            Vert(notations) => {
                let mut notations = notations.iter();
                // Assume that the Notation has been normalized, so `notations.len()` >= 2.
                let first_notation = notations.next().expect("non-normal Notation");
                let mut total_bounds = self.compute(first_notation, in_join, in_surround);
                for notation in notations {
                    let bounds = self.compute(notation, in_join, in_surround);
                    total_bounds = BoundSet::vert(total_bounds, bounds, self.allocator);
                }
                total_bounds
            }
            IfEmptyText(left, right) => {
                if self.is_empty_text {
                    self.compute(left, in_join, in_surround)
                } else {
                    self.compute(right, in_join, in_surround)
                }
            }
            NoWrap(notation) => {
                let mut boundset = BoundSet::new();
                for (bound, value) in self.compute(notation, in_join, in_surround) {
                    if bound.height == 1 {
                        // These bounds have already been verified to not
                        // dominate each other.
                        boundset.unchecked_insert(bound, value);
                    }
                }
                boundset
            }
            Choice(notations) => {
                let mut boundset = BoundSet::new();
                for notation in notations {
                    for (bound, value) in self.compute(notation, in_join, in_surround) {
                        boundset.insert(bound, value);
                    }
                }
                boundset
            }
            Repeat(box RepeatInner {
                empty,
                lone,
                join,
                surround,
            }) => match self.child_bounds.len() {
                0 => self.compute(empty, None, None),
                1 => self.compute(lone, None, None),
                _ => {
                    let mut total_bounds = self.get_child_bounds(0);
                    for i in 1..self.child_bounds.len() {
                        let child = self.get_child_bounds(i);
                        let in_join = (&total_bounds, &child);
                        total_bounds = self.compute(join, Some(in_join), None);
                    }
                    self.compute(surround, None, Some(&total_bounds))
                }
            },
            Left => in_join.expect("Exposed `Left`").0.to_owned(),
            Right => in_join.expect("Exposed `Right`").1.to_owned(),
            Surrounded => in_surround.expect("Exposed `Surround`").to_owned(),
        }
    }

    fn get_child_bounds(&self, i: usize) -> BoundSet<T> {
        self.child_bounds[i]
            .iter()
            .map(|(b, _)| (b, T::child(i, b, self.allocator)))
            .collect()
    }

    fn get_text_bounds(&self, style: Style) -> BoundSet<T> {
        self.child_bounds[0]
            .iter()
            .map(|(b, _)| (b, T::text(b, style, self.allocator)))
            .collect()
    }
}
