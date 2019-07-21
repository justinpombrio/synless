#![feature(slice_patterns)]

mod common;

use common::{assert_strings_eq, make_json_doc, make_long_json_list};
use pretty::{PlainText, Pos, PrettyDocument, PrettyWindow};

// TODO: test horz concat

#[test]
fn test_pretty_print_very_small_screen_left() {
    let doc = make_json_doc();
    let doc_pos = Pos { row: 2, col: 4 };
    let mut window = PlainText::new(Pos { row: 6, col: 6 });
    doc.as_ref()
        .pretty_print(80, &mut window.pane().unwrap(), doc_pos)
        .unwrap();
    assert_strings_eq(
        &window.to_string(),
        r#"astNam
sAlive
ge": 2
ddress
{
  "str"#,
    );
}

#[test]
fn test_pretty_print_small_screen_left() {
    let doc = make_json_doc();
    let doc_pos = Pos { row: 2, col: 4 };
    let mut window = PlainText::new(Pos { row: 6, col: 8 });

    doc.as_ref()
        .pretty_print(80, &mut window.pane().unwrap(), doc_pos)
        .unwrap();
    assert_strings_eq(
        &window.to_string(),
        r#"astName"
sAlive":
ge": 27,
ddress":
{
  "stree"#,
    );
}

#[test]
fn test_pretty_print_long_list() {
    let doc = make_long_json_list();
    let mut window = PlainText::new_infinite_scroll(80);
    let doc_pos = Pos::zero();
    doc.as_ref()
        .pretty_print(80, &mut window.pane().unwrap(), doc_pos)
        .unwrap();
    assert_strings_eq(
        &window.to_string(),
        r#"[true, false, true, true, false, true, false, true, true, false, true,
 false, true, true, false, true, false, true, false, true, true,
 false, true, false, true, true, false, true, false, true, true,
 false, true, false, true, false, true, true, false, true, false,
 true, true, false, true, false, true, true, false, true, false]"#,
    );
}

#[test]
fn test_pretty_print_small_screen_right() {
    let doc = make_json_doc();
    let doc_pos = Pos { row: 7, col: 13 };
    let mut window = PlainText::new(Pos { row: 4, col: 16 });

    doc.as_ref()
        .pretty_print(80, &mut window.pane().unwrap(), doc_pos)
        .unwrap();
    assert_strings_eq(
        &window.to_string(),
        r#"Address": "21 2n
 "New York",
: "NY",
Code": "10021-31"#,
    );
}

#[test]
fn test_pretty_print_small_screen_middle() {
    let doc = make_json_doc();
    let doc_pos = Pos { row: 1, col: 4 };
    let mut window = PlainText::new(Pos { row: 1, col: 1 });

    doc.as_ref()
        .pretty_print(80, &mut window.pane().unwrap(), doc_pos)
        .unwrap();
    assert_strings_eq(&window.to_string(), "i");
}

#[test]
fn test_pretty_print_small_screen_bottom() {
    let doc = make_json_doc();
    // Go past the bottom right corner of the document
    let doc_pos = Pos { row: 27, col: 63 };
    let mut window = PlainText::new(Pos { row: 4, col: 13 });

    doc.as_ref()
        .pretty_print(74, &mut window.pane().unwrap(), doc_pos)
        .unwrap();
    assert_strings_eq(&window.to_string(), "longer\"]]]]");
}

#[test]
fn test_lay_out_json_80() {
    let doc = make_json_doc();
    let doc_pos = Pos::zero();
    let mut window = PlainText::new_infinite_scroll(80);
    doc.as_ref()
        .pretty_print(80, &mut window.pane().unwrap(), doc_pos)
        .unwrap();

    assert_strings_eq(
        &window.to_string(),
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
    let doc_pos = Pos::zero();
    let mut window = PlainText::new_infinite_scroll(30);
    doc.as_ref()
        .pretty_print(30, &mut window.pane().unwrap(), doc_pos)
        .unwrap();

    assert_strings_eq(
        &window.to_string(),
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
    // Eventually the strings should wrap, and it should stop panicking.
    let doc = make_json_doc();
    let doc_pos = Pos::zero();
    let mut window = PlainText::new_infinite_scroll(28);
    doc.as_ref()
        .pretty_print(28, &mut window.pane().unwrap(), doc_pos)
        .unwrap();
}
