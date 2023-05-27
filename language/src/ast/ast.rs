use super::ast_forest::AstForest;
use super::forest::{Forest, Index};
use super::text::Text;
use crate::language::{Arity, Construct, ConstructId, Grammar};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(pub(super) usize);

#[derive(Clone)]
pub struct NodeData<'l> {
    pub(super) id: Id,
    pub(super) grammar: &'l Grammar,
    pub(super) construct_id: ConstructId,
    pub(super) text: Option<Text>,
}

/// An Abstract Syntax Tree.
///
/// More specifically, this is a mutable reference _to a node_ in an AST.
///
/// This value owns the entire tree. You must not drop it, or it will panic;
/// call delete() instead.
///
/// It also grants write access to the tree. Use [`borrow`](#method.borrow) to
/// obtain a shared reference with read-only access.
pub struct Ast(pub(super) Index);

// methods:
// - get arity, lang, construct, notation
// - text, text_mut
// - delete, detach, insert_first_child, insert_before, insert_after, replace
// - navigation (parent, root, child, prev, next)
// - nav preds (num_children, is_at_root, is_parent_at_root)
// - bookmarks (bookmark, goto_bookmark)

impl<'l> Ast {
    /// Get the arity of this node, or `None` if this is a leaf node.
    fn arity(&self, f: &AstForest<'l>) -> &'l Arity {
        let data = f.forest.data(self.0);
        &data.grammar.construct(data.construct_id).arity
    }
}

/*
pub enum AstCase<'a, 'l> {
    Texty(TextyAst<'a, 'l>),
    Fixed(FixedAst<'a, 'l>),
    Listy(ListyAst<'a, 'l>),
}

/// A wrapper around an `Ast` with `Texty` arity.
pub struct TextyAst<'a, 'l>(&'a mut Ast<'l>);

/// A wrapper around an `Ast` with `Fixed` arity.
pub struct FixedAst<'a, 'l>(&'a mut Ast<'l>);

/// A wrapper around an `Ast` with `Listy` arity.
pub struct ListyAst<'a, 'l>(&'a mut Ast<'l>);

impl<'l> Ast<'l> {
    pub(super) fn new(tree: Tree<NodeData<'l>, Text>) -> Ast<'l> {
        Ast { tree }
    }

    /// Get the arity of this node.
    pub fn arity(&self) -> &Arity {
        let node = self.tree.data();
        &node.grammar.construct(node.construct_id).arity
    }

    /// Get the syntactic construct this node falls into.
    pub fn construct(&self) -> &'l Construct {
        let node = self.tree.data();
        &node.grammar.construct(node.construct_id)
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

    /// Return the number of children this node has. For a Fixed node, this is
    /// its arity. For a Listy node, this is its current number of children.
    /// For text, this is considered 0.
    fn num_children(&self) -> usize {
        match self.arity() {
            Arity::Texty => 0,
            Arity::Fixed(sorts) => sorts.len(),
            Arity::Listy(_) => self.tree.num_children(),
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
    /// Panics if the index is out of bounds, or if this node has no parent.
    pub fn goto_sibling(&mut self, i: usize) {
        self.tree.goto_parent();
        self.tree.goto_child(i);
    }

    /// Go to this tree's root.
    pub fn goto_root(&mut self) {
        self.tree.goto_root()
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

    /// Produce a more specific type, based on this node's arity (fixed, texty, or listy). More
    /// methods are available on these specialized types.
    pub fn case<'a>(&'a mut self) -> AstCase<'a, 'l> {
        match self.arity() {
            Arity::Texty => AstCase::Texty(TextyAst(self)),
            Arity::Fixed(_) => AstCase::Fixed(FixedAst(self)),
            Arity::Listy(_) => AstCase::Listy(ListyAst(self)),
        }
    }
}

impl<'a, 'l> AstCase<'a, 'l> {
    pub fn unwrap_text(self) -> TextyAst<'a, 'l> {
        match self {
            AstCase::Texty(ast) => ast,
            _ => panic!("expected AstCase::Texty"),
        }
    }
    pub fn unwrap_fixed(self) -> FixedAst<'a, 'l> {
        match self {
            AstCase::Fixed(ast) => ast,
            _ => panic!("expected AstCase::Fixed"),
        }
    }
    pub fn unwrap_flexible(self) -> ListyAst<'a, 'l> {
        match self {
            AstCase::Listy(ast) => ast,
            _ => panic!("expected AstCase::Listy"),
        }
    }
}

impl<'a, 'l> TextyAst<'a, 'l> {
    /// Call the closure, giving it read-access to this node's text.
    pub fn with_text<R>(&self, func: impl FnOnce(&Text) -> R) -> R {
        self.0.tree.with_leaf(func)
    }

    /// Call the closure, giving it write-access to this node's text.
    pub fn with_text_mut<R>(&mut self, func: impl FnOnce(&mut Text) -> R) -> R {
        self.0.tree.with_leaf_mut(func)
    }
}

impl<'a, 'l> FixedAst<'a, 'l> {
    /// Return the number of children this node has. For a Fixed node, this is
    /// the same as its arity.
    pub fn num_children(&self) -> usize {
        self.0.num_children()
    }

    /// Go to this node's i'th child.
    ///
    /// # Panics
    ///
    /// Panics if `i` is out of bounds.
    pub fn goto_child(&mut self, i: usize) {
        self.0.tree.goto_child(i)
    }

    /// Replace this node's `i`th child with the `ast`. Return the old child if successful. If
    /// `ast` cannot be placed here because it has the wrong Sort, return it as `Err(ast)`.
    ///
    /// # Panics
    ///
    /// Panics if `i` is out of bounds.
    pub fn replace_child(&mut self, i: usize, ast: Ast<'l>) -> Result<Ast<'l>, Ast<'l>> {
        if !self.0.arity().child_sort(i).accepts(&ast.construct().sort) {
            // This ast can't go here, it has the wrong Sort! Send it back.
            return Err(ast);
        }
        Ok(Ast::new(self.0.tree.replace_child(i, ast.tree)))
    }
}

impl<'a, 'l> ListyAst<'a, 'l> {
    /// Return the number of children this node currently has.
    pub fn num_children(&self) -> usize {
        self.0.num_children()
    }

    /// Go to this node's i'th child.
    ///
    /// # Panics
    ///
    /// Panics if `i` is out of bounds.
    pub fn goto_child(&mut self, i: usize) {
        self.0.tree.goto_child(i)
    }

    /// Replace this node's `i`th child. If successful, return the replaced
    /// child. Otherwise, return the given ast as `Err(ast)`.
    ///
    /// # Panics
    ///
    /// Panics if `i` is out of bounds.
    pub fn replace_child(&mut self, i: usize, ast: Ast<'l>) -> Result<Ast<'l>, Ast<'l>> {
        if !self.0.arity().child_sort(i).accepts(&ast.construct().sort) {
            // This ast can't go here, it has the wrong Sort! Send it back.
            return Err(ast);
        }
        Ok(Ast::new(self.0.tree.replace_child(i, ast.tree)))
    }

    /// Insert `ast` as the `i`th child of this node. If it cannot be inserted because it has the
    /// wrong sort, return the given ast as `Err(ast)`.
    ///
    /// # Panics
    ///
    /// Panics if `i` is out of bounds.
    pub fn insert_child(&mut self, i: usize, ast: Ast<'l>) -> Result<(), Ast<'l>> {
        if !self.0.arity().child_sort(i).accepts(&ast.construct().sort) {
            // This tree can't go here, it has the wrong Sort! Send it back.
            return Err(ast);
        }
        self.0.tree.insert_child(i, ast.tree);
        Ok(())
    }

    /// Remove and return the `i`th child of this node.
    ///
    /// # Panics
    ///
    /// Panics if `i` is out of bounds.
    pub fn remove_child(&mut self, i: usize) -> Ast<'l> {
        Ast::new(self.0.tree.remove_child(i))
    }
}
*/
