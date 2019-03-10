use std::fmt;

use crate::ast::Ast;

pub enum CommandGroup<'l> {
    /// Execute all the commands as one group.
    Group(Vec<Command<'l>>),
    /// Undo the last group.
    Undo,
    /// Redo the last group.
    Redo,
}

#[derive(Debug)]
pub enum Command<'l> {
    Ed(EditorCmd),
    Tree(TreeCmd<'l>),
    TreeNav(TreeNavCmd),
    Text(TextCmd),
    TextNav(TextNavCmd),
}

#[derive(Debug)]
pub enum EditorCmd {
    /// Cut onto the clipboard.
    Cut,
    /// Copy onto the clipboard.
    Copy,
    /// Paste over the current node, replacing it.
    PasteReplace,
    /// In a flexible sequence, paste to the left of the current node.
    PasteBefore,
    /// In a flexible sequence, paste to the right of the current node.
    PasteAfter,
    /// In a flexible parent, paste at the beginning of its children.
    PastePrepend,
    /// In a flexible parent, paste at the end of its children.
    PastePostpend,
}

pub enum TreeCmd<'l> {
    /// Replace the current node.
    Replace(Ast<'l>),
    /// In a flexible sequence, insert to the left of the current node.
    InsertBefore(Ast<'l>),
    /// In a flexible sequence, insert to the right of the current node.
    InsertAfter(Ast<'l>),
    /// In a flexible parent, insert at the beginning of its children.
    InsertPrepend(Ast<'l>),
    /// In a flexible parent, insert at the end of its children.
    InsertPostpend(Ast<'l>),
    /// In a flexible sequence, remove the current node.
    Remove,
}

#[derive(Debug)]
pub enum TreeNavCmd {
    /// Move cursor to left sibling
    Left,
    /// Move cursor to right sibling
    Right,
    /// Move cursor to first child
    Child(usize),
    /// Move cursor to parent
    Parent,
}

#[derive(Debug)]
pub enum TextCmd {
    /// Insert the given character at the cursor position
    InsertChar(char),
    /// Delete the character immediately before the cursor
    DeleteCharBackward,
    /// Delete the character immediately after the cursor
    DeleteCharForward,
}

#[derive(Debug)]
pub enum TextNavCmd {
    /// Move cursor left one character
    Left,
    /// Move cursor right one character
    Right,
    /// Return to tree mode
    TreeMode,
}

impl<'l> From<TreeCmd<'l>> for Command<'l> {
    fn from(cmd: TreeCmd<'l>) -> Command<'l> {
        Command::Tree(cmd)
    }
}

impl<'l> From<TreeNavCmd> for Command<'l> {
    fn from(cmd: TreeNavCmd) -> Command<'l> {
        Command::TreeNav(cmd)
    }
}

impl<'l> From<TextCmd> for Command<'l> {
    fn from(cmd: TextCmd) -> Command<'l> {
        Command::Text(cmd)
    }
}

impl<'l> From<TextNavCmd> for Command<'l> {
    fn from(cmd: TextNavCmd) -> Command<'l> {
        Command::TextNav(cmd)
    }
}

impl<'l> fmt::Debug for TreeCmd<'l> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            TreeCmd::InsertAfter(_) => "InsertAfter",
            TreeCmd::InsertBefore(_) => "InsertBefore",
            TreeCmd::InsertPrepend(_) => "InsertPrepend",
            TreeCmd::InsertPostpend(_) => "InsertPostpend",
            TreeCmd::Replace(_) => "Replace",
            TreeCmd::Remove => "Remove",
        };
        write!(f, "TreeCmd::{}", name)
    }
}
