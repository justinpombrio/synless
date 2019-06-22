#![feature(slice_patterns)]

mod common;

use common::{assert_strings_eq, make_json_doc, make_json_notation, make_long_json_list, Doc};
use pretty::{
    render_pane, Content, PaneNotation, PaneSize, PlainText, Pos, PrettyDocument, PrettyWindow,
};

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

#[test]
fn test_pane_content() {
    let notations = make_json_notation();

    let doc = Doc::new_branch(notations["true"].clone(), Vec::new());
    let mut window = PlainText::new(Pos { row: 2, col: 10 });

    let pane_note = PaneNotation::Content {
        content: Content::ActiveDoc,
        style: None,
    };
    let mut pane = window.pane().unwrap();
    render_pane(&mut pane, &pane_note, None, |_: &Content| {
        Some(doc.as_ref())
    })
    .unwrap();
    assert_strings_eq(&window.to_string(), "true");
}

#[test]
fn test_pane_horz() {
    let notations = make_json_notation();

    let doc1 = Doc::new_branch(notations["true"].clone(), Vec::new());
    let doc2 = Doc::new_branch(notations["false"].clone(), Vec::new());
    let mut window = PlainText::new(Pos { row: 2, col: 10 });

    let content1 = PaneNotation::Content {
        content: Content::ActiveDoc,
        style: None,
    };
    let content2 = PaneNotation::Content {
        content: Content::KeyHints,
        style: None,
    };

    let pane_note = PaneNotation::Horz {
        panes: vec![
            (PaneSize::Fixed(5), content1),
            (PaneSize::Fixed(5), content2),
        ],
        style: None,
    };

    let mut pane = window.pane().unwrap();
    render_pane(
        &mut pane,
        &pane_note,
        None,
        |content: &Content| match content {
            Content::ActiveDoc => Some(doc1.as_ref()),
            Content::KeyHints => Some(doc2.as_ref()),
        },
    )
    .unwrap();
    assert_strings_eq(&window.to_string(), "true false");
}

#[test]
fn test_pane_vert() {
    let notations = make_json_notation();

    let doc1 = Doc::new_branch(notations["true"].clone(), Vec::new());
    let doc2 = Doc::new_branch(notations["false"].clone(), Vec::new());
    let mut window = PlainText::new(Pos { row: 2, col: 10 });

    let content1 = PaneNotation::Content {
        content: Content::ActiveDoc,
        style: None,
    };
    let content2 = PaneNotation::Content {
        content: Content::KeyHints,
        style: None,
    };

    let top_row_note = PaneNotation::Horz {
        panes: vec![
            (PaneSize::Fixed(4), content1.clone()),
            (PaneSize::Fixed(6), content2.clone()),
        ],
        style: None,
    };

    let bot_row_note = PaneNotation::Horz {
        panes: vec![
            (PaneSize::Proportional(1), content1),
            (PaneSize::Proportional(1), content2),
        ],
        style: None,
    };

    let pane_note = PaneNotation::Vert {
        panes: vec![
            (PaneSize::Fixed(1), top_row_note),
            (PaneSize::Fixed(1), bot_row_note),
        ],
        style: None,
    };

    let mut pane = window.pane().unwrap();
    render_pane(
        &mut pane,
        &pane_note,
        None,
        |content: &Content| match content {
            Content::ActiveDoc => Some(doc1.as_ref()),
            Content::KeyHints => Some(doc2.as_ref()),
        },
    )
    .unwrap();
    assert_strings_eq(&window.to_string(), "truefalse\ntrue false");
}
