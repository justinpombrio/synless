mod boundset;
mod layout;

pub use self::layout::{
    compute_bounds, compute_layouts, text_bounds, Bounds, Lay, Layout, LayoutRegion, Layouts,
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
        assert_eq!(format!("{:?}", actual), "***\n**");
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

    #[test]
    fn test_expand_notation() {
        let r = (lit("abc") ^ lit("de")).bound(vec![], 0);
        let syn = example_repeat_notation();
        let zero = &syn.layouts(vec![], 0).fit_width(80);
        let one = &syn.layouts(vec![r.clone()], 1).fit_width(80);
        let two = &syn.layouts(vec![r.clone(), r.clone()], 2).fit_width(80);
        let three = &syn
            .layouts(vec![r.clone(), r.clone(), r.clone()], 3)
            .fit_width(80);
        let four = &syn
            .layouts(vec![r.clone(), r.clone(), r.clone(), r], 4)
            .fit_width(80);
        assert_eq!(format!("{:?}", zero), "[]");
        assert_eq!(format!("{:?}", one), "[000\n 00]");
        assert_eq!(format!("{:?}", two), "[000\n 00,\n 111\n 11]");
        assert_eq!(format!("{:?}", three), "[000\n 00,\n 111\n 11,\n 222\n 22]");
        assert_eq!(
            format!("{:?}", four),
            "[000\n 00,\n 111\n 11,\n 222\n 22,\n 333\n 33]"
        );
    }
}
