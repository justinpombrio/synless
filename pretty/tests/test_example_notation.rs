#![feature(slice_patterns)]

mod common;

use common::make_example_doc;

// TODO: test horz concat

#[test]
fn test_lay_out() {
    let doc = make_example_doc();
    assert_eq!(doc.write(80), "func foo(abc, def) { 'abcdef' + 'abcdef' }");
    assert_eq!(doc.write(42), "func foo(abc, def) { 'abcdef' + 'abcdef' }");
    assert_eq!(
        doc.write(41),
        "func foo(abc, def) { 'abcdef'
                     + 'abcdef' }"
    );
    assert_eq!(
        doc.write(33),
        "func foo(abc, def) { 'abcdef'
                     + 'abcdef' }"
    );
    assert_eq!(
        doc.write(32),
        "func foo(abc, def) {
  'abcdef' + 'abcdef'
}"
    );
    assert_eq!(
        doc.write(21),
        "func foo(abc, def) {
  'abcdef' + 'abcdef'
}"
    );
    assert_eq!(
        doc.write(20),
        "func foo(abc, def) {
  'abcdef'
  + 'abcdef'
}"
    );
    assert_eq!(
        doc.write(19),
        "func foo(abc,
         def) {
  'abcdef'
  + 'abcdef'
}"
    );
    assert_eq!(
        doc.write(15),
        "func foo(abc,
         def) {
  'abcdef'
  + 'abcdef'
}"
    );
    assert_eq!(
        doc.write(14),
        "func foo(
  abc, def)
{
  'abcdef'
  + 'abcdef'
}"
    );
    assert_eq!(
        doc.write(12),
        "func foo(
  abc, def)
{
  'abcdef'
  + 'abcdef'
}"
    );
}
