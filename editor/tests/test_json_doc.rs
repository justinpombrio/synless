use editor::{make_json_lang, CommandGroup, TestEditor, TextCmd, TextNavCmd, TreeCmd, TreeNavCmd};
use pretty::{DocPosSpec, Pos};

/// Create a CommandGroup containing any number of commands, each of which are
/// any of the nested command enum types.
macro_rules! group {
    ($($command:expr),+) => {
        CommandGroup::Group(vec![$($command.into()),+])
    }
}

// TODO: expand this into a comprehensive test suite

/// Regression test for an old bug in the Clone implementation.
#[test]
fn test_tree_clone_panic() {
    let (lang, note_set) = make_json_lang();
    let (lang_set, lang_name) = TestEditor::lang_set_from(lang);
    let mut ed = TestEditor::new(&lang_set, &note_set, lang_name);
    ed.exec(TreeNavCmd::Child(0)).unwrap();
    ed.exec(TreeCmd::Replace(ed.node("entry").unwrap()))
        .unwrap();
}

#[test]
fn test_json_undo_redo() {
    let (lang, note_set) = make_json_lang();
    let (lang_set, lang_name) = TestEditor::lang_set_from(lang);
    let mut ed = TestEditor::new(&lang_set, &note_set, lang_name);

    ed.exec(TreeNavCmd::Child(0)).unwrap();

    ed.exec(group![
        TreeCmd::Replace(ed.node("list").unwrap()),
        TreeCmd::InsertHolePrepend,
        TreeCmd::Replace(ed.node("true").unwrap())
    ])
    .unwrap();

    ed.exec(group![
        TreeCmd::InsertHoleAfter,
        TreeCmd::Replace(ed.node("null").unwrap())
    ])
    .unwrap();
    ed.assert_render("[true, null]");

    ed.exec(group![
        TreeCmd::InsertHoleBefore,
        TreeCmd::Replace(ed.node("false").unwrap())
    ])
    .unwrap();

    ed.assert_render("[true, false, null]");

    ed.exec(group![TreeNavCmd::Left]).unwrap();
    ed.exec(CommandGroup::Undo).unwrap();
    ed.assert_render("[true, null]");

    ed.exec(CommandGroup::Undo).unwrap();
    ed.assert_render("[true]");

    ed.exec(CommandGroup::Redo).unwrap();
    ed.assert_render("[true, null]");

    ed.exec(CommandGroup::Redo).unwrap();
    ed.assert_render("[true, false, null]");

    ed.exec(CommandGroup::Undo).unwrap();
    ed.assert_render("[true, null]");

    ed.exec(group![
        TreeCmd::InsertHoleAfter,
        TreeCmd::Replace(ed.node("list").unwrap())
    ])
    .unwrap();
    ed.assert_render("[true, null, []]");

    ed.exec(CommandGroup::Undo).unwrap();
    ed.assert_render("[true, null]");

    ed.exec(CommandGroup::Undo).unwrap();
    ed.assert_render("[true]");

    ed.exec(CommandGroup::Undo).unwrap();
    ed.assert_render("?");

    ed.exec(CommandGroup::Redo).unwrap();
    ed.assert_render("[true]");

    ed.exec(CommandGroup::Redo).unwrap();
    ed.assert_render("[true, null]");

    ed.exec(CommandGroup::Redo).unwrap();
    ed.assert_render("[true, null, []]");

    assert!(ed.exec(CommandGroup::Redo).is_err());
    ed.assert_render("[true, null, []]");
}

#[test]
fn test_json_cursor_at_top() {
    let width = 7;
    let (lang, note_set) = make_json_lang();
    let (lang_set, lang_name) = TestEditor::lang_set_from(lang);
    let mut ed = TestEditor::new(&lang_set, &note_set, lang_name);

    ed.exec(TreeNavCmd::Child(0)).unwrap();

    ed.exec(group![
        TreeCmd::Replace(ed.node("list").unwrap()),
        TreeCmd::InsertHolePrepend,
        TreeCmd::Replace(ed.node("true").unwrap()),
        TreeCmd::InsertHoleAfter,
        TreeCmd::Replace(ed.node("false").unwrap()),
        TreeCmd::InsertHoleAfter,
        TreeCmd::Replace(ed.node("null").unwrap())
    ])
    .unwrap();

    ed.assert_render_with(
        "[true,\n false,\n null]",
        width,
        DocPosSpec::Fixed(Pos::zero()),
    );

    ed.assert_render_with(" null]", width, DocPosSpec::CursorAtTop);

    ed.exec(TreeNavCmd::Left).unwrap();
    ed.assert_render_with(" false,\n null]", width, DocPosSpec::CursorAtTop);

    ed.exec(TreeNavCmd::Left).unwrap();
    ed.assert_render_with("[true,\n false,\n null]", width, DocPosSpec::CursorAtTop);

    ed.exec(TreeNavCmd::Parent).unwrap();
    ed.assert_render_with("[true,\n false,\n null]", width, DocPosSpec::CursorAtTop);
}

#[test]
fn test_json_string() {
    let (lang, note_set) = make_json_lang();
    let (lang_set, lang_name) = TestEditor::lang_set_from(lang);
    let mut ed = TestEditor::new(&lang_set, &note_set, lang_name);

    ed.exec(TreeNavCmd::Child(0)).unwrap();
    ed.exec(TreeCmd::Replace(ed.node("list").unwrap())).unwrap();
    ed.exec(group![
        TreeCmd::InsertHolePrepend,
        TreeCmd::Replace(ed.node("string").unwrap())
    ])
    .unwrap();
    assert!(ed.doc.in_tree_mode());

    ed.exec(TreeNavCmd::Child(0)).unwrap();
    assert!(!ed.doc.in_tree_mode());

    ed.exec(TextNavCmd::TreeMode).unwrap();
    assert!(ed.doc.in_tree_mode());

    ed.exec(TreeNavCmd::Child(0)).unwrap();
    assert!(!ed.doc.in_tree_mode());

    ed.exec(TextCmd::InsertChar('a')).unwrap();
    ed.assert_render("[\"a\"]");
}
