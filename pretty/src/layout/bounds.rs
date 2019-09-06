use crate::geometry::{Bound, Col, MAX_WIDTH};
use crate::layout::boundset::BoundSet;
use crate::notation::{Notation, NotationOps, RepeatInner};
use Notation::*;

/// Every node must keep an up-to-date `Bounds`, computed using
/// [`compute_bounds`](compute_bounds). It contains pre-computed information
/// that helps pretty-print a document.
#[derive(Debug, Clone)]
pub struct Bounds {
    flat_width: Option<Col>,
    bound_set: BoundSet,
}

impl Bounds {
    /// Find the largest Bound that fits within the given width. Panics if none
    /// fits.
    pub fn fit_width(&self, width: Col) -> Bound {
        match self.bound_set.fit_width(width) {
            None => panic!("No bound fit within width {}", width),
            Some(bound) => bound,
        }
    }
}

/// Compute the [`Bounds`](Bounds) of a node, given (i) the Bounds of its
/// children, (ii) if it is a text node, whether its text is empty, and (iii)
/// the Notation with which it is being displayed.
///
/// If the node is texty, then `child_bounds` should contain exactly one
/// `Bounds`, computed by [`text_bounds()`](text_bounds). If the node is not
/// texty, then `is_empty_text` will not be used and can have any value.
pub fn compute_bounds(
    notation: &mut Notation,
    child_bounds: &[Bounds],
    is_empty_text: bool,
) -> Bounds {
    ComputeBounds::new(child_bounds, is_empty_text).compute(notation, None, None)
}

/// Compute the [`Bounds`](Bounds) of a piece of text.
pub fn compute_text_bounds(text: &str) -> Bounds {
    Bounds::literal(text)
}

impl NotationOps for Option<Col> {
    fn empty() -> Option<Col> {
        Some(0)
    }

    fn literal(s: &str) -> Option<Col> {
        Some(s.chars().count() as Col)
    }

    fn nest(fw1: Option<Col>, fw2: Option<Col>) -> Option<Col> {
        let fw = fw1? + fw2?;
        if fw <= MAX_WIDTH {
            Some(fw)
        } else {
            None
        }
    }

    fn vert(_fw1: Option<Col>, _fw2: Option<Col>) -> Option<Col> {
        None
    }

    fn if_flat(fw1: Option<Col>, fw2: Option<Col>) -> Option<Col> {
        fw1.or(fw2)
    }
}

impl NotationOps for Bounds {
    fn empty() -> Bounds {
        Bounds {
            flat_width: <Option<Col>>::empty(),
            bound_set: BoundSet::empty(),
        }
    }

    fn literal(s: &str) -> Bounds {
        Bounds {
            flat_width: <Option<Col>>::literal(s),
            bound_set: BoundSet::literal(s),
        }
    }

    fn nest(b1: Bounds, b2: Bounds) -> Bounds {
        Bounds {
            flat_width: <Option<Col>>::nest(b1.flat_width, b2.flat_width),
            bound_set: BoundSet::nest(b1.bound_set, b2.bound_set),
        }
    }

    fn vert(b1: Bounds, b2: Bounds) -> Bounds {
        Bounds {
            flat_width: <Option<Col>>::vert(b1.flat_width, b2.flat_width),
            bound_set: BoundSet::vert(b1.bound_set, b2.bound_set),
        }
    }

    fn if_flat(b1: Bounds, b2: Bounds) -> Bounds {
        Bounds {
            flat_width: <Option<Col>>::if_flat(b1.flat_width, b2.flat_width),
            bound_set: BoundSet::if_flat(b1.bound_set, b2.bound_set),
        }
    }
}

struct ComputeBounds<'a> {
    child_bounds: &'a [Bounds],
    is_empty_text: bool,
}

impl ComputeBounds<'_> {
    fn new(child_bounds: &[Bounds], is_empty_text: bool) -> ComputeBounds {
        ComputeBounds {
            child_bounds: child_bounds,
            is_empty_text: is_empty_text,
        }
    }

    fn compute(
        &self,
        notation: &mut Notation,
        in_join: Option<(&Bounds, &Bounds)>,
        in_surround: Option<&Bounds>,
    ) -> Bounds {
        match notation {
            Empty => Bounds::empty(),
            Literal(string, _) => Bounds::literal(string),
            Text(_) => self.child_bounds[0].clone(),
            Child(i) => self.child_bounds[*i].clone(),
            Nest(notations) => {
                let mut notations = notations.iter_mut();
                // Assume that the Notation has been normalized, so `notations.len()` >= 2.
                let first_notation = notations.next().expect("non-normal Notation");
                let mut total_bounds = self.compute(first_notation, in_join, in_surround);
                for notation in notations {
                    let bounds = self.compute(notation, in_join, in_surround);
                    total_bounds = Bounds::nest(total_bounds, bounds);
                }
                total_bounds
            }
            Vert(notations) => {
                let mut notations = notations.iter_mut();
                // Assume that the Notation has been normalized, so `notations.len()` >= 2.
                let first_notation = notations.next().expect("non-normal Notation");
                let mut total_bounds = self.compute(first_notation, in_join, in_surround);
                for notation in notations {
                    let bounds = self.compute(notation, in_join, in_surround);
                    total_bounds = Bounds::vert(total_bounds, bounds);
                }
                total_bounds
            }
            IfFlat(left, right, flat_width) => {
                let left_bounds = self.compute(left, in_join, in_surround);
                let right_bounds = self.compute(right, in_join, in_surround);
                let bounds = Bounds::if_flat(left_bounds, right_bounds);
                *flat_width = bounds.flat_width;
                bounds
            }
            IfEmptyText(left, right) => {
                if self.is_empty_text {
                    self.compute(left, in_join, in_surround)
                } else {
                    self.compute(right, in_join, in_surround)
                }
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
                    let mut total_bounds = Bounds::empty();
                    for child in self.child_bounds {
                        let in_join = (&total_bounds, child);
                        total_bounds = self.compute(join, Some(in_join), None);
                    }
                    self.compute(surround, None, Some(&total_bounds))
                }
            },
            Left => in_join.expect("Exposed `Left`").0.clone(),
            Right => in_join.expect("Exposed `Right`").1.clone(),
            Surrounded => in_surround.expect("Exposed `Surround`").clone(),
        }
    }
}
