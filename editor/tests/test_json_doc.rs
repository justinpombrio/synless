mod common;

use common::make_json_lang;

use editor::{AstForest, Command, CommandGroup, Doc, TreeCmd, TreeNavCmd};
use pretty::{PlainText, PrettyDocument};

// TODO: expand this into a comprehensive test suite

#[test]
fn test_json_undo_redo() {
    let (lang_set, note_set) = make_json_lang();
    let forest = AstForest::new(&lang_set);
    let lang = lang_set.get("json").unwrap();

    let mut doc = Doc::new(
        "MyTestDoc",
        forest.new_fixed_tree(lang, lang.lookup_construct("root"), &note_set),
    );

    assert!(doc.execute(CommandGroup::Group(vec![Command::TreeNav(
        TreeNavCmd::Child(0)
    ),])));

    assert!(doc.execute(CommandGroup::Group(vec![
        Command::Tree(TreeCmd::Replace(forest.new_flexible_tree(
            &lang,
            lang.lookup_construct("list"),
            &note_set,
        ))),
        Command::Tree(TreeCmd::InsertPrepend(forest.new_fixed_tree(
            &lang,
            lang.lookup_construct("true"),
            &note_set,
        ))),
    ])));

    assert!(doc.execute(CommandGroup::Group(vec![Command::Tree(
        TreeCmd::InsertAfter(forest.new_fixed_tree(
            &lang,
            lang.lookup_construct("null"),
            &note_set
        ),)
    )])));
    assert_render(&doc, "[true, null]");

    assert!(doc.execute(CommandGroup::Group(vec![Command::Tree(
        TreeCmd::InsertBefore(forest.new_fixed_tree(
            &lang,
            lang.lookup_construct("false"),
            &note_set
        ),)
    )])));
    assert_render(&doc, "[true, false, null]");

    assert!(doc.execute(CommandGroup::Undo));
    assert_render(&doc, "[true, null]");

    assert!(doc.execute(CommandGroup::Undo));
    assert_render(&doc, "[true]");

    assert!(doc.execute(CommandGroup::Redo));
    assert_render(&doc, "[true, null]");

    assert!(doc.execute(CommandGroup::Redo));
    assert_render(&doc, "[true, false, null]");

    assert!(doc.execute(CommandGroup::Undo));
    assert_render(&doc, "[true, null]");

    assert!(doc.execute(CommandGroup::Group(vec![Command::Tree(
        TreeCmd::InsertAfter(
            // forest.new_fixed_tree(&lang, lang.lookup_construct("false"), &note_set),
            forest.new_flexible_tree(&lang, lang.lookup_construct("list"), &note_set,)
        )
    )])));
    assert_render(&doc, "[true, null, []]");

    assert!(doc.execute(CommandGroup::Undo));
    assert_render(&doc, "[true, null]");

    assert!(doc.execute(CommandGroup::Undo));
    assert_render(&doc, "[true]");

    assert!(doc.execute(CommandGroup::Undo));
    assert_render(&doc, "?");

    assert!(doc.execute(CommandGroup::Redo));
    assert_render(&doc, "[true]");

    assert!(doc.execute(CommandGroup::Redo));
    assert_render(&doc, "[true, null]");

    assert!(doc.execute(CommandGroup::Redo));
    assert_render(&doc, "[true, null, []]");

    assert!(!doc.execute(CommandGroup::Redo));
    assert_render(&doc, "[true, null, []]");
}

fn assert_render(doc: &Doc, rendered: &str) {
    let width: u16 = 80;
    let mut screen = PlainText::new(width as usize);
    doc.ast_ref().pretty_print(width, &mut screen).unwrap();
    assert_eq!(screen.to_string(), rendered)
}
