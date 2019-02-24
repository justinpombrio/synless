use std::cell::Ref;
use std::iter::Iterator;

use crate::forest::{Id, RawForest};
use crate::tree::{Bookmark, Forest, ReadData, ReadLeaf, Tree};

/// An immutable reference to a node in a tree.
#[derive(Clone)]
pub struct TreeRef<'f, D, L> {
    pub(super) forest: &'f Forest<D, L>,
    pub(super) root: Id,
    pub(super) id: Id,
}

impl<D, L> Tree<D, L> {
    /// Obtain an _immutable_ reference to this Tree.
    ///
    /// An Operation on the borrowed tree will **panic** if it happens
    /// concurrently with a mutable operation on any other tree in the forest.
    pub fn borrow(&self) -> TreeRef<D, L> {
        TreeRef {
            forest: &self.forest,
            root: self.root,
            id: self.id,
        }
    }
}

impl<'f, D, L> TreeRef<'f, D, L> {
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
            id: self.id,
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
            id: self.id,
        }
    }

    /// Returns the number of children this node has.
    ///
    /// # Panics
    ///
    /// Panics if this is a leaf node.
    pub fn num_children(&self) -> usize {
        self.forest().children(self.id).count()
    }

    /// Save a bookmark to return to later.
    pub fn bookmark(&self) -> Bookmark {
        Bookmark { id: self.id }
    }

    /// Return to a previously saved bookmark, as long as that
    /// bookmark's node is present somewhere in this tree. This will
    /// work even if the Tree has been modified since the bookmark was
    /// created. However, it will return `None` if the bookmark's node
    /// has since been deleted, or if it is currently located in a
    /// different tree.
    pub fn lookup_bookmark(&self, mark: Bookmark) -> Option<TreeRef<'f, D, L>> {
        if self.forest().is_valid(mark.id) && self.forest().root(mark.id) == self.root {
            Some(TreeRef {
                forest: self.forest,
                root: self.root,
                id: mark.id,
            })
        } else {
            None
        }
    }

    /// Get the parent node. Returns `None` if we're already at the
    /// root of the tree.
    pub fn parent(&self) -> Option<TreeRef<'f, D, L>> {
        match self.forest().parent(self.id) {
            None => None,
            Some(parent) => Some(TreeRef {
                forest: self.forest,
                root: self.root,
                id: parent,
            }),
        }
    }

    /// Get the `i`th child of this branch node.
    ///
    /// # Panics
    ///
    /// Panics if this is a leaf node, or if `i` is out of bounds.
    pub fn child(&self, i: usize) -> TreeRef<'f, D, L> {
        let child = self.forest().child(self.id, i);
        TreeRef {
            forest: self.forest,
            root: self.root,
            id: child,
        }
    }

    /// Obtain an iterator over all of the (direct) children of this node.
    ///
    /// # Panics
    ///
    /// Panics if this is a leaf node.
    pub fn children(&self) -> impl Iterator<Item = TreeRef<'f, D, L>> {
        self.forest()
            .children(self.id)
            .map(|&child_id| TreeRef {
                forest: self.forest,
                root: self.root,
                id: child_id,
            })
            .collect::<Vec<_>>()
            .into_iter()
    }

    /// Determine this node's index among its siblings. Returns `0` when at the
    /// root.
    pub fn index(&self) -> usize {
        self.forest().index(self.id)
    }

    /// Make a copy of this tree.
    pub fn to_owned_tree(&self) -> Tree<D, L>
    where
        D: Clone,
        L: Clone,
    {
        if self.is_leaf() {
            self.forest.new_leaf(self.leaf().clone())
        } else {
            let children = self.children().map(|child| child.to_owned_tree()).collect();
            self.forest.new_branch(self.data().clone(), children)
        }
    }

    // Private //

    fn forest(&self) -> Ref<'f, RawForest<D, L>> {
        self.forest.read_lock()
    }
}
