use crate::geometry::{Bound, Col};
use crate::layout::boundset::BoundSet;
use crate::notation::Notation::*;
use crate::notation::{Notation, RepeatInner};
use utility::error;

pub struct Bounds {
    flat_width: Option<usize>,
    bound_set: BoundSet,
}

struct ComputeBounds<'a> {
    child_bounds: &'a [Bounds],
}

impl<'a> ComputeBounds<'a> {
    fn new(child_bounds: &[Bounds]) -> ComputeBounds {
        ComputeBounds { child_bounds }
    }

    fn compute_flat_width(
        &self,
        notation: &Notation,
        in_join: Option<(usize, usize)>,
        in_surround: Option<usize>,
    ) -> Option<usize> {
        match notation {
            Empty => Some(0),
            Literal(s, _) => Some(s.chars().count()),
            Text(_) => self.child_bounds[0].flat_width,
            Child(i) => self.child_bounds[*i].flat_width,
            Nest(notations) => {
                let mut flat_width = 0;
                for notation in notations {
                    match self.compute_flat_width(notation, in_join, in_surround) {
                        None => return None,
                        Some(w) => flat_width += w,
                    }
                }
                Some(flat_width)
            }
            // assume Notation is normalized
            Vert(notations) => None,
            IfFlat(n1, n2) => self
                .compute_flat_width(n1, in_join, in_surround)
                .or_else(|| self.compute_flat_width(n2, in_join, in_surround)),
            IfEmptyText(n1, n2) => {
                if self.child_bounds[0].flat_width == Some(0) {
                    self.compute_flat_width(n1, in_join, in_surround)
                } else {
                    self.compute_flat_width(n2, in_join, in_surround)
                }
            }
            Repeat(box RepeatInner {
                empty,
                lone,
                join,
                surround,
            }) => match self.child_bounds.len() {
                0 => self.compute_flat_width(empty, in_join, in_surround),
                1 => self.compute_flat_width(lone, in_join, in_surround),
                _ => {
                    let fw = self.child_bounds[0].flat_width?;
                    let mut flat_width = fw;
                    for fw in self.child_bounds.iter().skip(1).map(|b| b.flat_width) {
                        let in_join = Some((flat_width, fw?));
                        flat_width = self.compute_flat_width(join, in_join, None)?;
                    }
                    self.compute_flat_width(surround, None, Some(flat_width))
                }
            },
            Left => Some(in_join.expect("ComputeBounds: unexpected Left").0),
            Right => Some(in_join.expect("ComputeBounds: unexpected Right").1),
            Surrounded => Some(in_surround.expect("ComputeBounds: unexpected Surrounded")),
        }
    }

    fn compute_bound_set(&self, notation: &Notation) -> BoundSet {
        unimplemented!()
    }

    fn compute_bounds(&self, notation: &Notation) -> Bounds {
        Bounds {
            flat_width: self.compute_flat_width(notation, None, None),
            bound_set: self.compute_bound_set(notation),
        }
    }
}

// If the node is texty, `child_bounds` should be a singleton vec of the text bounds.
pub fn compute_bounds(child_bounds: &[Bounds], notation: &Notation) -> Bounds {
    ComputeBounds::new(child_bounds).compute_bounds(notation)
}
