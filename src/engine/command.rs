use crate::language::{Construct, Storage};
use crate::tree::Node;

#[derive(Debug)]
pub enum Command {
    Ed(EdCommand),
    Clipboard(ClipboardCommand),
    Nav(NavCommand),
}

#[derive(Debug)]
pub enum EdCommand {
    Tree(TreeEdCommand),
    Text(TextEdCommand),
}

#[derive(Debug)]
pub enum NavCommand {
    Tree(TreeNavCommand),
    Text(TextNavCommand),
    Bookmark(BookmarkCommand),
}

#[derive(Debug)]
pub enum TreeEdCommand {
    /// In a listy sequence, insert the given node after the cursor. In a fixed sequence, replace
    /// the node at the cursor with the given node. Either way, move the cursor to the new node.
    Insert(Node),
    /// Replace the node at the cursor with the given node.
    Replace(Node),
    /// In a listy sequence, delete the node at the cursor and move the cursor to the left. In a
    /// fixed sequence, replace the node at the cursor with a hole.
    Backspace,
    /// In a listy sequence, delete the node at the cursor and move the cursor to the right. In a
    /// fixed sequence, replace the node at the cursor with a hole.
    Delete,
}

#[derive(Debug)]
pub enum TextEdCommand {
    /// Insert the given character at the cursor position, moving the cursor after the
    /// new character.
    Insert(char),
    /// Delete the character immediately before the cursor.
    Backspace,
    /// Delete the character immediately after the cursor.
    Delete,
}

// TODO: cut=copy,backspace  paste-copy=dup,paste
#[derive(Debug)]
pub enum ClipboardCommand {
    /// Copy the node at the cursor and push it onto the clipboard stack.
    Copy,
    /// Pop the top node from the clipboard stack, and insert it at the cursor (in the same manner
    /// as [`TreeEdCommand::Insert`]).
    Paste,
    /// Swap the top node in the clipboard stack with the node at the cursor.
    PasteSwap,
    /// Duplicate the top node in the clipboard stack.
    Dup,
    /// Discard the top node in the clipboard stack.
    Pop,
}

// TODO: First set of user nav commands to try: down-left & down-right
#[derive(Debug)]
pub enum TreeNavCommand {
    /// Move the cursor back one node.
    Prev,
    /// Move the cursor to the first sibling.
    First,
    /// Move the cursor forward one node.
    Next,
    /// Move the cursor to the last sibling, or after it in a listy sequence.
    Last,
    /// Move the cursor to its parent node.
    Parent,
    /// Move the cursor to the before the first child of the node at the cursor, if possible
    /// (otherwise to the first child).
    BeforeFirstChild,
    /// Move the cursor to the first child of the node at the cursor.
    FirstChild,
    /// Move the cursor to the last child of the node at the cursor.
    LastChild,
    /// Move the cursor to the previous leaf node (node with no children).
    PrevLeaf,
    /// Move the cursor to the next leaf node (node with no children).
    NextLeaf,
    /// Move the cursor to the (inorder) previous node of the given construct.
    PrevConstruct(Construct),
    /// Move the cursor to the (inorder) next node of the given construct.
    NextConstruct(Construct),
    /// Move the cursor to the previous texty node.
    PrevText,
    /// Move the cursor to the next texty node.
    NextText,
    /// If the node at the cursor is texty, enter text mode, placing the cursor at the
    /// end of the text.
    EnterText,
    /// Use this when the node at the cursor has just been `Insert`ed, to move the cursor to a
    /// convenient editing location.
    FirstInsertLoc,
}

#[derive(Debug)]
pub enum TextNavCommand {
    /// Move the cursor back one character.
    Left,
    /// Move the cursor forward one character.
    Right,
    /// Move the cursor to the beginning of the text.
    Beginning,
    /// Move the cursor to the end of the text.
    End,
    /// Exit text mode, keeping the edits.
    ExitText,
}

#[derive(Debug)]
pub enum BookmarkCommand {
    /// Save the cursor position as a bookmark.
    Save(char),
    /// Move the cursor to the bookmark saved under the given character. The bookmark follows, in
    /// priority order: (i) the left node, (ii) the right node, (iii) the parent node.
    Goto(char),
}

impl EdCommand {
    pub fn delete_trees(self, s: &mut Storage) {
        match self {
            EdCommand::Tree(cmd) => cmd.delete_trees(s),
            EdCommand::Text(_) => (),
        }
    }
}

impl TreeEdCommand {
    fn delete_trees(self, s: &mut Storage) {
        use TreeEdCommand::*;

        match self {
            Insert(node) | Replace(node) => node.delete_root(s),
            Backspace | Delete => (),
        }
    }
}

impl From<EdCommand> for Command {
    fn from(cmd: EdCommand) -> Command {
        Command::Ed(cmd)
    }
}

impl From<ClipboardCommand> for Command {
    fn from(cmd: ClipboardCommand) -> Command {
        Command::Clipboard(cmd)
    }
}

impl From<NavCommand> for Command {
    fn from(cmd: NavCommand) -> Command {
        Command::Nav(cmd)
    }
}

impl From<TreeEdCommand> for EdCommand {
    fn from(cmd: TreeEdCommand) -> EdCommand {
        EdCommand::Tree(cmd)
    }
}

impl From<TreeEdCommand> for Command {
    fn from(cmd: TreeEdCommand) -> Command {
        Command::Ed(EdCommand::Tree(cmd))
    }
}

impl From<TextEdCommand> for EdCommand {
    fn from(cmd: TextEdCommand) -> EdCommand {
        EdCommand::Text(cmd)
    }
}

impl From<TextEdCommand> for Command {
    fn from(cmd: TextEdCommand) -> Command {
        Command::Ed(EdCommand::Text(cmd))
    }
}

impl From<TreeNavCommand> for NavCommand {
    fn from(cmd: TreeNavCommand) -> NavCommand {
        NavCommand::Tree(cmd)
    }
}

impl From<TreeNavCommand> for Command {
    fn from(cmd: TreeNavCommand) -> Command {
        Command::Nav(NavCommand::Tree(cmd))
    }
}

impl From<TextNavCommand> for NavCommand {
    fn from(cmd: TextNavCommand) -> NavCommand {
        NavCommand::Text(cmd)
    }
}

impl From<TextNavCommand> for Command {
    fn from(cmd: TextNavCommand) -> Command {
        Command::Nav(NavCommand::Text(cmd))
    }
}

impl From<BookmarkCommand> for NavCommand {
    fn from(cmd: BookmarkCommand) -> NavCommand {
        NavCommand::Bookmark(cmd)
    }
}

impl From<BookmarkCommand> for Command {
    fn from(cmd: BookmarkCommand) -> Command {
        Command::Nav(NavCommand::Bookmark(cmd))
    }
}
