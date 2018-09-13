//! Syntax describes how to display a language.

mod shapes;
mod syntax;
mod find_bounds;
//mod layout;

pub use self::shapes::{Bound, BoundSet, Region};
pub use self::syntax::Syntax;







// TODO: put tests somewhere!


/*
#[cfg(test)]
mod tests {
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
        let lay = &syn.lay_out(0, &vec!(), false).fit_width(80);
        assert_eq!(format!("{:?}", lay), "abcdef\n   g");
    }

    #[test]
    fn test_expand_syntax() {
        let r = (flush(lit("abc")) + lit("de")).bound(0, vec!(), false);
        let syn = example_repeat_syntax();
        let zero = &syn
            .lay_out(1, &vec!(r.clone()), false)
            .fit_width(80);
        let one = &syn
            .lay_out(1, &vec!(r.clone(), r.clone()), false)
            .fit_width(80);
        let two = &syn
            .lay_out(1, &vec!(r.clone(), r.clone(), r.clone()), false)
            .fit_width(80);
        let three = &syn
            .lay_out(1, &vec!(r.clone(), r.clone(), r.clone(), r.clone()), false)
            .fit_width(80);
        let four = &syn
            .lay_out(1, &vec!(r.clone(), r.clone(), r.clone(), r.clone(), r.clone()), false)
            .fit_width(80);
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
*/


/*
mod bounds;
mod syntax;
mod layout;
mod layout_region;

pub use bound::Bound;
pub use syntax::syntax::{Syntax, Repeat};
pub use syntax::syntax::{ empty, literal, text, repeat, flush, no_wrap,
                          child, star, concat, choice, if_empty_text };
pub use geometry::{Bound, BoundSet};
pub use syntax::layout_region::{Layout, LayoutRegion};

// Notation-related words:
// script, phrase, syntax, representation, format, diction


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

*/
