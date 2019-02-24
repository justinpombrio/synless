use crate::doc::Command;

struct UndoGroup<'l> {
    group: Vec<Command<'l>>
}

struct Doc<'l> {
    ast: Ast<'l>,
    undos: Vec<UndoGroup<'l>>
}

impl<'l> Doc<'l> {
    
}

pub enum NavCommand {
    /// Move cursor to left sibling
    Left,
    /// Move cursor to right sibling
    Right,
    /// Move cursor to first child
    Child,
    /// Move cursor to parent
    Parent,
    /// Move cursor to leftmost sibling
    Leftmost,
    /// Move cursor to rightmost sibling
    Rightmost,
    /// Move cursor to root
    GotoRoot,
    /// Move cursor to leftmost leaf
    GotoLeaf
}
