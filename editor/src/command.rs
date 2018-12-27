use crate::ast::Ast;

pub enum Command<'l> {
    Nav(NavCmd),
    Doc(DocCmd<'l>),
    Ed(EditorCmd)
}

pub enum NavCmd {
    /// Move cursor to left sibling
    Left,
    /// Move cursor to right sibling
    Right,
    /// Move cursor to first child
    Child(usize),
    /// Move cursor to parent
    Parent
}

pub enum DocCmd<'l> {
    /// Undo the last operation
    Undo,
    /// Redo the last undone operation
    Redo,
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
    Remove
}

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
    PastePostpend
}
