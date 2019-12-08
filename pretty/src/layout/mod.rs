mod boundset;

// TODO rename modules to fix this for real
mod compute_bounds;
#[allow(clippy::module_inception)]
mod compute_layout;
mod layout_debug_print;
mod notation_ops;
mod staircase;

pub use boundset::BoundSet;
pub use compute_bounds::compute_bounds;
pub use compute_layout::{compute_layout, Layout, LayoutElement};
pub use notation_ops::{NotationOps, ResolvedNotation};

#[cfg(test)]
mod layout_tests {
    use typed_arena::Arena;

    use super::*;
    use crate::geometry::{Bound, Pos};
    use crate::notation::*;
    use crate::style::Style;

    #[test]
    fn test_bound_construction() {
        let sty = Style::plain();
        let actual = Bound::vert(
            Bound::literal("abc", sty, ()),
            Bound::follow(
                Bound::literal("Schrödinger", sty, ()),
                Bound::vert(
                    Bound::follow(
                        Bound::literal("I", sty, ()),
                        Bound::literal(" am indented", sty, ()),
                        (),
                    ),
                    Bound::literal("me too", sty, ()),
                    (),
                ),
                (),
            ),
            (),
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
        compute_bounds(notation, &[], is_empty_text, ())
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
    fn test_nested_list() {
        let child_notation = lit("hello");
        let child_boundset = simple_compute_bounds(&child_notation, false);
        let list_notation = example_repeat_notation();
        let inner_list_boundset: BoundSet<()> =
            compute_bounds(&list_notation, &[&child_boundset], false, ());
        let outer_list_boundset: BoundSet<()> =
            compute_bounds(&list_notation, &[&inner_list_boundset], false, ());

        assert_eq!(
            child_boundset.fit_width(80).0,
            Bound {
                width: 5,
                indent: 5,
                height: 1,
            }
        );
        assert_eq!(
            inner_list_boundset.fit_width(80).0,
            Bound {
                width: 7,
                indent: 7,
                height: 1,
            }
        );
        assert_eq!(
            outer_list_boundset.fit_width(80).0,
            Bound {
                width: 9,
                indent: 9,
                height: 1,
            }
        );
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
        let arena = Arena::new();
        let notation = lit("abc") + (lit("def") ^ lit("g"));
        let layout = compute_layout(&notation, Pos::zero(), 80, &[], false, &arena);
        assert_eq!(format!("{:?}", layout), "abcdef\n   g");
    }

    #[test]
    fn test_show_nested_list_layout() {
        let arena = Arena::new();
        let child_notation = lit("hello");
        let child_boundset = simple_compute_bounds(&child_notation, false);
        let list_notation = example_repeat_notation();
        let inner_list_boundset: BoundSet<()> =
            compute_bounds(&list_notation, &[&child_boundset], false, ());

        let outer_list_layout = compute_layout(
            &list_notation,
            Pos::zero(),
            80,
            &[&inner_list_boundset],
            false,
            &arena,
        );
        assert_eq!(format!("{:?}", outer_list_layout), "[0000000]");

        let inner_list_layout = compute_layout(
            &list_notation,
            Pos::zero(),
            80,
            &[&child_boundset],
            false,
            &arena,
        );
        assert_eq!(format!("{:?}", inner_list_layout), "[00000]");

        let child_layout = compute_layout(&child_notation, Pos::zero(), 80, &[], false, &arena);
        assert_eq!(format!("{:?}", child_layout), "hello");
    }

}
