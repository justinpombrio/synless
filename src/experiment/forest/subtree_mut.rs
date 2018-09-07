use std::mem;

use super::forest::{Id, Forest};
use super::tree::{Tree, Bookmark};
use super::subtree_ref::SubtreeRef;


/// A mutable reference to a Tree.
///
/// This reference will begin pointing at the root of the Tree, but
/// can move to subtrees after being created.
///
/// Essentially all operations require a reference to the Forest that
/// created the Tree as their first argument.
pub struct SubtreeMut<'a> {
    root: &'a mut Tree, // INVARIANT: This root remains valid despite edits
    id: Id
}

impl Tree {
    /// Obtain a mutable reference to this Tree.
    pub fn as_mut<D, L>(&mut self, _f: &mut Forest<D, L>) -> SubtreeMut {
        SubtreeMut {
            id: self.id,
            root: self
        }
    }
}

impl<'a> SubtreeMut<'a> {

    // Conversion //

    /// Get an _immutable_ reference at this location in the tree.
    pub fn as_ref<D, L>(&self, _f: &Forest<D, L>) -> SubtreeRef {
        SubtreeRef {
            root: self.root,
            id: self.id
        }
    }
    
    /// Returns `true` if this is a leaf node, and `false` if this is
    /// a branch node.
    pub fn is_leaf<D, L>(&self, f: &Forest<D, L>) -> bool {
        f.is_leaf(self.id)
    }

    /// Obtain a reference to the data value at this node.
    ///
    /// # Panics
    ///
    /// Panics if this is not a branch node. (Leaves do not have data.)
    pub fn data<D, L>(&self, f: &'a Forest<D, L>) -> &'a D {
        f.data(self.id)
    }

    /// Obtain a reference to the leaf value at this node.
    ///
    /// # Panics
    ///
    /// Panics if this is a branch node.
    pub fn leaf<D, L>(&self, f: &'a Forest<D, L>) -> &'a L {
        f.leaf(self.id)
    }

    /// Returns the number of children this node has.
    ///
    /// # Panics
    ///
    /// Panics if this is a leaf node.
    pub fn num_children<D, L>(&self, f: &Forest<D, L>) -> usize {
        f.children(self.id).len()
    }

    /// Obtain a mutable reference to the data value at this node.
    ///
    /// # Panics
    ///
    /// Panics if this is not a branch node. (Leaves do not have data.)
    pub fn data_mut<'b, D, L>(&'b mut self, f: &'b mut Forest<D, L>) -> &'b mut D {
        f.data_mut(self.id)
    }

    /// Obtain a mutable reference to the leaf value at this node.
    ///
    /// # Panics
    ///
    /// Panics if this is a branch node.
    pub fn leaf_mut<'b, D, L>(&'b mut self, f: &'b mut Forest<D, L>) -> &'b mut L {
        f.leaf_mut(self.id)
    }

    /// Replace the `i`th child of this node with `tree`.
    /// Returns the original child.
    /// 
    /// # Panics
    ///
    /// Panics if this is a leaf node, or if `i` is out of bounds.
    pub fn replace_child<D, L>(&mut self, f: &mut Forest<D, L>, i: usize, tree: Tree) -> Tree {
        let new_tree = Tree {
            id: f.replace_child(self.id, i, tree.id)
        };
        mem::forget(tree);
        new_tree
    }

    /// Insert `tree` as the `i`th child of this node.
    /// 
    /// # Panics
    ///
    /// Panics if this is a leaf node, or if `i` is out of bounds.
    pub fn insert_child<D, L>(&mut self, f: &mut Forest<D, L>, i: usize, tree: Tree) {
        f.insert_child(self.id, i, tree.id);
        mem::forget(tree);
    }

    /// Remove and return the `i`th child of this node.
    /// 
    /// # Panics
    ///
    /// Panics if this is a leaf node, or if `i` is out of bounds.
    pub fn remove_child<D, L>(&mut self, f: &mut Forest<D, L>, i: usize) -> Tree {
        Tree {
            id: f.remove_child(self.id, i)
        }
    }

    /// Save a bookmark to return to later.
    pub fn bookmark<D, L>(&mut self, _f: &mut Forest<D, L>) -> Bookmark {
        Bookmark {
            id: self.id
        }
    }

    /// Jump to a previously saved bookmark, as long as that
    /// bookmark's node is present somewhere in this tree. This will
    /// work even if the Tree has been modified since the bookmark was
    /// created. However, it will return `None` if the bookmark's node
    /// has since been deleted, or if it is currently located in a
    /// different tree.
    pub fn goto_bookmark<D, L>(&mut self, f: &mut Forest<D, L>, mark: Bookmark) -> bool {
        if f.is_valid(mark.id) && f.root(mark.id) == self.root.id {
            self.id = mark.id;
            true
        } else {
            false
        }
    }

    // TODO: panic if there is no parent, instead
    pub fn goto_parent<D, L>(&mut self, f: &mut Forest<D, L>) -> bool {
        match f.parent(self.id) {
            None => false,
            Some(parent) => {
                self.id = parent;
                true
            }
        }
    }

    /// Go to the `i`th child of this branch node.
    ///
    /// # Panics
    ///
    /// Panics if this is a leaf node, or if `i` is out of bounds.
    pub fn goto_child<D, L>(&mut self, f: &mut Forest<D, L>, i: usize) {
        self.id = f.child(self.id, i);
    }
}