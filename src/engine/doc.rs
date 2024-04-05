use super::doc_command::{
    BookmarkCommand, ClipboardCommand, DocCommand, EdCommand, NavCommand, TextEdCommand,
    TextNavCommand, TreeEdCommand, TreeNavCommand,
};
use crate::language::Storage;
use crate::pretty_doc::DocRef;
use crate::tree::{Bookmark, Location, Mode, Node};
use crate::util::{bug_assert, SynlessBug};
use std::collections::HashMap;
use std::mem;

/// A set of changes that can be undone/redone all at once.
#[derive(Debug)]
pub struct UndoGroup {
    /// The position of the cursor before the first command was executed (where it should be
    /// restored to after undo-ing).
    restore_loc: Location,
    /// To undo using a (loc, cmd) pair, goto loc then execute cmd. Stored as a stack.
    commands: Vec<(Location, EdCommand)>,
}

#[derive(thiserror::Error, Debug)]
pub enum DocError {
    #[error("Cannot execute text command while not in text mode")]
    NotInTextMode,
    #[error("Cannot execute tree command while not in tree mode")]
    NotInTreeMode,
    #[error("Nothing to undo")]
    NothingToUndo,
    #[error("Nothing to redo")]
    NothingToRedo,
    #[error("Cannot move there")]
    CannotMove,
    #[error("Bookmark not found")]
    BookmarkNotFound,
    #[error("Cannot delete character here")]
    CannotDeleteChar,
    #[error("Cannot place that node here")]
    CannotPlaceNode,
    #[error("No node to act on here")]
    NoNodeHere,
    #[error("Clipboard is empty")]
    EmptyClipboard,
}

#[derive(Debug)]
pub struct Doc {
    cursor: Location,
    recent: Option<UndoGroup>,
    undo_stack: Vec<UndoGroup>,
    redo_stack: Vec<UndoGroup>,
    bookmarks: HashMap<char, Bookmark>,
}

impl Doc {
    /// Constructs a new Doc. Returns `None` if the node is not a root with the root construct.
    pub fn new(s: &Storage, root_node: Node) -> Option<Self> {
        if !root_node.construct(s).is_root(s) || !root_node.is_root(s) {
            return None;
        }
        Some(Doc {
            cursor: Location::before_children(s, root_node).bug(),
            recent: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            bookmarks: HashMap::new(),
        })
    }

    pub fn doc_ref_source<'d>(&self, s: &'d Storage) -> DocRef<'d> {
        DocRef::new_source(s, self.cursor, self.cursor.root_node(s))
    }

    pub fn doc_ref_display<'d>(&self, s: &'d Storage) -> DocRef<'d> {
        DocRef::new_display(s, self.cursor, self.cursor.root_node(s))
    }

    pub fn cursor(&self) -> Location {
        self.cursor
    }

    pub fn mode(&self) -> Mode {
        self.cursor.mode()
    }

    /// Executes a single command. Clears the redo stack if it was an editing command (but not if
    /// it was a navigation command).
    pub fn execute(
        &mut self,
        s: &mut Storage,
        cmd: DocCommand,
        clipboard: &mut Vec<Node>,
    ) -> Result<(), DocError> {
        let restore_loc = self.cursor;
        let undos = match cmd {
            DocCommand::Ed(cmd) => execute_ed(s, cmd, &mut self.cursor)?,
            DocCommand::Clipboard(cmd) => execute_clipboard(s, cmd, &mut self.cursor, clipboard)?,
            DocCommand::Nav(cmd) => {
                execute_nav(s, cmd, &mut self.cursor, &mut self.bookmarks)?;
                Vec::new()
            }
        };
        if undos.is_empty() {
            return Ok(());
        }
        self.clear_redos(s);
        if let Some(recent) = &mut self.recent {
            recent.commands.extend(undos);
        } else {
            self.recent = Some(UndoGroup::new(restore_loc, undos));
        }
        Ok(())
    }

    /// Groups together all editing commands that have been `.execute()`ed since the last call to
    /// `.end_undo_group()`. They will be treated as a single unit ("undo group") by calls to
    /// `.undo()` and `.redo()`.
    pub fn end_undo_group(&mut self) {
        if let Some(recent) = self.recent.take() {
            self.undo_stack.push(recent);
        }
    }

    /// Undoes the last undo group on the undo stack and moves it to the redo stack.
    /// Returns `Err(DocError::NothingToUndo)` if the undo stack is empty.
    /// If there were recent edits _not_ completed with a call to end_undo_group(),
    /// the group is automatically ended and then undone.
    pub fn undo(&mut self, s: &mut Storage) -> Result<(), DocError> {
        self.end_undo_group();

        let undo_group = self.undo_stack.pop().ok_or(DocError::NothingToUndo)?;
        let redo_group = undo_group.execute(s, &mut self.cursor);
        self.redo_stack.push(redo_group);
        Ok(())
    }

    /// Redoes the last undo group on the redo stack and moves it to the undo stack.
    /// Returns DocError::NothingToRedo if the redo stack is empty.
    pub fn redo(&mut self, s: &mut Storage) -> Result<(), DocError> {
        let redo_group = self.redo_stack.pop().ok_or(DocError::NothingToRedo)?;
        bug_assert!(
            self.recent.is_none(),
            "redo: recent edits should have cleared the redo stack"
        );
        let undo_group = redo_group.execute(s, &mut self.cursor);
        self.undo_stack.push(undo_group);
        Ok(())
    }

    fn clear_redos(&mut self, s: &mut Storage) {
        for group in self.redo_stack.drain(..) {
            group.delete_trees(s);
        }
    }
}

impl UndoGroup {
    fn new(restore_loc: Location, commands: Vec<(Location, EdCommand)>) -> UndoGroup {
        bug_assert!(!commands.is_empty(), "empty undo group");
        UndoGroup {
            restore_loc,
            commands,
        }
    }

    fn execute(self, s: &mut Storage, cursor: &mut Location) -> UndoGroup {
        let mut redo_restore_loc = None;
        let mut redos = Vec::new();
        for (loc, cmd) in self.commands.into_iter().rev() {
            if redo_restore_loc.is_none() {
                redo_restore_loc = Some(loc);
            }
            jump_to(s, cursor, loc);
            redos.extend(execute_ed(s, cmd, cursor).bug_msg("Failed to undo/redo"));
        }

        jump_to(s, cursor, self.restore_loc);
        UndoGroup::new(redo_restore_loc.bug(), redos)
    }

    fn delete_trees(self, s: &mut Storage) {
        for (_loc, cmd) in self.commands {
            cmd.delete_trees(s);
        }
    }
}

fn jump_to(s: &Storage, cursor: &mut Location, loc: Location) {
    bug_assert!(
        cursor.validate_bookmark(s, loc.bookmark()).is_some(),
        "invalid loc"
    );
    *cursor = loc;
}

fn execute_ed(
    s: &mut Storage,
    cmd: EdCommand,
    cursor: &mut Location,
) -> Result<Vec<(Location, EdCommand)>, DocError> {
    match cmd {
        EdCommand::Tree(cmd) => execute_tree_ed(s, cmd, cursor),
        EdCommand::Text(cmd) => execute_text_ed(s, cmd, cursor),
    }
}

fn execute_nav(
    s: &Storage,
    cmd: NavCommand,
    cursor: &mut Location,
    bookmarks: &mut HashMap<char, Bookmark>,
) -> Result<(), DocError> {
    match cmd {
        NavCommand::Tree(cmd) => execute_tree_nav(s, cmd, cursor),
        NavCommand::Text(cmd) => execute_text_nav(s, cmd, cursor),
        NavCommand::Bookmark(cmd) => execute_bookmark(s, cmd, cursor, bookmarks),
    }
}

fn execute_tree_ed(
    s: &mut Storage,
    cmd: TreeEdCommand,
    cursor: &mut Location,
) -> Result<Vec<(Location, EdCommand)>, DocError> {
    use TreeEdCommand::*;

    if cursor.mode() != Mode::Tree {
        return Err(DocError::NotInTreeMode);
    }

    match cmd {
        Insert(node) => match cursor.insert(s, node) {
            Ok(None) => Ok(vec![(*cursor, Backspace.into())]),
            Ok(Some(detached_node)) => {
                Ok(vec![(cursor.prev(s).bug(), Insert(detached_node).into())])
            }
            Err(()) => Err(DocError::CannotPlaceNode),
        },
        Replace(new_node) => {
            let old_node = cursor.left_node(s).ok_or(DocError::NoNodeHere)?;
            if old_node.swap(s, new_node) {
                Ok(vec![(*cursor, Replace(old_node).into())])
            } else {
                Err(DocError::CannotPlaceNode)
            }
        }
        Backspace => {
            let old_node = cursor
                .delete_neighbor(s, true)
                .ok_or(DocError::NoNodeHere)?;
            Ok(vec![(*cursor, Insert(old_node).into())])
        }
        Delete => {
            let old_node = cursor
                .delete_neighbor(s, false)
                .ok_or(DocError::NoNodeHere)?;
            Ok(vec![(*cursor, Insert(old_node).into())])
        }
    }
}

fn execute_text_ed(
    s: &mut Storage,
    cmd: TextEdCommand,
    cursor: &mut Location,
) -> Result<Vec<(Location, EdCommand)>, DocError> {
    use TextEdCommand::{Backspace, Delete, Insert};

    let (node, char_index) = cursor.text_pos_mut().ok_or(DocError::NotInTextMode)?;
    let text = node.text_mut(s).bug();

    match cmd {
        Insert(ch) => {
            text.insert(*char_index, ch);
            *char_index += 1;
            Ok(vec![(*cursor, Backspace.into())])
        }
        Backspace => {
            if *char_index == 0 {
                return Err(DocError::CannotDeleteChar);
            }
            let ch = text.delete(*char_index - 1);
            *char_index -= 1;
            Ok(vec![(*cursor, Insert(ch).into())])
        }
        Delete => {
            let text_len = text.num_chars();
            if *char_index == text_len {
                return Err(DocError::CannotDeleteChar);
            }
            let ch = text.delete(*char_index);
            Ok(vec![(*cursor, Insert(ch).into())])
        }
    }
}

fn execute_clipboard(
    s: &mut Storage,
    cmd: ClipboardCommand,
    cursor: &mut Location,
    clipboard: &mut Vec<Node>,
) -> Result<Vec<(Location, EdCommand)>, DocError> {
    use ClipboardCommand::*;

    match cmd {
        Copy => {
            let node = cursor.left_node(s).ok_or(DocError::NoNodeHere)?;
            clipboard.push(node.deep_copy(s));
            Ok(Vec::new())
        }
        Paste => {
            let node = clipboard.pop().ok_or(DocError::EmptyClipboard)?;
            let result = execute_tree_ed(s, TreeEdCommand::Insert(node), cursor);
            if result.is_err() {
                clipboard.push(node);
            }
            result
        }
        PasteSwap => {
            let clip_node = clipboard.pop().ok_or(DocError::EmptyClipboard)?;
            let doc_node = cursor.left_node(s).ok_or(DocError::NoNodeHere)?;
            if doc_node.swap(s, clip_node) {
                clipboard.push(doc_node.deep_copy(s));
                Ok(vec![(*cursor, TreeEdCommand::Replace(doc_node).into())])
            } else {
                clipboard.push(clip_node);
                Err(DocError::CannotPlaceNode)
            }
        }
        Dup => {
            let clip_node = clipboard.last().ok_or(DocError::EmptyClipboard)?;
            clipboard.push(clip_node.deep_copy(s));
            Ok(Vec::new())
        }
        Pop => {
            let clip_node = clipboard.pop().ok_or(DocError::EmptyClipboard)?;
            clip_node.delete_root(s);
            Ok(Vec::new())
        }
    }
}

fn execute_tree_nav(
    s: &Storage,
    cmd: TreeNavCommand,
    cursor: &mut Location,
) -> Result<(), DocError> {
    use TreeNavCommand::*;

    if cursor.mode() != Mode::Tree {
        return Err(DocError::NotInTreeMode);
    }

    let new_loc = match cmd {
        Prev => cursor.prev(s),
        Next => cursor.next(s),
        First => cursor.first(s),
        Last => cursor.last(s),
        Parent => cursor.after_parent(s),
        LastChild => cursor
            .left_node(s)
            .and_then(|node| Location::after_children(s, node)),
        InorderNext => cursor.inorder_next(s),
        InorderPrev => cursor.inorder_prev(s),
        EnterText => cursor
            .left_node(s)
            .and_then(|node| Location::end_of_text(s, node)),
    };

    if let Some(new_loc) = new_loc {
        *cursor = new_loc;
        Ok(())
    } else {
        Err(DocError::CannotMove)
    }
}

fn execute_text_nav(
    s: &Storage,
    cmd: TextNavCommand,
    cursor: &mut Location,
) -> Result<(), DocError> {
    use TextNavCommand::*;

    let (node, char_index) = cursor.text_pos_mut().ok_or(DocError::NotInTextMode)?;
    let text = node.text(s).bug();

    match cmd {
        Left => {
            if *char_index == 0 {
                return Err(DocError::CannotMove);
            }
            *char_index -= 1;
        }
        Right => {
            if *char_index >= text.num_chars() {
                return Err(DocError::CannotMove);
            }
            *char_index += 1;
        }
        Beginning => *char_index = 0,
        End => *char_index = text.num_chars(),
        ExitText => *cursor = cursor.exit_text().bug(),
    }
    Ok(())
}

fn execute_bookmark(
    s: &Storage,
    cmd: BookmarkCommand,
    cursor: &mut Location,
    bookmarks: &mut HashMap<char, Bookmark>,
) -> Result<(), DocError> {
    match cmd {
        BookmarkCommand::Save(letter) => {
            bookmarks.insert(letter, cursor.bookmark());
            Ok(())
        }
        BookmarkCommand::Goto(letter) => {
            if let Some(loc) = bookmarks
                .get(&letter)
                .and_then(|bookmark| cursor.validate_bookmark(s, *bookmark))
            {
                *cursor = loc;
                Ok(())
            } else {
                Err(DocError::BookmarkNotFound)
            }
        }
    }
}
