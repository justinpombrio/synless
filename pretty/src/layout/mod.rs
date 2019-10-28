mod boundset;

// TODO rename modules to fix this for real
mod compute_bounds;
#[allow(clippy::module_inception)]
mod compute_layout;
mod layout_debug_print;
mod notation_ops;
mod staircase;

use crate::notation::Notation;
use crate::style::Style;
use boundset::BoundSet;
use notation_ops::NotationOps;

/// Every node must keep an up-to-date `Bounds`, computed using
/// [`compute_bounds`](compute_bounds). It contains pre-computed information
/// that helps pretty-print a document.
#[derive(Debug, Clone)]
pub struct Bounds(BoundSet<()>);

/// Compute the [`Bounds`](Bounds) of a node, given (i) the Notation with which
/// it is being displayed, (ii) the Bounds of its children, and (iii) if it is a
/// text node, whether its text is empty.
///
/// If the node is texty, then `child_bounds` should contain exactly one
/// `Bounds`, computed by [`text_bounds()`](text_bounds). If the node is not
/// texty, then `is_empty_text` will not be used (but should be false).
pub fn compute_bounds(
    notation: &mut Notation,
    child_bounds: &[Bounds],
    is_empty_text: bool,
) -> Bounds {
    let child_bounds: Vec<_> = child_bounds.iter().map(|bs| &bs.0).collect();
    Bounds(compute_bounds::compute_bounds(
        notation,
        &child_bounds,
        is_empty_text,
    ))
}

/// Compute the [`Bounds`](Bounds) of a piece of text.
pub fn compute_text_bounds(text: &str) -> Bounds {
    Bounds(BoundSet::literal(text, Style::plain()))
}

/*
pub use self::layout::{
    compute_bounds, compute_layouts, text_bounds, Layout, LayoutRegion, Layouts,
};

#[cfg(test)]
mod layout_tests {
    use super::*;
    use crate::geometry::Bound;
    use crate::notation::*;
    use crate::style::Style;

    impl Notation {
        /// Compute the possible Layouts for this `Notation`, given
        /// information about its children.
        fn layouts(&self, child_bounds: Vec<Bounds>, len: usize) -> Layouts {
            let notation = self.expand(len);
            compute_layouts(&child_bounds, &notation)
        }

        /// Precompute the Bounds within which this `Notation` can be
        /// displayed, given information about its children.
        fn bound(&self, child_bounds: Vec<Bounds>, len: usize) -> Bounds {
            let notation = self.expand(len);
            compute_bounds(&child_bounds, &notation)
        }
    }

    #[test]
    fn test_bound_construction() {
        let sty = Style::plain();
        let actual = Bound::literal("abc", sty).vert(
            Bound::literal("Schrödinger", sty).horz(
                Bound::literal("I", sty)
                    .horz(Bound::literal(" am indented", sty))
                    .vert(Bound::literal("me too", sty)),
            ),
        );
        let expected = Bound {
            width: 24,
            indent: 17,
            height: 3,
        };
        assert_eq!(actual, expected);
    }

    fn lit(s: &str) -> Notation {
        literal(s, Style::plain())
    }

    fn example_notation() -> Notation {
        (lit("if ") + lit("true"))
            ^ ((lit("  ") + lit("* ") + lit("bulleted")) ^ lit("list"))
            ^ lit("end")
    }

    fn example_repeat_notation() -> Notation {
        repeat(Repeat {
            empty: lit("[]"),
            lone: lit("[") + child(0) + lit("]"),
            surround: lit("[") + child(0) + lit("]"),
            join: (child(0) + lit(",")) ^ child(1),
        })
    }

    #[test]
    fn test_bound() {
        let actual = example_notation().bound(vec![], 0).fit_width(80);
        let expected = Bound {
            width: 12,
            indent: 3,
            height: 4,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_bound_2() {
        let actual = (lit("abc") ^ lit("de")).bound(vec![], 0).fit_width(80);
        let expected = Bound {
            width: 3,
            indent: 2,
            height: 2,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_bound_3() {
        let actual = if_empty_text(lit("a"), lit("bc"))
            .bound(vec![], 0)
            .fit_width(80);
        let expected = Bound {
            width: 1,
            indent: 1,
            height: 1,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_bound_4() {
        let actual = if_empty_text(lit("a"), lit("bc"))
            .bound(vec![], 1)
            .fit_width(80);
        let expected = Bound {
            width: 2,
            indent: 2,
            height: 1,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_show_layout() {
        let syn = lit("abc") + (lit("def") ^ lit("g"));
        let lay = &syn.layouts(vec![], 0).fit_width(80);
        assert_eq!(format!("{:?}", lay), "abcdef\n   g");
    }
}
*/
