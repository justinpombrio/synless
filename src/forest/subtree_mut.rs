use std::mem;
use std::cell::{Ref, RefMut};

use super::{Id, RawForest};
use super::tree::{Tree, Bookmark, Forest, ReadData, ReadLeaf, WriteData, WriteLeaf};
use super::subtree_ref::SubtreeRef;


/// A mutable reference to a [`Tree`](struct.Tree.html).
///
/// This reference will begin pointing at the root of the Tree, but
/// can move to subtrees after being created.
pub struct SubtreeMut<'f, D: 'f, L: 'f> {
    forest: &'f Forest<D, L>,
    root: Id, // INVARIANT: This root remains valid despite edits
    id: Id
}

impl<'f, D, L> Tree<'f, D, L> {
    /// Obtain a mutable reference to this Tree.
    ///
    /// An Operation on the borrowed tree will **panic** if it happens
    /// concurrently with any other operation on a tree in the same forest.
    pub fn as_mut(&mut self) -> SubtreeMut<'f, D, L> {
        SubtreeMut {
            forest: self.forest,
            id: self.id,
            root: self.id
        }
    }
}


impl<'f, D, L> SubtreeMut<'f, D, L> {

    // Conversion //
    /// Get an _immutable_ reference at this location in the tree.
    pub fn as_ref(&self) -> SubtreeRef<'f, D, L> {
        SubtreeRef {
            forest: self.forest,
            root: self.root,
            id: self.id
        }
    }

    /// Returns `true` if this is a leaf node, and `false` if this is
    /// a branch node.
    pub fn is_leaf(&self) -> bool {
        self.forest().is_leaf(self.id)
    }

    /// Obtain a shared reference to the data value at this node.
    ///
    /// # Panics
    ///
    /// Panics if this is not a branch node. (Leaves do not have data.)
    pub fn data(&self) -> ReadData<'f, D, L> {
        ReadData {
            guard: self.forest(),
            id: self.id
        }
    }

    /// Obtain a shared reference to the leaf value at this node.
    ///
    /// # Panics
    ///
    /// Panics if this is a branch node.
    pub fn leaf(&self) -> ReadLeaf<'f, D, L> {
        ReadLeaf {
            guard: self.forest(),
            id: self.id
        }
    }

    /// Returns the number of children this node has.
    ///
    /// # Panics
    ///
    /// Panics if this is a leaf node.
    pub fn num_children(&self) -> usize {
        self.forest().children(self.id).len()
    }

    /// Obtain a mutable reference to the data value at this node.
    ///
    /// # Panics
    ///
    /// Panics if this is not a branch node. (Leaves do not have data.)
    pub fn data_mut(&mut self) -> WriteData<'f, D, L> {
        WriteData {
            guard: self.forest_mut(),
            id: self.id
        }
    }

    /// Obtain a mutable reference to the leaf value at this node.
    ///
    /// # Panics
    ///
    /// Panics if this is a branch node.
    pub fn leaf_mut(&mut self) -> WriteLeaf<'f, D, L> {
        WriteLeaf {
            guard: self.forest_mut(),
            id: self.id
        }
    }

    /// Replace the `i`th child of this node with `tree`.
    /// Returns the original child.
    /// 
    /// # Panics
    ///
    /// Panics if this is a leaf node, or if `i` is out of bounds.
    pub fn replace_child(&mut self, i: usize, tree: Tree<D, L>) -> Tree<'f, D, L> {
        let old_tree_id = self.forest_mut().replace_child(self.id, i, tree.id);
        mem::forget(tree);
        Tree::new(self.forest, old_tree_id)
    }

    /// Insert `tree` as the `i`th child of this node.
    /// 
    /// # Panics
    ///
    /// Panics if this is a leaf node, or if `i` is out of bounds.
    pub fn insert_child(&mut self, i: usize, tree: Tree<D, L>) {
        let id = tree.id;
        mem::forget(tree);
        self.forest_mut().insert_child(self.id, i, id);
    }

    /// Remove and return the `i`th child of this node.
    /// 
    /// # Panics
    ///
    /// Panics if this is a leaf node, or if `i` is out of bounds.
    pub fn remove_child(&mut self, i: usize) -> Tree<'f, D, L> {
        let old_tree_id = self.forest_mut().remove_child(self.id, i);
        Tree::new(self.forest, old_tree_id)
    }

    /// Save a bookmark to return to later.
    pub fn bookmark(&mut self) -> Bookmark {
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
    pub fn goto_bookmark(&mut self, mark: Bookmark) -> bool {
        if self.forest().is_valid(mark.id) && self.forest().root(mark.id) == self.root {
            self.id = mark.id;
            true
        } else {
            false
        }
    }

    /// Returns `true` if this is the root of the tree, and `false` if
    /// it isn't (and thus this node has a parent).
    pub fn at_root(&self) -> bool {
        match self.forest().parent(self.id) {
            None => true,
            Some(_) => false
        }
    }

    /// Go to the parent of this node.
    ///
    /// # Panics
    ///
    /// Panics if this is the root of the tree, and there is no parent.
    pub fn goto_parent(&mut self) {
        let id = self.forest().parent(self.id).expect("Forest - root node has no parent!");
        self.id = id;
    }

    /// Go to the `i`th child of this branch node.
    ///
    /// # Panics
    ///
    /// Panics if this is a leaf node, or if `i` is out of bounds.
    pub fn goto_child(&mut self, i: usize) {
        let id = self.forest().child(self.id, i);
        self.id = id;
    }

    // Private //

    fn forest(&self) -> Ref<'f, RawForest<D, L>> {
        self.forest.read_lock()
    }

    fn forest_mut(&self) -> RefMut<'f, RawForest<D, L>> {
        self.forest.write_lock()
    }
}
