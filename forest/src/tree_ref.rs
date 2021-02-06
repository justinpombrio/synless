use super::node::{Bookmark, Key};
use super::node_slab::NodeSlab;
use std::cell::RefCell;
use std::rc::Rc;

/// An immutable reference to a node in a tree.
#[derive(Clone, Copy)]
pub struct TreeRef<'f, D, L> {
    pub(super) slab: &'f Rc<RefCell<NodeSlab<D, L>>>,
    pub(super) key: Key,
}

impl<'f, D, L> TreeRef<'f, D, L> {
    /// Returns `true` if this is a leaf node, and `false` if this is
    /// a branch node.
    #[allow(clippy::wrong_self_convention)]
    pub fn is_leaf(self) -> bool {
        self.slab.borrow()[self.key].is_leaf()
    }

    /// Calls the closure, giving it read access to the data value at this node.
    ///
    /// # Panics
    ///
    /// Panics if this is not a branch node. (Leaves do not have data.)
    pub fn with_data<R>(self, func: impl FnOnce(&D) -> R) -> R {
        func(&self.slab.borrow()[self.key].data())
    }

    /// Calls the closure, giving it read access to this leaf node.
    ///
    /// # Panics
    ///
    /// Panics if this is a branch node.
    pub fn with_leaf<R>(self, func: impl FnOnce(&L) -> R) -> R {
        func(&self.slab.borrow()[self.key].leaf())
    }

    /// Returns the number of children this node has.
    ///
    /// # Panics
    ///
    /// Panics if this is a leaf node.
    pub fn num_children(self) -> usize {
        self.slab.borrow()[self.key].children().len()
    }

    /// Save a bookmark to return to later.
    pub fn bookmark(&self) -> Bookmark {
        let uuid = self.slab.borrow()[self.key].uuid;
        Bookmark {
            key: self.key,
            uuid,
        }
    }

    /// Return to a previously saved bookmark, as long as that
    /// bookmark's node is present somewhere in this tree. This will
    /// work even if the Tree has been modified since the bookmark was
    /// created. However, it will return `None` if the bookmark's node
    /// has since been deleted, or if it is currently located in a
    /// different tree.
    pub fn lookup_bookmark(self, mark: Bookmark) -> Option<TreeRef<'f, D, L>> {
        let slab = self.slab.borrow();
        if !slab.contains(mark.key) {
            // The bookmark has been deleted.
            return None;
        }
        if slab[mark.key].uuid != mark.uuid {
            // The bookmark has been deleted, and its space reused.
            return None;
        }
        if slab.root(mark.key).uuid != slab.root(self.key).uuid {
            // The bookmark exists, but is in a different tree.
            return None;
        }
        // The bookmark exists, and is in this tree. Thus we can safely return to it.
        Some(TreeRef {
            slab: self.slab,
            key: mark.key,
        })
    }

    /// Get the parent node. Returns `None` if we're already at the
    /// root of the tree.
    pub fn parent(self) -> Option<TreeRef<'f, D, L>> {
        let slab = self.slab.borrow();
        slab[self.key].parent.map(|parent_key| TreeRef {
            key: parent_key,
            slab: self.slab,
        })
    }

    /// Get the `i`th child of this branch node.
    ///
    /// # Panics
    ///
    /// Panics if this is a leaf node, or if `i` is out of bounds.
    pub fn child(self, i: usize) -> TreeRef<'f, D, L> {
        let slab = self.slab.borrow();
        let child_key = slab[self.key].children()[i];
        TreeRef {
            key: child_key,
            slab: self.slab,
        }
    }

    /// Obtain a list of all of the (direct) children of this node.
    ///
    /// # Panics
    ///
    /// Panics if this is a leaf node.
    pub fn children<'a>(&'a self) -> Vec<TreeRef<'f, D, L>> {
        self.slab.borrow()[self.key]
            .children()
            .iter()
            .map(|child_key| TreeRef {
                key: *child_key,
                slab: self.slab,
            })
            .collect()
    }

    /// Determine this node's index among its siblings. Returns `0` when at the
    /// root.
    pub fn index(&self) -> usize {
        let slab = self.slab.borrow();
        match slab[self.key].parent {
            None => return 0,
            Some(parent_key) => {
                for (index, child_key) in slab[parent_key].children().iter().enumerate() {
                    if *child_key == self.key {
                        return index;
                    }
                }
            }
        }
        panic!("Forest::index - not found");
    }
}
