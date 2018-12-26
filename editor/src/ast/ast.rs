use std::ops::Deref;

use forest::{Tree, ReadLeaf, Bookmark};
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
#[derive(Clone)]
pub struct Ast<'l> {
    pub(super) tree: Tree<Node<'l>, String>
}

impl<'l> Ast<'l> {
    pub(super) fn new(tree: Tree<Node<'l>, String>) -> Ast<'l> {
        let mut ast = Ast {
            tree: tree
        };
        ast.update();
        ast
    }

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

    /// Returns `true` if this is the root of the tree, and `false` if
    /// it isn't (and thus this node has a parent).
    fn at_root(&self) -> bool {
        self.tree.at_root()
    }

    /// Go to the parent of this node.
    ///
    /// # Panics
    ///
    /// Panics if this is the root of the tree, and there is no parent.
    pub fn goto_parent(&mut self) {
        self.tree.goto_parent()
    }

    /// Save a bookmark to return to later.
    pub fn bookmark(&mut self) -> Bookmark {
        self.tree.bookmark()
    }

    /// Jump to a previously saved bookmark, as long as that
    /// bookmark's node is present somewhere in this tree. This will
    /// work even if the Tree has been modified since the bookmark was
    /// created. However, it will return `false` if the bookmark's node
    /// has since been deleted, or if it is currently located in a
    /// different tree.
    pub fn goto_bookmark(&mut self, mark: Bookmark) -> bool {
        self.tree.goto_bookmark(mark)
    }

    /// Update bounds. This must be called every time the tree is modified!
    ///
    /// # Panics
    /// 
    /// Panics if this is a leaf node.
    fn update(&mut self) {
        let bookmark = self.bookmark();
        self.tree.data_mut().bounds = Bounds::compute(&self.borrow());
        while !self.at_root() {
            self.goto_parent();
            self.tree.data_mut().bounds = Bounds::compute(&self.borrow());
        }
        self.goto_bookmark(bookmark);
    }
}

pub struct ReadText<'f, 'l>(pub(super) ReadLeaf<'f, Node<'l>, String>);

impl<'f, 'l> AsRef<str> for ReadText<'f, 'l> {
    fn as_ref(&self) -> &str {
        self.0.deref().as_ref()
    }
}
