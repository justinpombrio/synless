//! Syntax describes how to display a language.

mod shapes;
mod syntax;
mod layout;

// TODO: put language tests (below) somewhere!
// TODO: clean up these tests. Should be more local.

pub use self::shapes::{Bound, Region};
pub use self::syntax::Syntax;
pub use self::syntax::*;


#[cfg(test)]
mod tests1 {
    use super::*;
    use super::layout::Layable;
    use style::Style;

    #[test]
    fn test_bound_construction() {
        let sty = Style::plain();
        let actual = Bound::literal("abc", sty).flush().concat(
            Bound::literal("Schrödinger", sty).concat(
                Bound::literal("I", sty).concat(
                    Bound::literal(" am indented", sty)).flush().concat(
                    Bound::literal("me too", sty))));
        let expected = Bound {
            width:  24,
            indent: 17,
            height: 2
        };
        assert_eq!(actual, expected);
    }

    fn lit(s: &str) -> Syntax {
        literal(s, Style::plain())
    }

    fn example_syntax() -> Syntax {
        flush(lit("if ") + lit("true"))
            + flush(lit("  ")
                + lit("* ")
                + flush(lit("bulleted"))
                + lit("list"))
            + lit("end")
    }

    #[test]
    fn test_bound() {
        let actual = example_syntax()
            .bound(0, vec!(), false).first().0;
        let expected = Bound {
            width:  12,
            indent: 3,
            height: 3
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_bound_2() {
        let actual = (flush(lit("abc")) + lit("de"))
            .bound(0, vec!(), false).first().0;
        let expected = Bound {
            width:  3,
            indent: 2,
            height: 1
        };
        assert_eq!(actual, expected);
        assert_eq!(format!("{:?}", actual), "***\n**");
    }

    #[test]
    fn test_bound_3() {
        let actual = if_empty_text(lit("a"), lit("bc"))
            .bound(0, vec!(), true).first().0;
        let expected = Bound {
            width: 1,
            indent: 1,
            height: 0
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_bound_4() {
        let actual = if_empty_text(lit("a"), lit("bc"))
            .bound(0, vec!(), false).first().0;
        let expected = Bound {
            width: 2,
            indent: 2,
            height: 0
        };
        assert_eq!(actual, expected);
    }
}


#[cfg(test)]
mod tests2 {
    use style::Style;
    use super::*;

    fn lit(s: &str) -> Syntax {
        literal(s, Style::plain())
    }

    fn example_repeat_syntax() -> Syntax {
        child(0) +
            repeat(Repeat{
                empty:  lit("[]"),
                lone:   lit("[") + star() + lit("]"),
                first:  lit("[") + flush(star() + lit(",")),
                middle: flush(star() + lit(",")),
                last:   star() + lit("]")
            })
    }

    #[test]
    fn test_show_layout() {
        let syn = lit("abc") + (flush(lit("def")) + lit("g"));
        let lay = &syn.lay_out(0, vec!(), false).fit_width(80).1;
        assert_eq!(format!("{:?}", lay), "abcdef\n   g");
    }

    #[test]
    fn test_expand_syntax() {
        let r = (flush(lit("abc")) + lit("de")).bound(0, vec!(), false);
        let syn = example_repeat_syntax();
        let zero = &syn
            .lay_out(1, vec!(&r), false)
            .fit_width(80).1;
        let one = &syn
            .lay_out(1, vec!(&r, &r), false)
            .fit_width(80).1;
        let two = &syn
            .lay_out(1, vec!(&r, &r, &r), false)
            .fit_width(80).1;
        let three = &syn
            .lay_out(1, vec!(&r, &r, &r, &r), false)
            .fit_width(80).1;
        let four = &syn
            .lay_out(1, vec!(&r, &r, &r, &r, &r), false)
            .fit_width(80).1;
        assert_eq!(format!("{:?}", zero), "000\n00[]");
        assert_eq!(format!("{:?}", one), "000\n00[111\n   11]");
        assert_eq!(format!("{:?}", two),
                   "000\n00[111\n   11,\n   222\n   22]");
        assert_eq!(format!("{:?}", three),
                   "000\n00[111\n   11,\n   222\n   22,\n   333\n   33]");
        assert_eq!(format!("{:?}", four),
                   "000\n00[111\n   11,\n   222\n   22,\n   333\n   33,\n   444\n   44]");
    }
}




/*
#[cfg(test)]
mod tests {
    use language::Language;
    use language::make_example_tree;

    #[test]
    fn test_lay_out() {
        let lang = Language::example_language();
        let doc = make_example_tree(&lang, false);

        assert_eq!(doc.as_ref().write(80),
                   "func foo(abc, def) { 'abcdef' + 'abcdef' }");
        assert_eq!(doc.as_ref().write(42),
                   "func foo(abc, def) { 'abcdef' + 'abcdef' }");
        assert_eq!(doc.as_ref().write(41),
                   "func foo(abc, def) { 'abcdef'
                     + 'abcdef' }");
        assert_eq!(doc.as_ref().write(33),
                   "func foo(abc, def) { 'abcdef'
                     + 'abcdef' }");
        assert_eq!(doc.as_ref().write(32),
                   "func foo(abc, def) {
  'abcdef' + 'abcdef'
}");
        assert_eq!(doc.as_ref().write(21),
                   "func foo(abc, def) {
  'abcdef' + 'abcdef'
}");
        assert_eq!(doc.as_ref().write(20),
                   "func foo(abc, def) {
  'abcdef'
  + 'abcdef'
}");
        assert_eq!(doc.as_ref().write(19),
                   "func foo(abc,
         def) {
  'abcdef'
  + 'abcdef'
}");
        assert_eq!(doc.as_ref().write(15),
                   "func foo(abc,
         def) {
  'abcdef'
  + 'abcdef'
}");
        assert_eq!(doc.as_ref().write(14),
                   "func foo(
  abc, def)
{
  'abcdef'
  + 'abcdef'
}");
        assert_eq!(doc.as_ref().write(12),
                   "func foo(
  abc, def)
{
  'abcdef'
  + 'abcdef'
}");
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use style::Style;
    use language::Language;
    use language::make_example_tree;

    #[test]
    fn test_regionize() {
        let small_region = Region{
            pos: Pos{ row: 2, col: 3 },
            bound: Bound{ width: 5, height: 2, indent: 2 }
        };
        let layout = Literal("ok".to_string(), Style::plain());
        assert_eq!(layout.regionize(small_region), LayoutRegion{
            layout: Layout::Literal("ok".to_string(), Style::plain()),
            region: Region{
                pos: Pos{ row: 2, col: 3 },
                bound: Bound{ width: 2, height: 0, indent: 2 }
            }
        });
        
        let layout = Text(Style::plain(),
                          Bound{ width: 2, height: 1, indent: 3 });
        assert_eq!(layout.regionize(small_region), LayoutRegion{
            layout: Layout::Text(Style::plain()),
            region: Region{
                pos: Pos{ row: 2, col: 3 },
                bound: Bound{ width: 2, height: 1, indent: 3 }
            }
        });

        let layout = Flush(Box::new(Text(
            Style::plain(),
            Bound{ width: 2, height: 1, indent: 3 })));
        assert_eq!(layout.regionize(small_region), LayoutRegion{
            layout: Layout::Flush(Box::new(LayoutRegion{
                layout: Layout::Text(Style::plain()),
                region: Region{
                    pos: Pos{ row: 2, col: 3 },
                    bound: Bound{ width: 2, height: 1, indent: 3 }
                }
            })),
            region: Region{
                pos: Pos{ row: 2, col: 3 },
                bound: Bound{ width: 2, height: 2, indent: 0 }
            }
        });

        let left_layout = Child(0, Bound{ width: 3, height: 1, indent: 1 });
        let right_layout = Literal("ok".to_string(), Style::plain());
        let layout = Concat(Box::new(left_layout), Box::new(right_layout));
        let left_expected = LayoutRegion{
            layout: Layout::Child(0),
            region: Region{
                pos: Pos{ row: 2, col: 3 },
                bound: Bound{ width: 3, height: 1, indent: 1 }
            }
        }; // 2:3-3:4;3
        let right_expected = LayoutRegion{
            layout: Layout::Literal("ok".to_string(), Style::plain()),
            region: Region{
                pos: Pos{ row: 3, col: 4 },
                bound: Bound{ width: 2, height: 0, indent: 2 }
            }
        }; // 3:4-3:6;2
        let expected = LayoutRegion{
            layout: Layout::Concat(Box::new(left_expected), Box::new(right_expected)),
            region: Region{
                pos: Pos{ row: 2, col: 3 },
                bound: Bound{ width: 5, height: 1, indent: 3 }
            }
        }; // 2:3-3:6;5
        assert_eq!(layout.regionize(small_region), expected);
        
        let lang = Language::example_language();
        let doc = make_example_tree(&lang, false);
        let region = Region{
            pos: Pos{ row: 2, col: 3 },
            bound: Bound::infinite_scroll(20)
        };
        let lay = doc.as_ref().lay_out(region);
        // layout:
"func foo(abc, def) {
  'abcdef'
  + 'abcdef'
}";

        fn left_of_concat(lay: LayoutRegion) -> LayoutRegion {
            match lay.layout {
                Layout::Concat(box lay, _) => lay,
                _ => panic!("Expected concat")
            }
        }
        fn right_of_concat(lay: LayoutRegion) -> LayoutRegion {
            match lay.layout {
                Layout::Concat(_, box lay) => lay,
                _ => panic!("Expected concat")
            }
        }

        println!("{:?}", lay);
        assert_eq!(&format!("{}", lay.region),
                   "2:3-5:4;20");
        let lay = right_of_concat(left_of_concat(lay));
        assert_eq!(&format!("{}", lay.region),
                   "3:3-5:3;20");
    }
}


*/
