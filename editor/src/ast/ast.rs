use std::ops::Deref;

use forest::{Tree, ReadLeaf};
use pretty::{Bounds, Notation};
use language::{Language, Construct, Arity};


#[derive(Clone)]
pub struct Node<'l> {
    pub(super) bounds: Bounds,
    pub(super) language: &'l Language,
    pub(super) construct: &'l Construct,
    pub(super) notation: &'l Notation
}

/// An Abstract Syntax Tree.
///
/// More specifically, this is a mutable reference _to a node_ in an AST.
///
/// This value owns the entire tree. When it is dropped, the tree is deleted.
///
/// It also grants write access to the tree. Use [`borrow`](#method.borrow) to
/// obtain a shared reference with read-only access.
///
/// All write operations mutably borrow the _entire forest_. While an AST is
/// being mutated, or when some of its data is mutably borrowed, _no other tree
/// in the forest can be accessed_.
///
/// The AST automatically keeps track the `Bounds` information that's needed for
/// pretty-printing.
// TODO: can its data ever be mutably bororwed? If so, give an example. If not,
// delete that phrase.
pub struct Ast<'l> {
    pub(super) tree: Tree<Node<'l>, String>
}

impl<'l> Ast<'l> {
    /// Get the arity of this node.
    fn arity(&self) -> Arity {
        self.tree.data().construct.arity.clone()
    }

    /// Get a shared reference to the text at this node.
    ///
    /// # Panics
    ///
    /// Panics if the arity of this node is not `Text`.
    fn text<'f>(&'f self) -> ReadText<'f, 'l> {
        ReadText(self.tree.leaf())
    }
}

pub struct ReadText<'f, 'l>(pub(super) ReadLeaf<'f, Node<'l>, String>);

impl<'f, 'l> AsRef<str> for ReadText<'f, 'l> {
    fn as_ref(&self) -> &str {
        self.0.deref().as_ref()
    }
}
