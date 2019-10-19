use editor::{
    make_json_lang, make_singleton_lang_set, CommandGroup, DocError, EditorCmd, TestEditor,
    TextCmd, TextNavCmd, TreeCmd, TreeNavCmd,
};
use pretty::{DocPosSpec, Pos};

/// Create a CommandGroup containing any number of commands, each of which are
/// any of the nested command enum types.
macro_rules! group {
    ($($command:expr),+) => {
        CommandGroup::Group(vec![$($command.into()),+])
    }
}

/// Check if the expression matches the pattern, and panic with a informative
/// message if it doesn't.
macro_rules! assert_matches {
    ($expression:expr, $pattern:pat) => {
        if let $pattern = $expression {
            ()
        } else {
            panic!(
                "assertion failed: `(expr matches pattern)`\n  expr: {:?}\n  pattern: {:?}",
                $expression,
                stringify!($pattern)
            )
        }
    };
}

// TODO: expand this into a comprehensive test suite

/// Regression test for an old bug in the Clone implementation.
#[test]
fn test_tree_clone_panic() {
    let (lang, note_set) = make_json_lang();
    let (lang_set, lang_name) = make_singleton_lang_set(lang);
    let mut ed = TestEditor::new(&lang_set, &note_set, lang_name);
    ed.exec(TreeNavCmd::Child(0)).unwrap();
    ed.exec(TreeCmd::Replace(ed.node("entry").unwrap()))
        .unwrap();
}

#[test]
fn test_json_undo_redo() {
    let (lang, note_set) = make_json_lang();
    let (lang_set, lang_name) = make_singleton_lang_set(lang);
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

    assert_matches!(ed.exec(CommandGroup::Redo), Err(DocError::NothingToRedo));
    ed.assert_render("[true, null, []]");
}

#[test]
fn test_json_cursor_at_top() {
    let width = 7;
    let (lang, note_set) = make_json_lang();
    let (lang_set, lang_name) = make_singleton_lang_set(lang);
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
    let (lang_set, lang_name) = make_singleton_lang_set(lang);
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

    assert_matches!(
        ed.exec(TextCmd::DeleteCharForward),
        Err(DocError::CannotDeleteChar)
    );
    assert_matches!(
        ed.exec(TextCmd::DeleteCharBackward),
        Err(DocError::CannotDeleteChar)
    );
    assert_matches!(ed.exec(TextNavCmd::Left), Err(DocError::CannotMove));
    assert_matches!(ed.exec(TextNavCmd::Right), Err(DocError::CannotMove));

    ed.exec(TextCmd::InsertChar('a')).unwrap();
    ed.assert_render("[\"a\"]");

    assert_matches!(
        ed.exec(TextCmd::DeleteCharForward),
        Err(DocError::CannotDeleteChar)
    );
    assert_matches!(ed.exec(TextNavCmd::Right), Err(DocError::CannotMove));
    ed.exec(TextNavCmd::Left).unwrap();
    assert_matches!(ed.exec(TextNavCmd::Left), Err(DocError::CannotMove));
    ed.exec(TextNavCmd::Right).unwrap();
    assert_matches!(ed.exec(TextNavCmd::Right), Err(DocError::CannotMove));

    ed.exec(TextCmd::DeleteCharBackward).unwrap();
    ed.assert_render("[\"\"]");

    ed.exec(TextCmd::InsertChar('a')).unwrap();
    ed.exec(TextCmd::InsertChar('b')).unwrap();
    ed.exec(TextCmd::InsertChar('c')).unwrap();
    ed.assert_render("[\"abc\"]");

    assert_matches!(ed.exec(TextNavCmd::Right), Err(DocError::CannotMove));
    ed.exec(TextNavCmd::Left).unwrap();
    ed.exec(TextNavCmd::Left).unwrap();
    ed.exec(TextNavCmd::Left).unwrap();
    assert_matches!(ed.exec(TextNavCmd::Left), Err(DocError::CannotMove));
    ed.exec(TextCmd::DeleteCharForward).unwrap();
    ed.assert_render("[\"bc\"]");

    ed.exec(TextNavCmd::Right).unwrap();
    ed.exec(TextCmd::InsertChar('d')).unwrap();
    ed.assert_render("[\"bdc\"]");

    ed.exec(TextCmd::DeleteCharForward).unwrap();
    ed.assert_render("[\"bd\"]");

    assert_matches!(
        ed.exec(TextCmd::DeleteCharForward),
        Err(DocError::CannotDeleteChar)
    );

    ed.exec(TextCmd::DeleteCharBackward).unwrap();
    ed.assert_render("[\"b\"]");

    ed.exec(TextCmd::DeleteCharBackward).unwrap();
    ed.assert_render("[\"\"]");

    assert_matches!(
        ed.exec(TextCmd::DeleteCharBackward),
        Err(DocError::CannotDeleteChar)
    );
}

#[test]
fn test_insert() {
    let (lang, note_set) = make_json_lang();
    let (lang_set, lang_name) = make_singleton_lang_set(lang);
    let mut ed = TestEditor::new(&lang_set, &note_set, lang_name);

    ed.exec(TreeNavCmd::Child(0)).unwrap();
    ed.exec(TreeCmd::Replace(ed.node("list").unwrap())).unwrap();
    ed.assert_render("[]");

    // Cursor is now on the `[]`, and its parent (the root) isn't flexible:
    assert_matches!(
        ed.exec(TreeCmd::InsertHoleBefore),
        Err(DocError::CannotInsert)
    );
    assert_matches!(
        ed.exec(TreeCmd::InsertHoleAfter),
        Err(DocError::CannotInsert)
    );

    ed.exec(TreeCmd::InsertHolePrepend).unwrap();
    ed.assert_render("[?]");
    ed.exec(TreeCmd::Replace(ed.node("true").unwrap())).unwrap();
    ed.assert_render("[true]");

    // Cursor is now on the `true`, which isn't flexible:
    assert_matches!(
        ed.exec(TreeCmd::InsertHolePrepend),
        Err(DocError::CannotInsert)
    );
    assert_matches!(
        ed.exec(TreeCmd::InsertHolePostpend),
        Err(DocError::CannotInsert)
    );

    ed.exec(TreeCmd::InsertHoleBefore).unwrap();
    ed.assert_render("[?, true]");
    ed.exec(TreeCmd::Replace(ed.node("false").unwrap()))
        .unwrap();
    ed.assert_render("[false, true]");
    ed.exec(TreeCmd::InsertHoleAfter).unwrap();
    ed.assert_render("[false, ?, true]");
    ed.exec(TreeNavCmd::Right).unwrap();
    ed.exec(TreeCmd::InsertHoleAfter).unwrap();
    ed.assert_render("[false, ?, true, ?]");
    ed.exec(TreeCmd::Replace(ed.node("list").unwrap())).unwrap();
    ed.assert_render("[false, ?, true, []]");
    ed.exec(TreeCmd::InsertHolePostpend).unwrap();
    ed.assert_render("[false, ?, true, [?]]");
    ed.exec(TreeCmd::Replace(ed.node("dict").unwrap())).unwrap();
    ed.assert_render("[false, ?, true, [{}]]");
    ed.exec(TreeNavCmd::Parent).unwrap();
    ed.exec(TreeCmd::InsertHolePostpend).unwrap();
    ed.assert_render("[false, ?, true, [{}, ?]]");
    ed.exec(TreeNavCmd::Parent).unwrap();
    ed.exec(TreeCmd::InsertHolePrepend).unwrap();
    ed.assert_render("[false, ?, true, [?, {}, ?]]");
    ed.exec(TreeNavCmd::Parent).unwrap();
    ed.exec(TreeCmd::InsertHoleAfter).unwrap();
    ed.assert_render("[false, ?, true, [?, {}, ?], ?]");

    ed.exec(TreeNavCmd::Left).unwrap();
    ed.exec(TreeNavCmd::Child(1)).unwrap();
    ed.exec(TreeCmd::InsertHolePrepend).unwrap();
    ed.assert_render("[false, ?, true, [?, {?}, ?], ?]");
    ed.exec(TreeCmd::Replace(ed.node("entry").unwrap()))
        .unwrap();
    ed.assert_render("[false, ?, true, [?, {?: ?}, ?], ?]");
    ed.exec(TreeNavCmd::Child(1)).unwrap();
    ed.exec(TreeCmd::Replace(ed.node("null").unwrap())).unwrap();
    ed.assert_render("[false, ?, true, [?, {?: null}, ?], ?]");

    // Can't do any type of insertion when cursor is on dict entry key.
    assert_matches!(
        ed.exec(TreeCmd::InsertHolePrepend),
        Err(DocError::CannotInsert)
    );
    assert_matches!(
        ed.exec(TreeCmd::InsertHolePostpend),
        Err(DocError::CannotInsert)
    );
    assert_matches!(
        ed.exec(TreeCmd::InsertHoleBefore),
        Err(DocError::CannotInsert)
    );
    assert_matches!(
        ed.exec(TreeCmd::InsertHoleAfter),
        Err(DocError::CannotInsert)
    );

    ed.exec(TreeNavCmd::Parent).unwrap();
    ed.exec(TreeCmd::InsertHoleBefore).unwrap();
    ed.assert_render(
        r#"[false,
 ?,
 true,
 [?,
  {
    ?,
    ?: null
  },
  ?],
 ?]"#,
    );
}

#[test]
fn test_remove() {
    let (lang, note_set) = make_json_lang();
    let (lang_set, lang_name) = make_singleton_lang_set(lang);
    let mut ed = TestEditor::new(&lang_set, &note_set, lang_name);

    ed.exec(TreeNavCmd::Child(0)).unwrap();
    ed.exec(TreeCmd::Replace(ed.node("list").unwrap())).unwrap();
    ed.assert_render("[]");
    ed.exec(TreeCmd::InsertHolePostpend).unwrap();
    ed.assert_render("[?]");
    ed.exec(TreeCmd::Remove).unwrap();
    ed.assert_render("[]");
    ed.exec(TreeCmd::InsertHolePrepend).unwrap();
    ed.assert_render("[?]");
    ed.exec(TreeCmd::Replace(ed.node("true").unwrap())).unwrap();
    ed.assert_render("[true]");
    ed.exec(TreeCmd::InsertHoleAfter).unwrap();
    ed.assert_render("[true, ?]");
    ed.exec(TreeCmd::Remove).unwrap();
    ed.assert_render("[true]");
    ed.exec(TreeCmd::InsertHoleAfter).unwrap();
    ed.assert_render("[true, ?]");
    ed.exec(TreeNavCmd::Left).unwrap();
    ed.exec(TreeCmd::Remove).unwrap();
    ed.assert_render("[?]");

    ed.exec(TreeCmd::Replace(ed.node("dict").unwrap())).unwrap();
    ed.assert_render("[{}]");
    ed.exec(TreeCmd::InsertHolePostpend).unwrap();
    ed.assert_render("[{?}]");
    ed.exec(TreeCmd::Remove).unwrap();
    ed.assert_render("[{}]");
    ed.exec(TreeCmd::InsertHolePostpend).unwrap();
    ed.assert_render("[{?}]");
    ed.exec(TreeCmd::Replace(ed.node("entry").unwrap()))
        .unwrap();
    ed.assert_render("[{?: ?}]");
    ed.exec(TreeNavCmd::Child(0)).unwrap();
    assert_matches!(ed.exec(TreeCmd::Remove), Err(DocError::CannotRemoveNode));
    ed.exec(TreeNavCmd::Right).unwrap();
    assert_matches!(ed.exec(TreeCmd::Remove), Err(DocError::CannotRemoveNode));
    ed.assert_render("[{?: ?}]");
    ed.exec(TreeNavCmd::Parent).unwrap();
    ed.exec(TreeNavCmd::Parent).unwrap();
    ed.exec(TreeCmd::Remove).unwrap();
    ed.assert_render("[]");
    assert_matches!(ed.exec(TreeCmd::Remove), Err(DocError::CannotRemoveNode));
    ed.assert_render("[]");
}

#[test]
fn test_cut_copy_paste() {
    let (lang, note_set) = make_json_lang();
    let (lang_set, lang_name) = make_singleton_lang_set(lang);
    let mut ed = TestEditor::new(&lang_set, &note_set, lang_name);

    ed.exec(TreeNavCmd::Child(0)).unwrap();
    ed.assert_render("?");
    assert_eq!(ed.clipboard.len(), 0);

    assert_matches!(ed.exec(EditorCmd::PasteSwap), Err(DocError::EmptyClipboard));
    ed.assert_render("?");
    assert_eq!(ed.clipboard.len(), 0);

    ed.exec(TreeCmd::Replace(ed.node("true").unwrap())).unwrap();
    ed.assert_render("true");

    ed.exec(EditorCmd::Cut).unwrap();
    ed.assert_render("?");
    assert_eq!(ed.clipboard.len(), 1);

    ed.exec(TreeCmd::Replace(ed.node("list").unwrap())).unwrap();
    ed.exec(TreeCmd::InsertHolePrepend).unwrap();
    ed.assert_render("[?]");

    ed.exec(EditorCmd::PasteSwap).unwrap();
    ed.assert_render("[true]");
    assert_eq!(ed.clipboard.len(), 1); // Contains a hole

    ed.exec(TreeCmd::InsertHoleAfter).unwrap();
    ed.assert_render("[true, ?]");

    // Swap the two holes, which is uninteresting
    ed.exec(EditorCmd::PasteSwap).unwrap();
    ed.assert_render("[true, ?]");
    assert_eq!(ed.clipboard.len(), 1);

    ed.exec(EditorCmd::PopClipboard).unwrap();
    assert_eq!(ed.clipboard.len(), 0);
    assert_matches!(
        ed.exec(EditorCmd::PopClipboard),
        Err(DocError::EmptyClipboard)
    );
    assert_eq!(ed.clipboard.len(), 0);

    ed.exec(TreeCmd::Replace(ed.node("false").unwrap()))
        .unwrap();
    ed.exec(TreeCmd::InsertHoleAfter).unwrap();
    ed.exec(TreeCmd::Replace(ed.node("null").unwrap())).unwrap();
    ed.assert_render("[true, false, null]");

    ed.exec(EditorCmd::Cut).unwrap();
    ed.assert_render("[true, false, ?]");
    assert_eq!(ed.clipboard.len(), 1);

    ed.exec(TreeNavCmd::Left).unwrap();
    ed.exec(EditorCmd::Cut).unwrap();
    ed.assert_render("[true, ?, ?]");
    assert_eq!(ed.clipboard.len(), 2);

    ed.exec(TreeNavCmd::Left).unwrap();
    ed.exec(EditorCmd::PasteSwap).unwrap();
    ed.assert_render("[false, ?, ?]");
    assert_eq!(ed.clipboard.len(), 2);

    ed.exec(TreeNavCmd::Right).unwrap();
    ed.exec(TreeNavCmd::Right).unwrap();
    ed.exec(EditorCmd::PasteSwap).unwrap();
    ed.exec(EditorCmd::PopClipboard).unwrap();
    ed.assert_render("[false, ?, true]");
    assert_eq!(ed.clipboard.len(), 1);

    ed.exec(TreeNavCmd::Left).unwrap();
    ed.exec(EditorCmd::PasteSwap).unwrap();
    ed.assert_render("[false, null, true]");
    assert_eq!(ed.clipboard.len(), 1);

    ed.exec(TreeNavCmd::Parent).unwrap();
    ed.exec(EditorCmd::Copy).unwrap();
    ed.assert_render("[false, null, true]");
    assert_eq!(ed.clipboard.len(), 2);

    ed.exec(TreeCmd::InsertHolePostpend).unwrap();
    ed.assert_render("[false, null, true, ?]");

    ed.exec(EditorCmd::PasteSwap).unwrap();
    ed.assert_render("[false, null, true, [false, null, true]]");
    assert_eq!(ed.clipboard.len(), 2);
}

#[test]
fn test_clear() {
    let (lang, note_set) = make_json_lang();
    let (lang_set, lang_name) = make_singleton_lang_set(lang);
    let mut ed = TestEditor::new(&lang_set, &note_set, lang_name);

    ed.exec(TreeNavCmd::Child(0)).unwrap();
    ed.assert_render("?");

    ed.exec(TreeCmd::Clear).unwrap();
    ed.assert_render("?");

    ed.exec(TreeCmd::Replace(ed.node("list").unwrap())).unwrap();
    ed.exec(TreeCmd::InsertHolePostpend).unwrap();
    ed.exec(TreeCmd::Replace(ed.node("true").unwrap())).unwrap();
    ed.exec(TreeCmd::InsertHoleAfter).unwrap();
    ed.exec(TreeCmd::Replace(ed.node("false").unwrap()))
        .unwrap();
    ed.assert_render("[true, false]");

    ed.exec(TreeNavCmd::Left).unwrap();
    ed.exec(TreeCmd::Clear).unwrap();
    ed.assert_render("[?, false]");
    assert_eq!(ed.clipboard.len(), 0);

    ed.exec(TreeCmd::Replace(ed.node("dict").unwrap())).unwrap();
    ed.assert_render("[{}, false]");

    ed.exec(TreeCmd::InsertHolePostpend).unwrap();
    ed.assert_render("[{?}, false]");
    ed.exec(TreeCmd::Replace(ed.node("entry").unwrap()))
        .unwrap();
    ed.assert_render("[{?: ?}, false]");

    ed.exec(TreeNavCmd::Parent).unwrap();
    ed.exec(TreeNavCmd::Parent).unwrap();
    ed.exec(TreeCmd::Clear).unwrap();
    ed.assert_render("?");
}

#[test]
fn test_undo_clipboard() {
    let (lang, note_set) = make_json_lang();
    let (lang_set, lang_name) = make_singleton_lang_set(lang);
    let mut ed = TestEditor::new(&lang_set, &note_set, lang_name);
    assert_eq!(ed.clipboard.len(), 0);

    ed.exec(group![
        TreeNavCmd::Child(0),
        TreeCmd::Replace(ed.node("list").unwrap()),
        TreeCmd::InsertHolePostpend,
        TreeCmd::Replace(ed.node("true").unwrap()),
        TreeCmd::InsertHoleAfter,
        TreeCmd::Replace(ed.node("false").unwrap())
    ])
    .unwrap();
    ed.assert_render("[true, false]");

    ed.exec(EditorCmd::Cut).unwrap();
    ed.assert_render("[true, ?]");
    assert_eq!(ed.clipboard.len(), 1);

    ed.exec(CommandGroup::Undo).unwrap();
    ed.assert_render("[true, false]");
    assert_eq!(ed.clipboard.len(), 1);

    ed.exec(TreeNavCmd::Left).unwrap();
    ed.exec(EditorCmd::PasteSwap).unwrap();
    ed.assert_render("[false, false]");
    assert_eq!(ed.clipboard.len(), 1); // contains `true`

    ed.exec(TreeCmd::InsertHoleAfter).unwrap();
    ed.assert_render("[false, ?, false]");

    ed.exec(CommandGroup::Undo).unwrap();
    ed.assert_render("[false, false]");
    assert_eq!(ed.clipboard.len(), 1); // contains `true`

    ed.exec(CommandGroup::Undo).unwrap();
    ed.assert_render("[true, false]");
    assert_eq!(ed.clipboard.len(), 1); // contains `true`

    ed.exec(TreeNavCmd::Right).unwrap();
    ed.exec(TreeCmd::InsertHoleAfter).unwrap();
    ed.assert_render("[true, false, ?]");

    ed.exec(EditorCmd::PasteSwap).unwrap();
    ed.assert_render("[true, false, true]");
    assert_eq!(ed.clipboard.len(), 1); // contains hole

    ed.exec(EditorCmd::PasteSwap).unwrap();
    ed.assert_render("[true, false, ?]");
    assert_eq!(ed.clipboard.len(), 1); // contains `true`

    ed.exec(CommandGroup::Undo).unwrap();
    ed.assert_render("[true, false, true]");
    assert_eq!(ed.clipboard.len(), 1); // contains `true`

    ed.exec(CommandGroup::Redo).unwrap();
    ed.assert_render("[true, false, ?]");
    assert_eq!(ed.clipboard.len(), 1); // contains `true`

    ed.exec(TreeNavCmd::Left).unwrap();
    ed.exec(EditorCmd::PasteSwap).unwrap();
    ed.assert_render("[true, true, ?]");
    assert_eq!(ed.clipboard.len(), 1); // contains `false`

    ed.exec(TreeNavCmd::Right).unwrap();
    ed.exec(EditorCmd::PasteSwap).unwrap();
    ed.assert_render("[true, true, false]");
    assert_eq!(ed.clipboard.len(), 1); // contains hole
}

#[test]
fn test_bookmark() {
    let (lang, note_set) = make_json_lang();
    let (lang_set, lang_name) = make_singleton_lang_set(lang);
    let mut ed = TestEditor::new(&lang_set, &note_set, lang_name);

    ed.exec(group![
        TreeNavCmd::Child(0),
        TreeCmd::Replace(ed.node("list").unwrap()),
        TreeCmd::InsertHolePostpend,
        TreeCmd::Replace(ed.node("true").unwrap())
    ])
    .unwrap();

    let mark_true = ed.doc.bookmark();
    ed.exec(group![
        TreeCmd::InsertHoleAfter,
        TreeCmd::Replace(ed.node("false").unwrap())
    ])
    .unwrap();
    let mark_false = ed.doc.bookmark();

    ed.exec(group![
        TreeCmd::InsertHoleAfter,
        TreeCmd::Replace(ed.node("list").unwrap())
    ])
    .unwrap();
    let mark_list = ed.doc.bookmark();

    ed.exec(group![
        TreeCmd::InsertHolePrepend,
        TreeCmd::Replace(ed.node("null").unwrap())
    ])
    .unwrap();
    let mark_null = ed.doc.bookmark();

    ed.assert_render("[true, false, [null]]");

    ed.exec(TreeNavCmd::GotoBookmark(mark_true)).unwrap();
    ed.exec(TreeCmd::InsertHoleBefore).unwrap();
    ed.assert_render("[?, true, false, [null]]");

    ed.exec(TreeNavCmd::GotoBookmark(mark_null)).unwrap();
    ed.exec(TreeCmd::Clear).unwrap();
    ed.assert_render("[?, true, false, [?]]");

    ed.exec(TreeNavCmd::GotoBookmark(mark_false)).unwrap();
    ed.exec(TreeNavCmd::GotoBookmark(mark_false)).unwrap();
    ed.exec(TreeCmd::Replace(ed.node("false").unwrap()))
        .unwrap();
    ed.assert_render("[?, true, false, [?]]");

    assert_matches!(
        ed.exec(TreeNavCmd::GotoBookmark(mark_false)),
        Err(DocError::CannotMove)
    );
    assert_matches!(
        ed.exec(TreeNavCmd::GotoBookmark(mark_null)),
        Err(DocError::CannotMove)
    );
    ed.exec(TreeNavCmd::GotoBookmark(mark_list)).unwrap();
    ed.exec(TreeCmd::Replace(ed.node("list").unwrap())).unwrap();
    ed.assert_render("[?, true, false, []]");

    ed.exec(TreeNavCmd::GotoBookmark(mark_true)).unwrap();
    ed.exec(EditorCmd::Cut).unwrap();
    ed.assert_render("[?, ?, false, []]");

    ed.exec(TreeNavCmd::Left).unwrap();
    ed.exec(EditorCmd::PasteSwap).unwrap();
    ed.assert_render("[true, ?, false, []]");

    // Cut does not preserve bookmarks
    assert_matches!(
        ed.exec(TreeNavCmd::GotoBookmark(mark_true)),
        Err(DocError::CannotMove)
    );
    ed.exec(CommandGroup::Undo).unwrap();
    ed.assert_render("[?, ?, false, []]");

    ed.exec(CommandGroup::Undo).unwrap();
    ed.assert_render("[?, true, false, []]");

    // Undo preserves bookmarks
    ed.exec(TreeNavCmd::Right).unwrap();
    ed.exec(TreeNavCmd::Right).unwrap();
    ed.exec(TreeNavCmd::GotoBookmark(mark_true)).unwrap();
    ed.exec(TreeCmd::Replace(ed.node("null").unwrap())).unwrap();
    ed.assert_render("[?, null, false, []]");
}
