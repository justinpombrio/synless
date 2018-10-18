use std::mem;
use std::cell::{RefCell, Ref, RefMut};
use std::ops::{Deref, DerefMut};
use std::thread;

use super::{Id, RawForest};


/// All [Trees](struct.Tree.html) belong to a Forest.
///
/// It is your responsibility to ensure that Trees are kept with the
/// Forest they came from. The methods on Trees will panic if you use
/// them on a different Forest.
pub struct Forest<D, L> {
    pub (super) lock: RefCell<RawForest<D, L>>
}

/// Every Tree is either a leaf or a branch.
/// A branch contains an ordered list of child Trees, and a data value
/// (the type parameter `Data` or `D`). A leaf contains only a leaf
/// value (the type parameter `Leaf` or `L`).
///
/// To view or modify a Tree, take either an immutable
/// reference to it using [`as_ref`](#method.as_ref), or a mutable
/// reference to it using [`as_mut`](#method.as_mut).
pub struct Tree<'f, D, L> {
    pub (super) forest: &'f Forest<D, L>,
    pub (super) id: Id
}

#[derive(Clone, Copy)]
pub struct Bookmark {
    pub (super) id: Id
}

impl<D, L> Forest<D, L> {
    /// Construct a new forest.
    pub fn new() -> Forest<D, L> {
        Forest {
            lock: RefCell::new(RawForest::new())
        }
    }

    /// Construct a new leaf.
    pub fn new_leaf(&self, leaf: L) -> Tree<D, L> {
        let leaf_id = self.write_lock().create_leaf(leaf);
        Tree::new(self, leaf_id)
    }

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

    pub (super) fn write_lock(&self) -> RefMut<RawForest<D, L>> {
        self.lock.try_borrow_mut().expect("Failed to obtain write lock for forest.")
    }

    pub (super) fn read_lock(&self) -> Ref<RawForest<D, L>> {
        self.lock.try_borrow().expect("Failed to obtain read lock for forest.")
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


// Derefs //

/// Provides read access to a Tree's data. Released on drop.
pub struct ReadData<'f, D, L> {
    pub (super) guard: Ref<'f, RawForest<D, L>>,
    pub (super) id: Id
}

/// Provides read access to a Tree's leaf. Released on drop.
pub struct ReadLeaf<'f, D, L> {
    pub (super) guard: Ref<'f, RawForest<D, L>>,
    pub (super) id: Id
}

/// Provides write access to a Tree's data. Released on drop.
pub struct WriteData<'f, D, L> {
    pub (super) guard: RefMut<'f, RawForest<D, L>>,
    pub (super) id: Id
}

/// Provides write access to a Tree's leaf. Released on drop.
pub struct WriteLeaf<'f, D, L> {
    pub (super) guard: RefMut<'f, RawForest<D, L>>,
    pub (super) id: Id
}

impl<'f, D, L> Deref for ReadData<'f, D, L> {
    type Target = D;
    fn deref(&self) -> &D {
        self.guard.data(self.id)
    }
}

impl<'f, D, L> Deref for ReadLeaf<'f, D, L> {
    type Target = L;
    fn deref(&self) -> &L {
        self.guard.leaf(self.id)
    }
}

impl<'f, D, L> Deref for WriteData<'f, D, L> {
    type Target = D;
    fn deref(&self) -> &D {
        self.guard.data(self.id)
    }
}

impl<'f, D, L> DerefMut for WriteData<'f, D, L> {
    fn deref_mut(&mut self) -> &mut D {
        self.guard.data_mut(self.id)
    }
}

impl<'f, D, L> Deref for WriteLeaf<'f, D, L> {
    type Target = L;
    fn deref(&self) -> &L {
        self.guard.leaf(self.id)
    }
}

impl<'f, D, L> DerefMut for WriteLeaf<'f, D, L> {
    fn deref_mut(&mut self) -> &mut L {
        self.guard.leaf_mut(self.id)
    }
}
