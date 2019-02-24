#![feature(slice_patterns)]

mod common;

use common::{assert_strings_eq, make_example_doc, make_json_doc};

// TODO: test horz concat

#[test]
fn test_lay_out_json_80() {
    let doc = make_json_doc();

    assert_strings_eq(
        &doc.write(80),
        r#"{
  "firstName": "John",
  "lastName": "Smith",
  "isAlive": true,
  "age": 27,
  "address":
    {
      "streetAddress": "21 2nd Street",
      "city": "New York",
      "state": "NY",
      "postalCode": "10021-3100"
    },
  "phoneNumbers":
    [{
       "type": "home",
       "number": "212 555-1234"
     },
     "surprise string!",
     {
       "type": "office",
       "number": "646 555-4567"
     }],
  "children": [],
  "spouse": null,
  "favoriteThings":
    ["raindrops on roses", "whiskers on kittens", {"color": "red"},
     {"food": "pizza"}, "brown paper packages", [{}]],
  "lists": ["first", ["second", ["third", ["fourth", "fifth is longer"]]]]
}"#,
    )
}

#[test]
fn test_lay_out_json_30() {
    let doc = make_json_doc();

    assert_strings_eq(
        &doc.write(30),
        r#"{
  "firstName": "John",
  "lastName": "Smith",
  "isAlive": true,
  "age": 27,
  "address":
    {
      "streetAddress":
        "21 2nd Street",
      "city": "New York",
      "state": "NY",
      "postalCode":
        "10021-3100"
    },
  "phoneNumbers":
    [{
       "type": "home",
       "number":
         "212 555-1234"
     },
     "surprise string!",
     {
       "type": "office",
       "number":
         "646 555-4567"
     }],
  "children": [],
  "spouse": null,
  "favoriteThings":
    ["raindrops on roses",
     "whiskers on kittens",
     {"color": "red"},
     {"food": "pizza"},
     "brown paper packages",
     [{}]],
  "lists":
    ["first",
     ["second",
      ["third",
       ["fourth",
        "fifth is longer"]]]]
}"#,
    );
}

#[test]
#[should_panic]
fn test_lay_out_json_28() {
    // The doc won't fit in 28 characters
    let doc = make_json_doc();
    doc.write(28);
}

#[test]
fn test_lay_out_example() {
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
