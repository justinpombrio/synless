mod bounds;
mod boundset;
mod layout;

pub use self::bounds::{compute_bounds, compute_text_bounds, Bounds};
pub use self::layout::{compute_layout, Layout, LayoutElement};

#[cfg(test)]
mod layout_tests {
    use super::*;
    use crate::geometry::{Bound, Col, Pos};
    use crate::notation::NotationOps;
    use crate::notation::*;
    use crate::style::Style;
    use std::mem;

    use Notation::*;

    impl Notation {
        fn bounds(&mut self, child_bounds: &[Bounds], is_empty_text: bool) -> Bounds {
            *self = mem::replace(self, Empty).normalize();
            compute_bounds(self, child_bounds, is_empty_text)
        }

        fn layout(&mut self, width: Col, child_bounds: &[Bounds], is_empty_text: bool) -> Layout {
            self.bounds(child_bounds, is_empty_text);
            compute_layout(self, Pos::zero(), width, child_bounds, is_empty_text)
        }
    }

    #[test]
    fn test_bound_construction() {
        let actual = Bound::vert(
            Bound::literal("abc"),
            Bound::nest(
                Bound::literal("Schrödinger"),
                Bound::vert(
                    Bound::nest(Bound::literal("I"), Bound::literal(" am indented")),
                    Bound::literal("me too"),
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
        literal(s, Style::plain())
    }

    fn example_notation() -> Notation {
        lit("if ") + lit("true")
            ^ lit("  ") + lit("* ") + (lit("bulleted") ^ lit("list"))
            ^ lit("end")
    }

    fn example_repeat_notation() -> Notation {
        repeat(RepeatInner {
            empty: lit("[]"),
            lone: lit("[") + child(0) + lit("]"),
            surround: lit("[") + surrounded() + lit("]"),
            join: left() + lit(",") ^ right(),
        })
    }

    #[test]
    fn test_bound() {
        let actual = example_notation().bounds(&[], false).fit_width(80);
        let expected = Bound {
            width: 12,
            indent: 3,
            height: 4,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_bound_2() {
        let actual = (lit("abc") ^ lit("de")).bounds(&[], false).fit_width(80);
        let expected = Bound {
            width: 3,
            indent: 2,
            height: 2,
        };
        assert_eq!(actual, expected);
        assert_eq!(format!("{:?}", actual), "\n***\n**");
    }

    #[test]
    fn test_bound_3() {
        let actual = if_empty_text(lit("a"), lit("bc"))
            .bounds(&[], true)
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
            .bounds(&[], false)
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
        let mut notation = lit("abc") + (lit("def") ^ lit("g"));
        let layout = notation.layout(80, &[], false);
        assert_eq!(format!("{:?}", layout), "\nabcdef\n   g");
    }

    #[test]
    fn test_repeat_notation() {
        let child = (lit("abc") ^ lit("de")).bounds(&[], false);
        let zero = example_repeat_notation().layout(80, &[], false);
        let one = example_repeat_notation().layout(80, &[child.clone()], false);
        let two = example_repeat_notation().layout(80, &[child.clone(), child.clone()], false);
        let three = example_repeat_notation().layout(
            80,
            &[child.clone(), child.clone(), child.clone()],
            false,
        );
        let four = example_repeat_notation().layout(
            80,
            &[child.clone(), child.clone(), child.clone(), child.clone()],
            false,
        );
        assert_eq!(format!("{:?}", zero), "\n[]");
        assert_eq!(format!("{:?}", one), "\n[000\n 00]");
        assert_eq!(format!("{:?}", two), "\n[000\n 00,\n 111\n 11]");
        assert_eq!(
            format!("{:?}", three),
            "\n[000\n 00,\n 111\n 11,\n 222\n 22]"
        );
        assert_eq!(
            format!("{:?}", four),
            "\n[000\n 00,\n 111\n 11,\n 222\n 22,\n 333\n 33]"
        );
    }
}
