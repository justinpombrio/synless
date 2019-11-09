use std::fmt;

use crate::notationset::NotationSet;
use crate::text::Text;
use forest::{Bookmark, Tree};
use language::{Arity, Construct, Language};
use pretty::{Bounds, Notation};

#[derive(Clone)]
pub struct Node<'l> {
    pub(super) bounds: Bounds,
    pub(super) language: &'l Language,
    pub(super) construct: &'l Construct,
    pub(super) notation: &'l Notation,
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
    pub(super) tree: Tree<Node<'l>, Text>,
}

impl<'l> Ast<'l> {
    pub(super) fn new(tree: Tree<Node<'l>, Text>) -> Ast<'l> {
        let mut ast = Ast { tree };
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

    /// Call the closure, giving it read-access to this node's text.
    ///
    /// # Panics
    ///
    /// Panics if the arity of this node is not `Text`.
    fn text<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&Text) -> T,
    {
        assert_eq!(self.arity(), Some(Arity::Text));
        self.tree.child_leaf(f)
    }

    /// Call the closure, giving it write-access to this node's text.
    ///
    /// # Panics
    ///
    /// Panics if the arity of this node is not `Text`.
    fn text_mut<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Text) -> T,
    {
        assert_eq!(self.arity(), Some(Arity::Text));
        let out = self.tree.child_leaf_mut(f);
        self.update();
        out
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

    /// Create a new `hole` node that belongs to the same forest as this node.
    pub fn new_hole(&mut self) -> Ast<'l> {
        let node = Node {
            bounds: Bounds::uninitialized(),
            language: self.get_language(),
            construct: Construct::hole(),
            notation: NotationSet::hole(),
        };
        Ast::new(self.tree.forest_mut().new_branch(node, vec![]))
    }

    /// Replace this node's `i`th child with the `tree`. Return the replaced
    /// child if successful. If `tree` cannot be placed here because it has the wrong
    /// Sort, return it as `Err(tree)`.
    ///
    /// # Panics
    ///
    /// Panics if this node's arity is not `Fixed` or `Flexible`, or if `i` is
    /// out of bounds.
    fn replace_child(&mut self, i: usize, tree: Ast<'l>) -> Result<Ast<'l>, Ast<'l>> {
        if let Some(arity) = self.arity() {
            if !arity.is_fixed() && !arity.is_flexible() {
                panic!("Ast::replace_child called on a node that is neither fixed nor flexible.")
            }
            if !arity.child_sort(i).accepts(&tree.get_construct().sort) {
                // This tree can't go here, it has the wrong Sort! Send it back.
                return Err(tree);
            }
            let ast = Ast {
                tree: self.tree.replace_child(i, tree.tree),
            };
            self.update();
            Ok(ast)
        } else {
            panic!("Ast::replace_child called on a leaf node");
        }
    }

    /// Insert `tree` as the `i`th child of this node. This node must be
    /// `Flexible` or `Mixed`. For nodes of `Mixed` arity, `i` counts both tree
    /// and text children. If `tree` cannot be inserted because it has the wrong
    /// Sort, return it as `Err(tree)`.
    ///
    /// # Panics
    ///
    /// Panics if this node's arity is not `Flexible` or `Mixed`, or if `i` is
    /// out of bounds.
    fn insert_child(&mut self, i: usize, tree: Ast<'l>) -> Result<(), Ast<'l>> {
        if let Some(arity) = self.arity() {
            if !arity.is_flexible() && !arity.is_mixed() {
                panic!("Ast::insert_child called on a node that isn't Flexible or Mixed")
            }

            if !arity.child_sort(i).accepts(&tree.get_construct().sort) {
                // This tree can't go here, it has the wrong Sort! Send it back.
                return Err(tree);
            }

            self.tree.insert_child(i, tree.tree);
            self.update();
            Ok(())
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
    fn remove_child(&mut self, i: usize) -> Ast<'l> {
        if let Some(arity) = self.arity() {
            if !arity.is_flexible() && !arity.is_mixed() {
                panic!("Ast::remove_child called on a node that isn't Flexible or Mixed")
            }
            let ast = Ast {
                tree: self.tree.remove_child(i),
            };
            self.update();
            ast
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
    pub fn is_at_root(&self) -> bool {
        self.tree.is_at_root()
    }

    /// Return `true` if this node is a child of the root of the tree, and `false` otherwise.
    pub fn is_parent_at_root(&self) -> bool {
        self.tree.is_parent_at_root()
    }

    /// Return the number of children this node has. For a Fixed node, this is
    /// its arity. For a Flexible node, this is its current number of children.
    /// For a Mixed node, this is its _total number_ of children, counting both
    /// tree and text children.
    ///
    /// # Panics
    ///
    /// Panics if the arity of this node is `Text`.
    fn num_children(&self) -> usize {
        match self.arity() {
            None => panic!("Ast::num_children called on a leaf node"),
            Some(Arity::Text) => panic!("Ast::num_children called on a Text node"),
            Some(_) => self.tree.num_children(),
        }
    }

    /// Go to the parent of this node. Returns this node's index among its
    /// siblings (so that you can return to it later).
    ///
    /// # Panics
    ///
    /// Panics if this is the root of the tree, and there is no parent.
    pub fn goto_parent(&mut self) -> usize {
        self.tree.goto_parent()
    }

    /// Go to the i'th child of this node's parent.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    pub fn goto_sibling(&mut self, i: usize) {
        self.tree.goto_parent();
        self.tree.goto_child(i);
    }

    /// Go to this tree's root.
    pub fn goto_root(&mut self) {
        self.tree.goto_root()
    }

    /// Go to this node's i'th child. For nodes of `Mixed` arity, `i` counts
    /// both tree and text children.
    ///
    /// # Panics
    ///
    /// Panics if the arity of this node is `Text`, or if `i` is out of bounds.
    fn goto_child(&mut self, i: usize) {
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

    // Panics if it's a leaf
    pub fn inner<'a>(&'a mut self) -> AstKind<'a, 'l> {
        match self.arity().expect("Ast::inner() - at a leaf node") {
            Arity::Text => AstKind::Text(TextAst { ast: self }),
            Arity::Fixed(_) => AstKind::Fixed(FixedAst { ast: self }),
            Arity::Flexible(_) => AstKind::Flexible(FlexibleAst { ast: self }),
            Arity::Mixed(_) => unimplemented!(),
        }
    }

    /// Update bounds. This must be called every time the tree is modified!
    ///
    /// # Panics
    ///
    /// Panics if this is a leaf node.
    fn update(&mut self) {
        let bookmark = self.bookmark();
        self.tree.data_mut().bounds = Bounds::compute(&self.ast_ref());
        while !self.is_at_root() {
            self.goto_parent();
            self.tree.data_mut().bounds = Bounds::compute(&self.ast_ref());
        }
        self.goto_bookmark(bookmark);
    }
}

impl<'l> fmt::Debug for Ast<'l> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // The contents of Ast are complicated, but it should implement Debug so
        // that we can derive Debug for structs containing Asts.
        write!(f, "Ast")
    }
}

pub enum AstKind<'a, 'l> {
    Text(TextAst<'a, 'l>),
    Fixed(FixedAst<'a, 'l>),
    Flexible(FlexibleAst<'a, 'l>),
}

impl<'a, 'l> AstKind<'a, 'l> {
    pub fn unwrap_text(self) -> TextAst<'a, 'l> {
        match self {
            AstKind::Text(ast) => ast,
            _ => panic!("expected AstKind::Text"),
        }
    }
    pub fn unwrap_fixed(self) -> FixedAst<'a, 'l> {
        match self {
            AstKind::Fixed(ast) => ast,
            _ => panic!("expected AstKind::Fixed"),
        }
    }
    pub fn unwrap_flexible(self) -> FlexibleAst<'a, 'l> {
        match self {
            AstKind::Flexible(ast) => ast,
            _ => panic!("expected AstKind::Flexible"),
        }
    }
}

/// A wrapper around an `Ast` with `Text` arity.
pub struct TextAst<'a, 'l> {
    ast: &'a mut Ast<'l>,
}

impl<'a, 'l> TextAst<'a, 'l> {
    /// Call the closure, giving it read-access to this node's text.
    pub fn text<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&Text) -> T,
    {
        self.ast.text(f)
    }

    /// Call the closure, giving it write-access to this node's text.
    pub fn text_mut<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Text) -> T,
    {
        self.ast.text_mut(f)
    }
}

/// A wrapper around an `Ast` with `Fixed` arity.
pub struct FixedAst<'a, 'l> {
    ast: &'a mut Ast<'l>,
}

impl<'a, 'l> FixedAst<'a, 'l> {
    /// Return the number of children this node has. For a Fixed node, this is
    /// the same as its arity.
    pub fn num_children(&self) -> usize {
        self.ast.num_children()
    }

    /// Go to this node's i'th child.
    ///
    /// # Panics
    ///
    /// Panics if `i` is out of bounds.
    pub fn goto_child(&mut self, i: usize) {
        self.ast.goto_child(i)
    }

    /// Replace this node's `i`th child. If successful, return the replaced
    /// child. Otherwise, return the given tree as `Err(tree)`.
    ///
    /// # Panics
    ///
    /// Panics if `i` is out of bounds.
    pub fn replace_child(&mut self, i: usize, tree: Ast<'l>) -> Result<Ast<'l>, Ast<'l>> {
        self.ast.replace_child(i, tree)
    }
}

/// A wrapper around an `Ast` with `Flexible` arity.
pub struct FlexibleAst<'a, 'l> {
    ast: &'a mut Ast<'l>,
}

impl<'a, 'l> FlexibleAst<'a, 'l> {
    /// Return the number of children this node currently has.
    pub fn num_children(&self) -> usize {
        self.ast.num_children()
    }

    /// Go to this node's i'th child.
    ///
    /// # Panics
    ///
    /// Panics if `i` is out of bounds.
    pub fn goto_child(&mut self, i: usize) {
        self.ast.goto_child(i)
    }

    /// Replace this node's `i`th child. If successful, return the replaced
    /// child. Otherwise, return the given tree as `Err(tree)`.
    ///
    /// # Panics
    ///
    /// Panics if `i` is out of bounds.
    pub fn replace_child(&mut self, i: usize, tree: Ast<'l>) -> Result<Ast<'l>, Ast<'l>> {
        self.ast.replace_child(i, tree)
    }

    /// Insert `tree` as the `i`th child of this node. If unsuccessful, return the given tree as `Err(tree)`.
    ///
    /// # Panics
    ///
    /// Panics if `i` is out of bounds.
    pub fn insert_child(&mut self, i: usize, tree: Ast<'l>) -> Result<(), Ast<'l>> {
        self.ast.insert_child(i, tree)
    }

    /// Remove and return the `i`th child of this node.
    ///
    /// # Panics
    ///
    /// Panics if `i` is out of bounds.
    pub fn remove_child(&mut self, i: usize) -> Ast<'l> {
        self.ast.remove_child(i)
    }
}
