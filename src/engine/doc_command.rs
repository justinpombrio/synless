use crate::tree::{Bookmark, Node};

#[derive(Debug)]
pub enum DocCommand {
    Ed(EdCommand),
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
}

#[derive(Debug)]
pub enum TreeEdCommand {
    /// In a listy sequence, insert the given node at the cursor position. In a fixed sequence,
    /// replace the node after the cursor with the given node. Either way, move the cursor after
    /// the new node.
    Insert(Node),
    /// In a listy sequence, delete the node before the cursor. In a fixed sequence,
    /// replace the node before the cursor with a hole, and move the cursor before it.
    Backspace,
    /// In a listy sequence, delete the node after the cursor. In a fixed sequence,
    /// replace the node after the cursor with a hole, and move the cursor after it.
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

// TODO: First set of user nav commands to try: down-left & down-right
#[derive(Debug)]
pub enum TreeNavCommand {
    /// Move the cursor back one node.
    Prev,
    /// Move the cursor to before the first sibling.
    First,
    /// Move the cursor forward one node.
    Next,
    /// Move the cursor to after the last sibling.
    Last,
    /// Move the cursor to the next location in-order.
    InorderNext,
    /// Move the cursor to the previous location in-order.
    InorderPrev,
    /// Move the cursor after its parent.
    Parent,
    /// Move the cursor to after the last child of the node before the cursor.
    LastChild,
    /// If the node before the cursor is texty, enter text mode, placing the cursor at the
    /// end of the text.
    EnterText,
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
    /// Exit text mode, placing the cursor after the texty node.
    ExitText,
}

impl From<EdCommand> for DocCommand {
    fn from(cmd: EdCommand) -> DocCommand {
        DocCommand::Ed(cmd)
    }
}

impl From<NavCommand> for DocCommand {
    fn from(cmd: NavCommand) -> DocCommand {
        DocCommand::Nav(cmd)
    }
}

impl From<TreeEdCommand> for EdCommand {
    fn from(cmd: TreeEdCommand) -> EdCommand {
        EdCommand::Tree(cmd)
    }
}

impl From<TextEdCommand> for EdCommand {
    fn from(cmd: TextEdCommand) -> EdCommand {
        EdCommand::Text(cmd)
    }
}

impl From<TreeNavCommand> for NavCommand {
    fn from(cmd: TreeNavCommand) -> NavCommand {
        NavCommand::Tree(cmd)
    }
}

impl From<TextNavCommand> for NavCommand {
    fn from(cmd: TextNavCommand) -> NavCommand {
        NavCommand::Text(cmd)
    }
}

impl From<TreeEdCommand> for DocCommand {
    fn from(cmd: TreeEdCommand) -> DocCommand {
        DocCommand::Ed(EdCommand::Tree(cmd))
    }
}

impl From<TextEdCommand> for DocCommand {
    fn from(cmd: TextEdCommand) -> DocCommand {
        DocCommand::Ed(EdCommand::Text(cmd))
    }
}

impl From<TreeNavCommand> for DocCommand {
    fn from(cmd: TreeNavCommand) -> DocCommand {
        DocCommand::Nav(NavCommand::Tree(cmd))
    }
}

impl From<TextNavCommand> for DocCommand {
    fn from(cmd: TextNavCommand) -> DocCommand {
        DocCommand::Nav(NavCommand::Text(cmd))
    }
}
