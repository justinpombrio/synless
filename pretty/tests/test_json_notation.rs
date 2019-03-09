#![feature(slice_patterns)]

mod common;

use common::{assert_strings_eq, make_json_doc};
use pretty::{Bound, PlainText, Pos, PrettyDocument, Region};

// TODO: test horz concat
#[test]
fn test_pretty_print_small_screen_left() {
    let doc = make_json_doc();
    let mut screen = PlainText::new_bounded(Region {
        pos: Pos { row: 2, col: 4 },
        bound: Bound::new_rectangle(6, 8),
    });
    doc.as_ref().pretty_print(80, &mut screen).unwrap();
    assert_strings_eq(
        &screen.to_string(),
        r#"astName"
sAlive":
ge": 27,
ddress":
{
  "stree"#,
    );
}

#[test]
fn test_pretty_print_small_screen_right() {
    let doc = make_json_doc();
    let mut screen = PlainText::new_bounded(Region {
        pos: Pos { row: 7, col: 13 },
        bound: Bound::new_rectangle(4, 16),
    });
    doc.as_ref().pretty_print(80, &mut screen).unwrap();
    assert_strings_eq(
        &screen.to_string(),
        r#"Address": "21 2n
 "New York",
: "NY",
Code": "10021-31"#,
    );
}

#[test]
fn test_pretty_print_small_screen_middle() {
    let doc = make_json_doc();
    let mut screen = PlainText::new_bounded(Region::char_region(Pos { row: 1, col: 4 }));
    doc.as_ref().pretty_print(80, &mut screen).unwrap();
    assert_strings_eq(&screen.to_string(), "i");
}

#[test]
fn test_pretty_print_small_screen_bottom() {
    let doc = make_json_doc();
    let mut screen = PlainText::new_bounded(Region {
        pos: Pos { row: 27, col: 63 },
        // Go past the bottom right corner of the document
        bound: Bound::new_rectangle(4, 13),
    });
    doc.as_ref().pretty_print(74, &mut screen).unwrap();
    assert_strings_eq(&screen.to_string(), "longer\"]]]]");
}

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
    // Eventually the strings should wrap, so it
    let doc = make_json_doc();
    doc.write(28);
}
