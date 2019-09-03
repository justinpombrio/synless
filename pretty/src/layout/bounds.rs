use crate::geometry::{Col, MAX_WIDTH};
use crate::layout::boundset::BoundSet;
use crate::notation::Notation;
use crate::notation_ops::{apply_notation, NotationOps};

/// Every node must keep an up-to-date `Bounds`, computed using
/// [`compute_bounds`](compute_bounds). It contains pre-computed information to
/// speed up pretty-printing.
#[derive(Debug, Clone)]
pub struct Bounds {
    pub(super) flat_width: Option<Col>,
    pub(super) bound_set: BoundSet,
}

/// Compute the [`Bounds`](Bounds) of a node, given (i) the Bounds of its
/// children, (ii) if it is a text node, whether its text is empty, and (iii)
/// the Notation with which it is being displayed.
///
/// If the node is texty, then `child_bounds` should contain exactly one
/// `Bounds`, computed by [`text_bounds()`](text_bounds). If the node is not
/// texty, then `is_empty_text` will not be used and can have any value.
pub fn compute_bounds(child_bounds: &[Bounds], is_empty_text: bool, notation: &Notation) -> Bounds {
    apply_notation(child_bounds, is_empty_text, notation)
}

/// Compute the [`Bounds`](Bounds) of a piece of text.
pub fn text_bounds(text: &str) -> Bounds {
    Bounds::literal(text)
}

// TODO:
// 1. Remember flat_width
// 2. Do not clone

impl NotationOps for Option<Col> {
    fn empty() -> Option<Col> {
        Some(0)
    }

    fn literal(s: &str) -> Option<Col> {
        Some(s.chars().count() as Col)
    }

    fn nest(fw1: &Option<Col>, fw2: &Option<Col>) -> Option<Col> {
        let fw = (*fw1)? + (*fw2)?;
        if fw <= MAX_WIDTH {
            Some(fw)
        } else {
            None
        }
    }

    fn vert(_fw1: &Option<Col>, _fw2: &Option<Col>) -> Option<Col> {
        None
    }

    fn if_flat(fw1: &Option<Col>, fw2: &Option<Col>) -> Option<Col> {
        fw1.or(*fw2)
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

    fn nest(b1: &Bounds, b2: &Bounds) -> Bounds {
        Bounds {
            flat_width: <Option<Col>>::nest(&b1.flat_width, &b2.flat_width),
            bound_set: BoundSet::nest(&b1.bound_set, &b2.bound_set),
        }
    }

    fn vert(b1: &Bounds, b2: &Bounds) -> Bounds {
        Bounds {
            flat_width: <Option<Col>>::vert(&b1.flat_width, &b2.flat_width),
            bound_set: BoundSet::vert(&b1.bound_set, &b2.bound_set),
        }
    }

    fn if_flat(b1: &Bounds, b2: &Bounds) -> Bounds {
        Bounds {
            flat_width: <Option<Col>>::if_flat(&b1.flat_width, &b2.flat_width),
            bound_set: BoundSet::if_flat(&b1.bound_set, &b2.bound_set),
        }
    }
}
