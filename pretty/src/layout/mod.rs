mod boundset;

// TODO rename modules to fix this for real
mod compute_bounds;
#[allow(clippy::module_inception)]
mod compute_layout;
mod layout_debug_print;
mod notation_ops;
mod staircase;

use crate::geometry::{Col, Pos};
use crate::notation::Notation;
use crate::style::Style;
pub use boundset::BoundSet;
use notation_ops::NotationOps;

#[cfg(test)]
use crate::geometry::Bound;

pub use compute_bounds::compute_bounds;
pub use compute_layout::{compute_layout, Layout, LayoutElement};

#[cfg(test)]
mod layout_tests {
    use super::*;
    use crate::geometry::Bound;
    use crate::notation::*;
    use crate::style::Style;

    #[test]
    fn test_bound_construction() {
        let sty = Style::plain();
        let actual = Bound::vert(
            Bound::literal("abc", sty),
            Bound::follow(
                Bound::literal("Schrödinger", sty),
                Bound::vert(
                    Bound::follow(
                        Bound::literal("I", sty),
                        Bound::literal(" am indented", sty),
                    ),
                    Bound::literal("me too", sty),
                ),
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
        Notation::Literal(s.to_string(), Style::plain())
    }

    fn simple_compute_bounds(notation: &Notation, is_empty_text: bool) -> BoundSet<()> {
        compute_bounds(notation, &[], is_empty_text)
    }

    fn example_notation() -> Notation {
        (lit("if ") + lit("true"))
            ^ ((lit("  ") + lit("* ") + lit("bulleted")) ^ lit("list"))
            ^ lit("end")
    }

    fn example_repeat_notation() -> Notation {
        Notation::Repeat(Box::new(RepeatInner {
            empty: lit("[]"),
            lone: lit("[") + Notation::Child(0) + lit("]"),
            surround: lit("[") + Notation::Surrounded + lit("]"),
            join: (Notation::Left + lit(",")) ^ Notation::Right,
        }))
    }

    #[test]
    fn test_literal() {
        let notation = lit("hello");
        let actual = simple_compute_bounds(&notation, false).fit_width(80).0;
        let expected = Bound {
            width: 5,
            indent: 5,
            height: 1,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_vert() {
        let notation = lit("hello") ^ lit("world!");
        let actual = simple_compute_bounds(&notation, false).fit_width(80).0;
        let expected = Bound {
            width: 6,
            indent: 6,
            height: 2,
        };
        assert_eq!(actual, expected);
        let notation = lit("hello, dear") ^ lit("world");
        let actual = simple_compute_bounds(&notation, false).fit_width(80).0;
        let expected = Bound {
            width: 11,
            indent: 5,
            height: 2,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_follow() {
        let notation = lit("hello, ") + lit("world");
        let actual = simple_compute_bounds(&notation, false).fit_width(80).0;
        let expected = Bound {
            width: 12,
            indent: 12,
            height: 1,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_example() {
        let actual = simple_compute_bounds(&example_notation(), false)
            .fit_width(80)
            .0;
        let expected = Bound {
            width: 12,
            indent: 3,
            height: 4,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_is_empty_text() {
        let notation = Notation::IfEmptyText(Box::new(lit("a")), Box::new(lit("bc")));
        let actual = simple_compute_bounds(&notation, false).fit_width(80).0;
        let expected = Bound {
            width: 2,
            indent: 2,
            height: 1,
        };
        assert_eq!(actual, expected);

        let actual = simple_compute_bounds(&notation, true).fit_width(80).0;
        let expected = Bound {
            width: 1,
            indent: 1,
            height: 1,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_show_layout() {
        let notation = lit("abc") + (lit("def") ^ lit("g"));
        let layout = compute_layout(&notation, Pos::zero(), 80, &[], false);
        assert_eq!(format!("{:?}", layout), "abcdef\n   g");
    }
}
