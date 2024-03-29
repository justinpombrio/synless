use super::doc_command::{
    DocCommand, EdCommand, NavCommand, TextEdCommand, TextNavCommand, TreeEdCommand, TreeNavCommand,
};
use crate::language::Storage;
use crate::tree::{Bookmark, Location, Mode, Node};
use crate::util::{bug_assert, SynlessBug};
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
    #[error("No node there to delete")]
    CannotDeleteNode,
    #[error("Cannot insert that node here")]
    CannotInsertNode,
}

pub struct Doc {
    cursor: Location,
    recent: Option<UndoGroup>,
    undo_stack: Vec<UndoGroup>,
    redo_stack: Vec<UndoGroup>,
}

impl Doc {
    pub fn new(node: Node, s: &Storage) -> Self {
        Doc {
            cursor: Location::before(node, s),
            recent: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn mode(&self) -> Mode {
        self.cursor.mode()
    }

    /// Get a Bookmark pointing to the current cursor.
    pub fn bookmark(&self) -> Bookmark {
        self.cursor.bookmark()
    }

    /// Move the cursor to the bookmark's location, if it's in this document.
    pub fn goto_bookmark(&mut self, bookmark: Bookmark, s: &Storage) -> Result<(), DocError> {
        if let Some(new_loc) = self.cursor.validate_bookmark(bookmark, s) {
            self.cursor = new_loc;
            Ok(())
        } else {
            Err(DocError::BookmarkNotFound)
        }
    }

    /// Executes a single command. Clears the redo stack if it was an editing command (but not if
    /// it was a navigation command).
    pub fn execute(&mut self, cmd: DocCommand, s: &mut Storage) -> Result<(), DocError> {
        match cmd {
            DocCommand::Ed(cmd) => {
                self.redo_stack.clear();
                let restore_loc = self.cursor;
                let undos = execute_ed(cmd, &mut self.cursor, s)?;
                if let Some(recent) = &mut self.recent {
                    recent.commands.extend(undos);
                } else {
                    self.recent = Some(UndoGroup::new(restore_loc, undos));
                }
                Ok(())
            }
            DocCommand::Nav(cmd) => execute_nav(cmd, &mut self.cursor, s),
        }
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

        if let Some(undo_group) = self.undo_stack.pop() {
            let redo_group = undo_group.execute(&mut self.cursor, s);
            self.redo_stack.push(redo_group);
            Ok(())
        } else {
            Err(DocError::NothingToUndo)
        }
    }

    /// Redoes the last undo group on the redo stack and moves it to the undo stack.
    /// Returns DocError::NothingToRedo if the redo stack is empty.
    pub fn redo(&mut self, s: &mut Storage) -> Result<(), DocError> {
        if let Some(redo_group) = self.redo_stack.pop() {
            bug_assert!(
                self.recent.is_none(),
                "redo: recent edits should have cleared the redo stack"
            );
            let undo_group = redo_group.execute(&mut self.cursor, s);
            self.undo_stack.push(undo_group);
            Ok(())
        } else {
            Err(DocError::NothingToRedo)
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

    fn execute(self, cursor: &mut Location, s: &mut Storage) -> UndoGroup {
        let mut redo_restore_loc = None;
        let mut redos = Vec::new();
        for (loc, cmd) in self.commands.into_iter().rev() {
            if redo_restore_loc.is_none() {
                redo_restore_loc = Some(loc);
            }
            jump_to(cursor, loc, s);
            redos.extend(execute_ed(cmd, cursor, s).bug_msg("Failed to undo/redo"));
        }

        jump_to(cursor, self.restore_loc, s);
        UndoGroup::new(redo_restore_loc.bug(), redos)
    }
}

fn jump_to(cursor: &mut Location, loc: Location, s: &Storage) {
    bug_assert!(
        cursor.validate_bookmark(loc.bookmark(), s).is_some(),
        "invalid loc"
    );
    *cursor = loc;
}

fn execute_ed(
    cmd: EdCommand,
    cursor: &mut Location,
    s: &mut Storage,
) -> Result<Vec<(Location, EdCommand)>, DocError> {
    match cmd {
        EdCommand::Tree(cmd) => execute_tree_ed(cmd, cursor, s),
        EdCommand::Text(cmd) => execute_text_ed(cmd, cursor, s),
    }
}

fn execute_nav(cmd: NavCommand, cursor: &mut Location, s: &Storage) -> Result<(), DocError> {
    match cmd {
        NavCommand::Tree(cmd) => execute_tree_nav(cmd, cursor, s),
        NavCommand::Text(cmd) => execute_text_nav(cmd, cursor, s),
    }
}

fn execute_tree_ed(
    cmd: TreeEdCommand,
    cursor: &mut Location,
    s: &mut Storage,
) -> Result<Vec<(Location, EdCommand)>, DocError> {
    use TreeEdCommand::*;

    if cursor.mode() != Mode::Tree {
        return Err(DocError::NotInTreeMode);
    }

    match cmd {
        Insert(node) => match cursor.insert(node, s) {
            Ok(None) => Ok(vec![(*cursor, Backspace.into())]),
            Ok(Some(detached_node)) => {
                Ok(vec![(cursor.prev(s).bug(), Insert(detached_node).into())])
            }
            Err(()) => Err(DocError::CannotInsertNode),
        },
        Backspace => {
            if let Some(old_node) = cursor.delete_neighbor(true, s) {
                Ok(vec![(*cursor, Insert(old_node).into())])
            } else {
                Err(DocError::CannotDeleteNode)
            }
        }
        Delete => {
            if let Some(old_node) = cursor.delete_neighbor(false, s) {
                Ok(vec![(*cursor, Insert(old_node).into())])
            } else {
                Err(DocError::CannotDeleteNode)
            }
        }
    }
}

fn execute_text_ed(
    cmd: TextEdCommand,
    cursor: &mut Location,
    s: &mut Storage,
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

fn execute_tree_nav(
    cmd: TreeNavCommand,
    cursor: &mut Location,
    s: &Storage,
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
            .and_then(|node| Location::after_children(node, s)),
        InorderNext => cursor.inorder_next(s),
        InorderPrev => cursor.inorder_prev(s),
        EnterText => cursor
            .left_node(s)
            .and_then(|node| Location::end_of_text(node, s)),
    };

    if let Some(new_loc) = new_loc {
        *cursor = new_loc;
        Ok(())
    } else {
        Err(DocError::CannotMove)
    }
}

fn execute_text_nav(
    cmd: TextNavCommand,
    cursor: &mut Location,
    s: &Storage,
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
