//! Syntax describes how to display a language.

mod bounds;
mod syntax;
mod layout;
mod layout_region;

pub use syntax::syntax::{Syntax, Repeat};
pub use syntax::syntax::{ empty, literal, text, repeat, flush, no_wrap,
                          child, star, concat, choice, if_empty_text };
pub use geometry::Bound;
pub use syntax::bounds::BoundSet;
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

