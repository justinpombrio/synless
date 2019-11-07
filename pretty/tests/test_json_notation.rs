#![feature(slice_patterns)]

mod common;

use common::{
    assert_strings_eq, make_json_doc, make_json_notation, make_long_json_list,
    make_short_json_list, Doc,
};
use pretty::{
    Col, CursorVis, DocLabel, DocPosSpec, PaneNotation, PaneSize, PlainText, Pos, PrettyDocument,
    PrettyWindow, Style,
};

// TODO: test ScrollStrategies other than Beginning.
// TODO: test horz concat

#[test]
fn test_pretty_print_very_small_screen_left() {
    let doc = make_json_doc();
    let doc_pos_spec = DocPosSpec::Fixed(Pos { row: 2, col: 4 });
    let mut window = PlainText::new(Pos { row: 6, col: 6 });
    doc.as_ref()
        .pretty_print(
            80,
            &mut window.pane().unwrap(),
            doc_pos_spec,
            CursorVis::Hide,
        )
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
    let doc_pos_spec = DocPosSpec::Fixed(Pos { row: 2, col: 4 });
    let mut window = PlainText::new(Pos { row: 6, col: 8 });

    doc.as_ref()
        .pretty_print(
            80,
            &mut window.pane().unwrap(),
            doc_pos_spec,
            CursorVis::Hide,
        )
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
fn test_pretty_print_short_list() {
    let doc = make_short_json_list();
    let mut window = PlainText::new_infinite_scroll(80);
    let doc_pos_spec = DocPosSpec::Fixed(Pos::zero());
    doc.as_ref()
        .pretty_print(
            80,
            &mut window.pane().unwrap(),
            doc_pos_spec,
            CursorVis::Hide,
        )
        .unwrap();
    assert_strings_eq(&window.to_string(), r#"[true, false]"#);
}

#[test]
fn test_pretty_print_long_list() {
    let doc = make_long_json_list();
    let mut window = PlainText::new_infinite_scroll(80);
    let doc_pos_spec = DocPosSpec::Fixed(Pos::zero());
    doc.as_ref()
        .pretty_print(
            80,
            &mut window.pane().unwrap(),
            doc_pos_spec,
            CursorVis::Hide,
        )
        .unwrap();
    assert_strings_eq(
        &window.to_string(),
        r#"[true, false, true, true, false, true, false, true, true, false, true, false,
 true, true, false, true, false, true, false, true, true, false, true, false,
 true, true, false, true, false, true, true, false, true, false, true, false,
 true, true, false, true, false, true, true, false, true, false, true, true,
 false, true, false]"#,
    );
}

#[test]
fn test_pretty_print_small_screen_right() {
    let doc = make_json_doc();
    let doc_pos_spec = DocPosSpec::Fixed(Pos { row: 7, col: 13 });
    let mut window = PlainText::new(Pos { row: 4, col: 16 });

    doc.as_ref()
        .pretty_print(
            80,
            &mut window.pane().unwrap(),
            doc_pos_spec,
            CursorVis::Hide,
        )
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
    let doc_pos_spec = DocPosSpec::Fixed(Pos { row: 1, col: 4 });
    let mut window = PlainText::new(Pos { row: 1, col: 1 });

    doc.as_ref()
        .pretty_print(
            80,
            &mut window.pane().unwrap(),
            doc_pos_spec,
            CursorVis::Hide,
        )
        .unwrap();
    assert_strings_eq(&window.to_string(), "i");
}

#[test]
fn test_pretty_print_small_screen_bottom() {
    let doc = make_json_doc();
    // Go past the bottom right corner of the document
    let doc_pos_spec = DocPosSpec::Fixed(Pos { row: 27, col: 63 });
    let mut window = PlainText::new(Pos { row: 4, col: 13 });

    doc.as_ref()
        .pretty_print(
            74,
            &mut window.pane().unwrap(),
            doc_pos_spec,
            CursorVis::Hide,
        )
        .unwrap();
    assert_strings_eq(&window.to_string(), "longer\"]]]]");
}

#[test]
fn test_lay_out_json_80() {
    let doc = make_json_doc();
    let doc_pos_spec = DocPosSpec::Fixed(Pos::zero());
    let mut window = PlainText::new_infinite_scroll(80);
    doc.as_ref()
        .pretty_print(
            80,
            &mut window.pane().unwrap(),
            doc_pos_spec,
            CursorVis::Hide,
        )
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
fn test_string() {
    let notations = make_json_notation();
    let doc = Doc::new_leaf(notations["string"].clone(), "foobar");

    let doc_pos_spec = DocPosSpec::Fixed(Pos::zero());
    let mut window = PlainText::new_infinite_scroll(30);
    doc.as_ref()
        .pretty_print(
            30,
            &mut window.pane().unwrap(),
            doc_pos_spec,
            CursorVis::Hide,
        )
        .unwrap();

    assert_strings_eq(&window.to_string(), "\"foobar\"");
}

#[test]
fn test_string_in_list() {
    let notations = make_json_notation();
    let s = Doc::new_leaf(notations["string"].clone(), "foobar");
    let doc = Doc::new_branch(notations["list"].clone(), vec![s]);

    let doc_pos_spec = DocPosSpec::Fixed(Pos::zero());
    let mut window = PlainText::new_infinite_scroll(30);
    doc.as_ref()
        .pretty_print(
            30,
            &mut window.pane().unwrap(),
            doc_pos_spec,
            CursorVis::Hide,
        )
        .unwrap();

    assert_strings_eq(&window.to_string(), "[\"foobar\"]");
}

#[test]
fn test_dict_in_list() {
    let notations = make_json_notation();
    let boolean = Doc::new_branch(notations["true"].clone(), Vec::new());
    let dict = Doc::new_branch(notations["dict"].clone(), vec![boolean]);
    let doc = Doc::new_branch(notations["list"].clone(), vec![dict]);

    let doc_pos_spec = DocPosSpec::Fixed(Pos::zero());
    let mut window = PlainText::new_infinite_scroll(30);
    doc.as_ref()
        .pretty_print(
            30,
            &mut window.pane().unwrap(),
            doc_pos_spec,
            CursorVis::Hide,
        )
        .unwrap();

    assert_strings_eq(&window.to_string(), "[\"foobar\"]");
}

#[test]
fn test_lay_out_json_30() {
    let doc = make_json_doc();
    let doc_pos_spec = DocPosSpec::Fixed(Pos::zero());
    let mut window = PlainText::new_infinite_scroll(30);
    doc.as_ref()
        .pretty_print(
            30,
            &mut window.pane().unwrap(),
            doc_pos_spec,
            CursorVis::Hide,
        )
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
    let doc_pos_spec = DocPosSpec::Fixed(Pos::zero());
    let mut window = PlainText::new_infinite_scroll(28);
    doc.as_ref()
        .pretty_print(
            28,
            &mut window.pane().unwrap(),
            doc_pos_spec,
            CursorVis::Hide,
        )
        .unwrap();
}

#[test]
fn test_pane_content() {
    let notations = make_json_notation();

    let doc = Doc::new_branch(notations["true"].clone(), Vec::new());
    let mut window = PlainText::new(Pos { row: 2, col: 10 });

    let pane_note = PaneNotation::Doc {
        label: DocLabel::ActiveDoc,
        cursor_visibility: CursorVis::Hide,
        scroll_strategy: DocPosSpec::Beginning,
    };
    let mut pane = window.pane().unwrap();
    pane.render(&pane_note, |_: &DocLabel| Some(doc.as_ref()))
        .unwrap();
    assert_strings_eq(&window.to_string(), "true");
}

#[test]
fn test_pane_horz() {
    let notations = make_json_notation();

    let doc1 = Doc::new_branch(notations["true"].clone(), Vec::new());
    let doc2 = Doc::new_branch(notations["false"].clone(), Vec::new());
    let mut window = PlainText::new(Pos { row: 2, col: 10 });

    let content1 = PaneNotation::Doc {
        label: DocLabel::ActiveDoc,
        cursor_visibility: CursorVis::Hide,
        scroll_strategy: DocPosSpec::Beginning,
    };
    let content2 = PaneNotation::Doc {
        label: DocLabel::KeyHints,
        cursor_visibility: CursorVis::Hide,
        scroll_strategy: DocPosSpec::Beginning,
    };

    let pane_note = PaneNotation::Horz {
        panes: vec![
            (PaneSize::Fixed(5), content1),
            (PaneSize::Fixed(5), content2),
        ],
    };

    let mut pane = window.pane().unwrap();
    pane.render(&pane_note, |label: &DocLabel| match label {
        DocLabel::ActiveDoc => Some(doc1.as_ref()),
        DocLabel::KeyHints => Some(doc2.as_ref()),
        _ => None,
    })
    .unwrap();
    assert_strings_eq(&window.to_string(), "true false");
}

#[test]
fn test_pane_vert() {
    let notations = make_json_notation();

    let doc1 = Doc::new_branch(notations["true"].clone(), Vec::new());
    let doc2 = Doc::new_branch(notations["false"].clone(), Vec::new());
    let mut window = PlainText::new(Pos { row: 2, col: 10 });

    let content1 = PaneNotation::Doc {
        label: DocLabel::ActiveDoc,
        cursor_visibility: CursorVis::Hide,
        scroll_strategy: DocPosSpec::Beginning,
    };
    let content2 = PaneNotation::Doc {
        label: DocLabel::KeyHints,
        cursor_visibility: CursorVis::Hide,
        scroll_strategy: DocPosSpec::Beginning,
    };

    let top_row_note = PaneNotation::Horz {
        panes: vec![
            (PaneSize::Fixed(4), content1.clone()),
            (PaneSize::Fixed(6), content2.clone()),
        ],
    };

    let bot_row_note = PaneNotation::Horz {
        panes: vec![
            (PaneSize::Proportional(1), content1),
            (PaneSize::Proportional(1), content2),
        ],
    };

    let pane_note = PaneNotation::Vert {
        panes: vec![
            (PaneSize::Fixed(1), top_row_note),
            (PaneSize::Fixed(1), bot_row_note),
        ],
    };

    let mut pane = window.pane().unwrap();
    pane.render(&pane_note, |label: &DocLabel| match label {
        DocLabel::ActiveDoc => Some(doc1.as_ref()),
        DocLabel::KeyHints => Some(doc2.as_ref()),
        _ => None,
    })
    .unwrap();
    assert_strings_eq(&window.to_string(), "truefalse\ntrue false");
}

#[test]
fn test_pane_fill() {
    let notations = make_json_notation();

    let doc1 = Doc::new_branch(notations["true"].clone(), Vec::new());
    let mut window = PlainText::new(Pos { row: 3, col: 6 });

    let content1 = PaneNotation::Doc {
        label: DocLabel::ActiveDoc,
        cursor_visibility: CursorVis::Hide,
        scroll_strategy: DocPosSpec::Beginning,
    };

    let fill = PaneNotation::Fill {
        ch: '-',
        style: Style::plain(),
    };

    let pane_note = PaneNotation::Vert {
        panes: vec![(PaneSize::Fixed(1), content1), (PaneSize::Fixed(2), fill)],
    };

    let mut pane = window.pane().unwrap();
    pane.render(&pane_note, |_label: &DocLabel| Some(doc1.as_ref()))
        .unwrap();
    assert_strings_eq(&window.to_string(), "true\n------\n------");
}

fn assert_proportional(expected: &str, width: Col, hungers: (usize, usize, usize)) {
    let notations = make_json_notation();

    let doc1 = Doc::new_branch(notations["null"].clone(), Vec::new());

    let content1 = PaneNotation::Doc {
        label: DocLabel::ActiveDoc,
        cursor_visibility: CursorVis::Hide,
        scroll_strategy: DocPosSpec::Beginning,
    };

    let fill1 = PaneNotation::Fill {
        ch: '1',
        style: Style::plain(),
    };

    let fill2 = PaneNotation::Fill {
        ch: '2',
        style: Style::plain(),
    };

    let pane_note = PaneNotation::Horz {
        panes: vec![
            (PaneSize::Proportional(hungers.0), content1),
            (PaneSize::Proportional(hungers.1), fill1),
            (PaneSize::Proportional(hungers.2), fill2),
        ],
    };

    let mut window = PlainText::new(Pos { row: 1, col: width });
    let mut pane = window.pane().unwrap();
    pane.render(&pane_note, |_label: &DocLabel| Some(doc1.as_ref()))
        .unwrap();
    assert_strings_eq(&window.to_string(), expected);
}

#[test]
fn test_pane_proportional() {
    assert_proportional("null11112222", 12, (4, 4, 4));
    assert_proportional("null11112222", 12, (1, 1, 1));
    assert_proportional("null  111222", 12, (2, 1, 1));
    assert_proportional("null11222222", 12, (4, 2, 6));
    assert_proportional("null22222222", 12, (1, 0, 2));
}

#[test]
fn test_pane_dyn_height() {
    let notations = make_json_notation();

    let elem = Doc::new_branch(notations["true"].clone(), Vec::new());
    let doc0 = Doc::new_branch(notations["list"].clone(), vec![]);
    let doc1 = Doc::new_branch(notations["list"].clone(), vec![elem.clone()]);
    let doc2 = Doc::new_branch(notations["list"].clone(), vec![elem.clone(), elem.clone()]);
    let doc3 = Doc::new_branch(
        notations["list"].clone(),
        vec![elem.clone(), elem.clone(), elem],
    );

    let content1 = PaneNotation::Doc {
        label: DocLabel::ActiveDoc,
        cursor_visibility: CursorVis::Hide,
        scroll_strategy: DocPosSpec::Beginning,
    };

    let fill = PaneNotation::Fill {
        ch: '-',
        style: Style::plain(),
    };

    let pane_note = PaneNotation::Vert {
        panes: vec![
            (PaneSize::DynHeight, content1),
            (PaneSize::Proportional(1), fill),
        ],
    };

    let assert_render = |doc: Doc, expected: &str| {
        let mut window = PlainText::new(Pos { row: 3, col: 6 });
        let mut pane = window.pane().unwrap();
        pane.render(&pane_note, |_label: &DocLabel| Some(doc.as_ref()))
            .unwrap();
        assert_strings_eq(&window.to_string(), expected);
    };

    assert_render(doc0, "[]\n------\n------");
    assert_render(doc1, "[true]\n------\n------");
    assert_render(doc2, "[true,\n true]\n------");
    assert_render(doc3, "[true,\n true,\n true]");
}

// TODO this test currently panics, but should eventually be enabled
// #[test]
#[allow(dead_code)]
fn test_print_outside() {
    let notations = make_json_notation();

    let doc1 = Doc::new_branch(notations["false"].clone(), Vec::new());
    let mut window = PlainText::new(Pos { row: 1, col: 8 });

    let content1 = PaneNotation::Doc {
        label: DocLabel::ActiveDoc,
        cursor_visibility: CursorVis::Hide,
        scroll_strategy: DocPosSpec::Beginning,
    };
    let content2 = PaneNotation::Fill {
        ch: '-',
        style: Style::plain(),
    };

    let pane_note = PaneNotation::Horz {
        panes: vec![
            (PaneSize::Fixed(4), content1),
            (PaneSize::Fixed(4), content2),
        ],
    };

    let mut pane = window.pane().unwrap();
    pane.render(&pane_note, |_label: &DocLabel| Some(doc1.as_ref()))
        .unwrap();
    assert_strings_eq(&window.to_string(), "fals----");
}
