use editor::{
    make_json_lang, AstForest, Clipboard, Command, CommandGroup, Doc, TextCmd, TextNavCmd, TreeCmd,
    TreeNavCmd,
};
use language::LanguageSet;
use pretty::{CursorVis, DocPosSpec, PlainText, Pos, PrettyDocument, PrettyWindow};

// TODO: expand this into a comprehensive test suite
#[test]
fn test_tree_clone_panic() {
    let (lang, note_set) = make_json_lang();
    let name = lang.name().to_string();
    let lang_set = LanguageSet::new();
    lang_set.insert(name.clone(), lang);
    let forest = AstForest::new(&lang_set);
    let lang = lang_set.get(&name).unwrap();
    let mut clipboard = Clipboard::new();
    let mut doc = Doc::new(
        "MyTestDoc",
        forest.new_fixed_tree(lang, lang.lookup_construct("root"), &note_set),
    );

    doc.execute(
        CommandGroup::Group(vec![Command::TreeNav(TreeNavCmd::Child(0))]),
        &mut clipboard,
    )
    .unwrap();

    doc.execute(
        CommandGroup::Group(vec![Command::Tree(TreeCmd::Replace(
            forest.new_fixed_tree(&lang, lang.lookup_construct("entry"), &note_set),
        ))]),
        &mut clipboard,
    )
    .unwrap();
}

#[test]
fn test_json_undo_redo() {
    let (lang, note_set) = make_json_lang();
    let name = lang.name().to_string();
    let lang_set = LanguageSet::new();
    lang_set.insert(name.clone(), lang);
    let forest = AstForest::new(&lang_set);
    let lang = lang_set.get(&name).unwrap();
    let mut clipboard = Clipboard::new();
    let mut doc = Doc::new(
        "MyTestDoc",
        forest.new_fixed_tree(lang, lang.lookup_construct("root"), &note_set),
    );

    doc.execute(
        CommandGroup::Group(vec![Command::TreeNav(TreeNavCmd::Child(0))]),
        &mut clipboard,
    )
    .unwrap();

    doc.execute(
        CommandGroup::Group(vec![
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
        ]),
        &mut clipboard,
    )
    .unwrap();

    doc.execute(
        CommandGroup::Group(vec![Command::Tree(TreeCmd::InsertAfter(
            forest.new_fixed_tree(&lang, lang.lookup_construct("null"), &note_set),
        ))]),
        &mut clipboard,
    )
    .unwrap();
    assert_render(&doc, "[true, null]");

    doc.execute(
        CommandGroup::Group(vec![Command::Tree(TreeCmd::InsertBefore(
            forest.new_fixed_tree(&lang, lang.lookup_construct("false"), &note_set),
        ))]),
        &mut clipboard,
    )
    .unwrap();
    assert_render(&doc, "[true, false, null]");

    doc.execute(CommandGroup::Undo, &mut clipboard).unwrap();
    assert_render(&doc, "[true, null]");

    doc.execute(CommandGroup::Undo, &mut clipboard).unwrap();
    assert_render(&doc, "[true]");

    doc.execute(CommandGroup::Redo, &mut clipboard).unwrap();
    assert_render(&doc, "[true, null]");

    doc.execute(CommandGroup::Redo, &mut clipboard).unwrap();
    assert_render(&doc, "[true, false, null]");

    doc.execute(CommandGroup::Undo, &mut clipboard).unwrap();
    assert_render(&doc, "[true, null]");

    doc.execute(
        CommandGroup::Group(vec![Command::Tree(TreeCmd::InsertAfter(
            // forest.new_fixed_tree(&lang, lang.lookup_construct("false"), &note_set),
            forest.new_flexible_tree(&lang, lang.lookup_construct("list"), &note_set),
        ))]),
        &mut clipboard,
    )
    .unwrap();
    assert_render(&doc, "[true, null, []]");

    doc.execute(CommandGroup::Undo, &mut clipboard).unwrap();
    assert_render(&doc, "[true, null]");

    doc.execute(CommandGroup::Undo, &mut clipboard).unwrap();
    assert_render(&doc, "[true]");

    doc.execute(CommandGroup::Undo, &mut clipboard).unwrap();
    assert_render(&doc, "?");

    doc.execute(CommandGroup::Redo, &mut clipboard).unwrap();
    assert_render(&doc, "[true]");

    doc.execute(CommandGroup::Redo, &mut clipboard).unwrap();
    assert_render(&doc, "[true, null]");

    doc.execute(CommandGroup::Redo, &mut clipboard).unwrap();
    assert_render(&doc, "[true, null, []]");

    assert!(doc.execute(CommandGroup::Redo, &mut clipboard).is_err());
    assert_render(&doc, "[true, null, []]");
}

#[test]
fn test_json_cursor_at_top() {
    let (lang, note_set) = make_json_lang();
    let name = lang.name().to_string();
    let lang_set = LanguageSet::new();
    lang_set.insert(name.clone(), lang);
    let forest = AstForest::new(&lang_set);
    let lang = lang_set.get(&name).unwrap();
    let mut clipboard = Clipboard::new();
    let mut doc = Doc::new(
        "MyTestDoc",
        forest.new_fixed_tree(lang, lang.lookup_construct("root"), &note_set),
    );

    doc.execute(
        CommandGroup::Group(vec![Command::TreeNav(TreeNavCmd::Child(0))]),
        &mut clipboard,
    )
    .unwrap();

    doc.execute(
        CommandGroup::Group(vec![
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
            Command::Tree(TreeCmd::InsertAfter(forest.new_fixed_tree(
                &lang,
                lang.lookup_construct("false"),
                &note_set,
            ))),
            Command::Tree(TreeCmd::InsertAfter(forest.new_fixed_tree(
                &lang,
                lang.lookup_construct("null"),
                &note_set,
            ))),
        ]),
        &mut clipboard,
    )
    .unwrap();

    let test = |doc: &Doc, spec: DocPosSpec, rendered: &str| {
        let width = 7;
        let mut window = PlainText::new(Pos {
            col: width,
            row: 50,
        });
        doc.ast_ref()
            .pretty_print(width, &mut window.pane().unwrap(), spec, CursorVis::Hide)
            .unwrap();
        assert_eq!(window.to_string(), rendered);
    };

    test(
        &doc,
        DocPosSpec::Fixed(Pos::zero()),
        "[true,\n false,\n null]",
    );
    test(&doc, DocPosSpec::CursorAtTop, " null]");

    doc.execute(
        CommandGroup::Group(vec![Command::TreeNav(TreeNavCmd::Left)]),
        &mut clipboard,
    )
    .unwrap();

    test(&doc, DocPosSpec::CursorAtTop, " false,\n null]");

    doc.execute(
        CommandGroup::Group(vec![Command::TreeNav(TreeNavCmd::Left)]),
        &mut clipboard,
    )
    .unwrap();

    test(&doc, DocPosSpec::CursorAtTop, "[true,\n false,\n null]");

    doc.execute(
        CommandGroup::Group(vec![Command::TreeNav(TreeNavCmd::Parent)]),
        &mut clipboard,
    )
    .unwrap();

    test(&doc, DocPosSpec::CursorAtTop, "[true,\n false,\n null]");
}

#[test]
fn test_json_string() {
    let (lang, note_set) = make_json_lang();
    let name = lang.name().to_string();
    let lang_set = LanguageSet::new();
    lang_set.insert(name.clone(), lang);
    let forest = AstForest::new(&lang_set);
    let lang = lang_set.get(&name).unwrap();
    let mut clipboard = Clipboard::new();

    let mut doc = Doc::new(
        "MyTestDoc",
        forest.new_fixed_tree(&lang, lang.lookup_construct("root"), &note_set),
    );

    doc.execute(
        CommandGroup::Group(vec![Command::TreeNav(TreeNavCmd::Child(0))]),
        &mut clipboard,
    )
    .unwrap();

    doc.execute(
        CommandGroup::Group(vec![Command::Tree(TreeCmd::Replace(
            forest.new_flexible_tree(&lang, lang.lookup_construct("list"), &note_set),
        ))]),
        &mut clipboard,
    )
    .unwrap();

    doc.execute(
        CommandGroup::Group(vec![Command::Tree(TreeCmd::InsertPrepend(
            forest.new_text_tree(&lang, lang.lookup_construct("string"), &note_set),
        ))]),
        &mut clipboard,
    )
    .unwrap();

    assert!(doc.in_tree_mode());

    doc.execute(
        CommandGroup::Group(vec![Command::TreeNav(TreeNavCmd::Child(0))]),
        &mut clipboard,
    )
    .unwrap();
    assert!(!doc.in_tree_mode());
    doc.execute(
        CommandGroup::Group(vec![Command::TextNav(TextNavCmd::TreeMode)]),
        &mut clipboard,
    )
    .unwrap();
    assert!(doc.in_tree_mode());
    doc.execute(
        CommandGroup::Group(vec![Command::TreeNav(TreeNavCmd::Child(0))]),
        &mut clipboard,
    )
    .unwrap();
    assert!(!doc.in_tree_mode());
    doc.execute(
        CommandGroup::Group(vec![Command::Text(TextCmd::InsertChar('a'))]),
        &mut clipboard,
    )
    .unwrap();
    assert_render(&doc, "\"a\"");
}

fn assert_render(doc: &Doc, rendered: &str) {
    let width = 80;
    let doc_pos = DocPosSpec::Fixed(Pos::zero());
    let mut window = PlainText::new_infinite_scroll(width);
    doc.ast_ref()
        .pretty_print(width, &mut window.pane().unwrap(), doc_pos, CursorVis::Hide)
        .unwrap();
    assert_eq!(window.to_string(), rendered)
}
