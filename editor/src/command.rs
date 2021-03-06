use crate::ast::Ast;
use forest::Bookmark;

#[derive(Debug)]
pub enum MetaCommand<'l> {
    /// Execute the given command.
    Do(Command<'l>),
    /// Undo the most recent undo-group.
    Undo,
    /// Redo the most recently undone undo-group.
    Redo,
    /// End the current undo-group. Commands executed after this will be placed in a new undo-group.
    EndGroup,
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
    /// Swap the current node with the node on the top of the clipboard.
    PasteSwap,
    /// Discard the node on the top of the clipboard.
    PopClipboard,
}

#[derive(Debug)]
pub enum TreeCmd<'l> {
    /// Replace the current node with the given node.
    Replace(Ast<'l>),
    /// In a flexible sequence, insert a hole to the left of the current node.
    InsertHoleBefore,
    /// In a flexible sequence, insert a hole to the right of the current node.
    InsertHoleAfter,
    /// On a flexible parent, insert a hole at the beginning of its children.
    InsertHolePrepend,
    /// On a flexible parent, insert a hole at the end of its children.
    InsertHolePostpend,
    /// In a flexible sequence, remove the current node.
    Remove,
    /// Replace the current node with a hole.
    Clear,
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
    /// Move to the node pointed to by the Bookmark
    GotoBookmark(Bookmark),
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

impl<'l> From<EditorCmd> for Command<'l> {
    fn from(cmd: EditorCmd) -> Command<'l> {
        Command::Ed(cmd)
    }
}

impl<'l, T> From<T> for MetaCommand<'l>
where
    T: Into<Command<'l>>,
{
    fn from(cmd_like: T) -> MetaCommand<'l> {
        MetaCommand::Do(cmd_like.into())
    }
}
