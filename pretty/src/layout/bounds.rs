use crate::geometry::{Bound, Col, MAX_WIDTH};
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

impl Lay for Option<usize> {
    fn empty() -> Option<usize> {
        Some(0)
    }

    fn literal(s: &str) -> Option<usize> {
        Some(s.chars().count())
    }

    fn nest(self, other: Option<usize>) -> Option<usize> {
        let fw1 = self?;
        let fw2 = other?;
        let fw = fw1 + fw2;
        if fw <= MAX_WIDTH {
            Some(fw)
        } else {
            None
        }
    }

    fn vert(self, other: Option<usize>) -> Option<usize> {
        None
    }

    fn if_flat(self, other: Option<usize>) -> Option<usize> {
        self.or(other)
    }
}

impl Lay for BoundSet {
    fn empty() -> BoundSet {
        BoundSet::singleton(Bound::empty())
    }

    fn literal(s: &str) -> BoundSet {
        BoundSet::singleton(Bound::literal(s))
    }

    fn nest(self, other: BoundSet) -> BoundSet {
        let mut set = BoundSet::new();
        for width in 0..MAX_WIDTH {
            let bound_1 = self[width];
            let remaining_width = width - bound_1.index;
            let bound_2 = other[remaining_width];
            set.insert(bound_1.nest(bound_2));
        }
        set
    }

    fn vert(self, other: BoundSet) -> BoundSet {
        let mut set = BoundSet::new();
        for width in 0..MAX_WIDTH {
            let bound_1 = self[width];
            let bound_2 = other[width];
            set.insert(bound_1.vert(bound_2));
        }
        set
    }

    fn if_flat(self, other: BoundSet) -> BoundSet {
        let mut set = BoundSet::new();
        for width in 0..MAX_WIDTH {
            let bound_1 = self[width];
            if bound_1.height > 1 {
                set.insert(bound_1);
            } else {
                set.insert(other[width]);
            }
        }
        set
    }
}

impl Lay for Bounds {
    fn empty() -> Bounds {
        Bounds {
            flat_width: Option<usize>::empty(),
            bound_set: BoundSet::empty(),
        }
    }

    fn literal(s: &str) -> Bounds {
        Bounds {
            flat_width: Option<usize>::literal(s),
            bound_set: BoundSet::literal(s),
        }
    }

    fn nest(self, other: Bounds) -> Bounds {
        Bounds {
            flat_width: self.flat_width.nest(other.flat_width),
            bound_set: self.bound_set.nest(other.bound_set),
        }
    }

    fn vert(&self, other: Bounds) -> Bounds {
        Bounds {
            flat_width: self.flat_width.vert(other.flat_width),
            bound_set: self.bound_set.vert(other.bound_set),
        }
    }

    fn if_flat(&self, other: Bounds) -> Bounds {
        Bounds {
            flat_width: self.flat_width.if_flat(other.flat_width),
            bound_set: self.bound_set.if_flat(other.bound_set),
        }
    }
}

impl<'a> ComputeBounds<'a> {
    fn new(child_bounds: &[Bounds]) -> ComputeBounds {
        ComputeBounds { child_bounds }
    }

    fn compute(
        &self,
        notation: &Notation,
        in_join: Option<(Bounds, Bounds)>,
        in_surround: Option<Bounds>,
    ) -> Option<Bounds> {
        match notation {
            Empty => Bounds {
                flat_width: Some(0),
                bound_set: BoundSet::singleton(Bound::empty()),
            },
            Literal(s, _) => Bounds {
                flat_width: Some(s.chars().count()),
                bound_set: BoundSet::singleton(Bound::literal(s)),
            },
            Text(_) => self.child_bounds[0],
            Child(i) => self.child_bounds[*i],
            Nest(notations) => {
                let mut accum_bounds = Bounds {
                    flat_width: Some(0),
                    bound_set: BoundSet::singleton(Bound::empty()),
                };
                for notation in notations {
                    let bounds = self.compute(notation, in_join, in_surround);
                    match bounds.flat_width {
                        None => total_bounds.flat_width = None,
                        Some(w) => total_bounds.flat_width.value += w,
                    }
                    for width in 0..MAX_WIDTH {
                        let outer_bound = accum_bounds[width];
                        let remaining_width = width - outer_bound.indent;
                        let inner_bound = bounds[remaining_width];
                    }
                }
            }
        }
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
                    let mut flat_width = self.child_bounds[0].flat_width?;
                    for fw in self.child_bounds.iter().skip(1).map(|b| b.flat_width) {
                        let in_join = Some((flat_width, fw?));
                        flat_width = self.compute_flat_width(join, in_join, None)?;
                    }
                    let in_surround = Some(flat_width);
                    self.compute_flat_width(surround, None, in_surround)
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
