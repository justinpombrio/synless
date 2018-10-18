use std::mem;
use std::sync::{RwLock, RwLockWriteGuard, RwLockReadGuard};
use std::thread;

use super::{Id, RawForest};


/// All [Trees](struct.Tree.html) belong to a Forest.
///
/// It is your responsibility to ensure that Trees are kept with the
/// Forest they came from. The methods on Trees will panic if you use
/// them on a different Forest. In practice this is easy because you
/// should only ever have one Forest.
pub struct Forest<D, L> {
    pub (super) lock: RwLock<RawForest<D, L>>
}

/// Every Tree is either a leaf or a branch.
/// A branch contains an ordered list of child Trees, and a data value
/// (the type parameter `Data` or `D`). A leaf contains only a leaf
/// value (the type parameter `Leaf` or `L`).
///
/// To view or modify a Tree, you should either take an immutable
/// reference to it using [`as_ref`](#method.as_ref), or a mutable
/// reference to it using [`as_mut`](#method.as_mut). From there, all
/// operations will require a reference to the RawForest that created the
/// Tree.
///
/// **Trees must be explicitly deleted by the
/// [`delete`](#method.delete) method. Any tree not deleted this way
/// will leak memory. Recycle your trees!**
pub struct Tree<'f, D, L> {
    pub (super) forest: &'f Forest<D, L>,
    pub (super) id: Id
}

#[derive(Clone, Copy)]
pub struct Bookmark {
    pub (super) id: Id
}

impl<D, L> Forest<D, L> {
    /// Construct a new forest. All trees grow in a forest.
    pub fn new() -> Forest<D, L> {
        Forest {
            lock: RwLock::new(RawForest::new())
        }
    }

    // TODO: &self?
    /// Construct a new leaf.
    pub fn new_leaf(&self, leaf: L) -> Tree<D, L> {
        let leaf_id = self.write_lock().create_leaf(leaf);
        Tree::new(self, leaf_id)
    }

    // TODO: &self?
    /// Construct a new branch.
    pub fn new_branch(&self, data: D, children: Vec<Tree<D, L>>) -> Tree<D, L> {
        let child_ids = children.into_iter().map(|tree| {
            let id = tree.id;
            mem::forget(tree);
            id
        }).collect();
        let branch_id = self.write_lock().create_branch(data, child_ids);
        Tree::new(self, branch_id)
    }

    pub (super) fn write_lock(&self) -> RwLockWriteGuard<RawForest<D, L>> {
        self.lock.try_write().expect("Failed to obtain write lock for forest.")
    }

    pub (super) fn read_lock(&self) -> RwLockReadGuard<RawForest<D, L>> {
        self.lock.try_read().expect("Failed to obtain read lock for forest.")
    }
}

impl<'f, D, L> Tree<'f, D, L> {
    pub (super) fn new(forest: &'f Forest<D, L>, id: Id) -> Tree<'f, D, L> {
        Tree {
            forest: forest,
            id: id
        }
    }
}

impl<'f, D, L> Drop for Tree<'f, D, L> {
    fn drop(&mut self) {
        if !thread::panicking() {
            // If it's already panicking, let's not worry too much about cleanup up the hashmap.
            self.forest.write_lock().delete_tree(self.id);
        }
    }
}
