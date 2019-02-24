use std::ops::{Deref, DerefMut};

use forest::{Tree, ReadLeaf, WriteLeaf, Bookmark};
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
//
// Many methods here panics if called on a text leaf. Make sure this can't happen.
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

    /// Get the arity of this node, or `None` if this is a leaf node.
    pub fn arity(&self) -> Option<Arity> {
        if self.tree.is_leaf() {
            return None;
        }
        Some(self.tree.data().construct.arity.clone())
    }

    /// Get a shared reference to the text at this node.
    ///
    /// # Panics
    ///
    /// Panics if the arity of this node is not `Text`. Also panics if two
    /// nodes/texts in the forest are borrowed at the same time.
    fn text<'f>(&'f self) -> ReadText<'f, 'l> {
        ReadText(self.tree.leaf())
    }

    /// Obtain a mutable reference to the text at this node.
    ///
    /// # Panics
    ///
    /// Panics if the arity of this node is not `Text`. Also panics if two
    /// nodes/texts in the forest are borrowed at the same time.
    pub fn text_mut<'f>(&'f mut self) -> WriteText<'f, 'l> {
        WriteText(self.tree.leaf_mut())
    }

    /// Get the language of this node's syntactic construct.
    pub fn get_language(&self) -> &'l Language {
        &self.tree.data().language
    }

    /// Get the syntactic construct this node falls into.
    pub fn get_construct(&self) -> &'l Construct {
        &self.tree.data().construct
    }

    /// Get the notation with which this node is currently displayed.
    pub fn get_notation(&self) -> &'l Notation {
        &self.tree.data().notation
    }

    /// Replace a Fixed node's `i`th child. Returns the replaced child.
    ///
    /// # Panics
    ///
    /// Panics if this node's arity is not `Fixed`.
    pub fn replace_child(&mut self, i: usize, tree: Ast<'l>) -> Ast<'l> {
        if let Some(arity) = self.arity() {
            if !arity.is_fixed() {
                panic!("Ast::replace_child called on a non-Fixed node")
            }
            Ast {
                tree: self.tree.replace_child(i, tree.tree)
            }
        } else {
            panic!("Ast::replace_child called on a leaf node");
        }
    }

    /// Insert `tree` as the `i`th child of this node. This node must be
    /// `Flexible` or `Mixed`. For nodes of `Mixed` arity, `i` counts both tree
    /// and text children.
    ///
    /// # Panics
    ///
    /// Panics if this node's arity is not `Flexible` or `Mixed`, or if `i` is
    /// out of bounds.
    pub fn insert_child(&mut self, i: usize, tree: Ast<'l>) {
        if let Some(arity) = self.arity() {
            if !arity.is_flexible() && !arity.is_mixed() {
                panic!("Ast::insert_child called on a node that isn't Flexible or Mixed")
            }
            self.tree.insert_child(i, tree.tree);
        } else {
            panic!("Ast::insert_child called on a leaf node");
        }
    }

    /// Remove and return the `i`th child of this node. This node must be
    /// `Flexible` or `Mixed`. For nodes of `Mixed` arity, `i` counts both tree
    /// and text children.
    ///
    /// # Panics
    ///
    /// Panics if this node's arity is not `Flexible` or `Mixed`, or if `i` is
    /// out of bounds.
    pub fn remove_child(&mut self, i: usize) -> Ast<'l> {
        if let Some(arity) = self.arity() {
            if !arity.is_flexible() && !arity.is_mixed() {
                panic!("Ast::remove_child called on a node that isn't Flexible or Mixed")
            }
            Ast {
                tree: self.tree.remove_child(i)
            }
        } else {
            panic!("Ast::remove_child called on leaf node");
        }
    }

    /// Determine this node's index among its siblings. Returns `0` when at the
    /// root. For Mixed parents, counts both text and tree children.
    pub fn index(&self) -> usize {
        self.tree.index()
    }

    /// Determine the number of siblings that this node has, including itself.
    /// For Mixed parents, counts both text and tree children. When at the root,
    /// returns 1.
    pub fn num_siblings(&self) -> usize {
        self.tree.num_siblings()
    }

    /// Returns `true` if this is the root of the tree, and `false` if
    /// it isn't (and thus this node has a parent).
    pub fn at_root(&self) -> bool {
        self.tree.at_root()
    }

    /// Return the number of children this node has. For a Fixed node, this is
    /// its arity. For a Flexible node, this is its current number of children.
    /// For a Mixed node, this is its _total number_ of children, counting both
    /// tree and text children.
    ///
    /// # Panics
    ///
    /// Panics if the arity of this node is `Text`.
    pub fn num_children(&self) -> usize {
        match self.arity() {
            None => panic!("Ast::num_children called on a leaf node"),
            Some(Arity::Text) => panic!("Ast::num_children called on a Text node"),
            Some(_) => self.tree.num_children()
        }
    }

    /// Go to the parent of this node.
    ///
    /// # Panics
    ///
    /// Panics if this is the root of the tree, and there is no parent.
    pub fn goto_parent(&mut self) {
        self.tree.goto_parent()
    }

    /// Go to this tree's root.
    pub fn goto_root(&mut self) {
        self.tree.goto_root()
    }

    /// Go to this tree's i'th child. For nodes of `Mixed` arity, `i` counts
    /// both tree and text children.
    ///
    /// # Panics
    ///
    /// Panics if the arity of this node is `Text`, or if `i` is out of bounds.
    pub fn goto_child(&mut self, i: usize) {
        self.tree.goto_child(i)
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

pub struct WriteText<'f, 'l>(pub(super) WriteLeaf<'f, Node<'l>, String>);

impl<'f, 'l> AsMut<str> for WriteText<'f, 'l> {
    fn as_mut(&mut self) -> &mut str {
        self.0.deref_mut().as_mut_str()
    }
}
