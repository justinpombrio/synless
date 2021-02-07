use super::node::{Bookmark, Key};
use super::node_slab::NodeSlab;
use super::tree_ref::TreeRef;
use std::cell::RefCell;
use std::mem;
use std::rc::Rc;
use std::thread;
use utility::spanic;

/// A mutable reference to a node in a tree, that owns the tree.
///
/// Every node is either a leaf or a branch.
/// A branch contains an ordered list of child nodes, and a data value
/// (the type parameter `Data` or `D`). A leaf contains only a leaf
/// value (the type parameter `Leaf` or `L`).
///
/// This value owns the entire tree. When it is dropped, the tree is deleted.
///
/// It also grants write access to the tree. Use [`borrow`](#method.borrow) to
/// obtain a shared reference with read-only access.
///
/// All write operations mutably borrow the _entire forest_. While a tree is
/// being mutated, or when some of its data is mutably borrowed (e.g. with
/// `leaf_mut()`), _no other tree in the forest can be accessed_.
pub struct Tree<D, L> {
    slab: Rc<RefCell<NodeSlab<D, L>>>,
    pub(super) key: Key,
}

impl<D, L> Tree<D, L> {
    pub(super) fn new(slab: &Rc<RefCell<NodeSlab<D, L>>>, key: Key) -> Tree<D, L> {
        Tree {
            slab: slab.clone(),
            key,
        }
    }

    /// Obtain an _immutable_ reference to this Tree.
    ///
    /// An Operation on the borrowed tree will **panic** if it happens
    /// concurrently with a mutable operation on any other tree in the forest.
    pub fn borrow(&self) -> TreeRef<D, L> {
        TreeRef {
            slab: &self.slab,
            key: self.key,
        }
    }

    /// Returns `true` if this is a leaf node, and `false` if this is
    /// a branch node.
    pub fn is_leaf(&self) -> bool {
        self.slab.borrow_mut()[self.key].is_leaf()
    }

    /// Calls the closure, giving it read access to the data value at this node.
    pub fn with_data<R>(&self, func: impl FnOnce(&D) -> R) -> R {
        func(&self.slab.borrow()[self.key].data())
    }

    /// Calls the closure, giving it write access to the data value at this node.
    pub fn with_data_mut<R>(&mut self, func: impl FnOnce(&mut D) -> R) -> R {
        func(self.slab.borrow_mut()[self.key].data_mut())
    }

    /// Calls the closure, giving it read access to this leaf node.
    ///
    /// # Panics
    ///
    /// Panics if this is a branch node.
    pub fn with_leaf<R>(&self, func: impl FnOnce(&L) -> R) -> R {
        func(&self.slab.borrow()[self.key].leaf())
    }

    /// Calls the closure, giving it write access to this leaf node.
    ///
    /// # Panics
    ///
    /// Panics if this is a branch node.
    pub fn with_leaf_mut<R>(&mut self, func: impl FnOnce(&mut L) -> R) -> R {
        func(&mut self.slab.borrow_mut()[self.key].leaf_mut())
    }

    /// Returns the number of children this node has.
    ///
    /// # Panics
    ///
    /// Panics if this is a leaf node.
    pub fn num_children(&self) -> usize {
        self.slab.borrow()[self.key].children().len()
    }

    /// Replace the `i`th child of this node with `tree`.
    /// Returns the original child.
    ///
    /// # Panics
    ///
    /// Panics if this is a leaf node, or if `i` is out of bounds.
    pub fn replace_child(&mut self, i: usize, new_child: Tree<D, L>) -> Tree<D, L> {
        // Need to make these changes:
        //   new_child.parent = self;
        //   old_child.parent = None;
        //   self.child[i] = new_child;
        let mut slab = self.slab.borrow_mut();
        slab[new_child.key].parent = Some(self.key);
        let old_child_key = slab[self.key].children()[i];
        slab[old_child_key].parent = None;
        slab[self.key].children_mut()[i] = new_child.key;

        // Don't drop the tree, or it will delete itself from the slab!
        mem::forget(new_child);

        Tree::new(&self.slab, old_child_key)
    }

    /// Insert `tree` as the `i`th child of this node.
    ///
    /// # Panics
    ///
    /// Panics if this is a leaf node, or if `i` is out of bounds.
    pub fn insert_child(&mut self, i: usize, child: Tree<D, L>) {
        // Need to make these changes:
        //   child.parent = self;
        //   self.children.insert_at(i, child)
        let mut slab = self.slab.borrow_mut();
        slab[self.key].children_mut().insert(i, child.key);
        slab[child.key].parent = Some(self.key);

        // Don't drop the tree, or it will delete itself from the slab!
        mem::forget(child);
    }

    /// Remove and return the `i`th child of this node.
    ///
    /// # Panics
    ///
    /// Panics if this is a leaf node, or if `i` is out of bounds.
    pub fn remove_child(&mut self, i: usize) -> Tree<D, L> {
        // Need to make these changes:
        //   child = self.children.remove_at(i);
        //   child.parent = None;
        let mut slab = self.slab.borrow_mut();
        let child_key = slab[self.key].children_mut().remove(i);
        slab[child_key].parent = None;

        Tree::new(&self.slab, child_key)
    }

    /// Save a bookmark to return to later.
    pub fn bookmark(&self) -> Bookmark {
        let uuid = self.slab.borrow()[self.key].uuid;
        Bookmark {
            key: self.key,
            uuid,
        }
    }

    /// Jump to a previously saved bookmark, as long as that
    /// bookmark's node is present somewhere in this tree. This will
    /// work even if the Tree has been modified since the bookmark was
    /// created. However, it will return `false` if the bookmark's node
    /// has since been deleted, or if it is currently located in a
    /// different tree.
    pub fn goto_bookmark(&mut self, mark: Bookmark) -> bool {
        let slab = self.slab.borrow();
        if !slab.contains(mark.key) {
            // The bookmark has been deleted.
            return false;
        }
        if slab[mark.key].uuid != mark.uuid {
            // The bookmark has been deleted, and its space reused.
            return false;
        }
        if slab.root(mark.key).uuid != slab.root(self.key).uuid {
            // The bookmark exists, but is in a different tree.
            return false;
        }
        // The bookmark exists, and is in this tree. Thus we can safely jump to it.
        self.key = mark.key;
        true
    }

    /// Returns `true` if this is the root of the tree, and `false` if
    /// it isn't (and thus this node has a parent).
    pub fn is_at_root(&self) -> bool {
        self.slab.borrow()[self.key].parent.is_none()
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
        spanic!("Forest::index - not found");
    }

    /// Determine the number of siblings that this node has, including itself.
    /// When at the root, returns 1.
    pub fn num_siblings(&self) -> usize {
        let slab = self.slab.borrow();
        match slab[self.key].parent {
            None => 1,
            Some(parent_key) => slab[parent_key].children().len(),
        }
    }

    /// Go to the parent of this node. Returns this node's index among its
    /// siblings (so that you can return to it later).
    ///
    /// # Panics
    ///
    /// Panics if this is the root of the tree, and there is no parent.
    pub fn goto_parent(&mut self) -> usize {
        let slab = self.slab.borrow();
        match slab[self.key].parent {
            None => spanic!("Forest::goto_parent - root node has no parent"),
            Some(parent_key) => {
                for (index, child_key) in slab[parent_key].children().iter().enumerate() {
                    if *child_key == self.key {
                        self.key = parent_key;
                        return index;
                    }
                }
            }
        }
        spanic!("Forest::goto_parent - not found");
    }

    /// Go to the `i`th child of this branch node.
    ///
    /// # Panics
    ///
    /// Panics if this is a leaf node, or if `i` is out of bounds.
    pub fn goto_child(&mut self, i: usize) {
        let slab = self.slab.borrow();
        self.key = slab[self.key].children()[i];
    }

    /// Go to the root of this tree.
    pub fn goto_root(&mut self) {
        self.key = self.slab.borrow().root_key(self.key);
    }
}

impl<D, L> Drop for Tree<D, L> {
    fn drop(&mut self) {
        if thread::panicking() {
            // If it's already panicking, let's not worry too much about cleaning up the hashmap.
            return;
        }
        // If we did nothing, then the slab would retain the tree. We must call `slab.remove()` on
        // each node in the tree.
        self.slab.borrow_mut().free_tree(self.key);
    }
}
