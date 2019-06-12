use editor::{
    make_json_lang, AstForest, Command, CommandGroup, Doc, TextCmd, TextNavCmd, TreeCmd, TreeNavCmd,
};
use language::LanguageSet;
use pretty::{PlainText, PrettyDocument};

// TODO: expand this into a comprehensive test suite

#[test]
fn test_json_undo_redo() {
    let (lang, note_set) = make_json_lang();
    let name = lang.name().to_string();
    let lang_set = LanguageSet::new();
    lang_set.insert(name.clone(), lang);
    let forest = AstForest::new(&lang_set);
    let lang = lang_set.get(&name).unwrap();

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

#[test]
fn test_json_string() {
    let (lang, note_set) = make_json_lang();
    let name = lang.name().to_string();
    let lang_set = LanguageSet::new();
    lang_set.insert(name.clone(), lang);
    let forest = AstForest::new(&lang_set);
    let lang = lang_set.get(&name).unwrap();

    let mut doc = Doc::new(
        "MyTestDoc",
        forest.new_fixed_tree(&lang, lang.lookup_construct("root"), &note_set),
    );

    assert!(doc.execute(CommandGroup::Group(vec![Command::TreeNav(
        TreeNavCmd::Child(0)
    ),])));

    assert!(
        doc.execute(CommandGroup::Group(vec![Command::Tree(TreeCmd::Replace(
            forest.new_flexible_tree(&lang, lang.lookup_construct("list"), &note_set,)
        )),]))
    );

    assert!(doc.execute(CommandGroup::Group(vec![Command::Tree(
        TreeCmd::InsertPrepend(forest.new_text_tree(
            &lang,
            lang.lookup_construct("string"),
            &note_set,
        ))
    ),])));

    assert!(doc.is_tree_mode());

    assert!(doc.execute(CommandGroup::Group(vec![Command::TreeNav(
        TreeNavCmd::Child(0)
    ),])));
    assert!(!doc.is_tree_mode());
    assert!(doc.execute(CommandGroup::Group(vec![Command::TextNav(
        TextNavCmd::TreeMode
    ),])));
    assert!(doc.is_tree_mode());
    assert!(doc.execute(CommandGroup::Group(vec![Command::TreeNav(
        TreeNavCmd::Child(0)
    ),])));
    assert!(!doc.is_tree_mode());
    assert!(doc.execute(CommandGroup::Group(vec![Command::Text(
        TextCmd::InsertChar('a')
    ),])));
    assert_render(&doc, "\"a\"");
}

fn assert_render(doc: &Doc, rendered: &str) {
    let width: u16 = 80;
    let mut screen = PlainText::new(width as usize);
    doc.ast_ref().pretty_print(width, &mut screen).unwrap();
    assert_eq!(screen.to_string(), rendered)
}
