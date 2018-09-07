use std::marker::PhantomData;
use std::mem;
use std::thread;

use super::forest::{Id, Forest};


/// Every Tree is either a leaf or a branch.
/// A branch contains an ordered list of child Trees, and a data value
/// (the type parameter `Data` or `D`). A leaf contains only a leaf
/// value (the type parameter `Leaf` or `L`).
///
/// To view or modify a Tree, you should either take an immutable
/// reference to it using [`as_ref`](#method.as_ref), or a mutable
/// reference to it using [`as_mut`](#method.as_mut). From there, all
/// operations will require a reference to the Forest that created the
/// Tree.
///
/// **Trees must be explicitly deleted by the
/// [`delete`](#method.delete) method. Any tree not deleted this way
/// will leak memory. Recycle your trees!**
pub struct Tree<D, L> {
    pub (super) id: Id,
    phantom_data: PhantomData<D>,
    phantom_leaf: PhantomData<L>
}

#[derive(Clone, Copy)]
pub struct Bookmark {
    pub (super) id: Id
}

impl<D, L> Tree<D, L> {
    /// Constructs a new leaf.
    pub fn new_leaf(f: &mut Forest<D, L>, leaf: L) -> Tree<D, L> {
        Tree::new(f.create_leaf(leaf))
    }
    
    /// Constructs a new branch.
    pub fn new_branch(f: &mut Forest<D, L>, data: D, children: Vec<Tree<D, L>>)
                      -> Tree<D, L>
    {
        let children = children.into_iter().map(|tree| {
            let id = tree.id;
            mem::forget(tree);
            id
        }).collect();
        Tree::new(f.create_branch(data, children))
    }

    pub (super) fn new(id: Id) -> Tree<D, L> {
        Tree {
            id: id,
            phantom_data: PhantomData,
            phantom_leaf: PhantomData
        }
    }

    /// Trees must be `deleted`, or else they will leak memory.
    /// Call this method on all Trees that you do not surrender
    /// ownership of.
    pub fn delete(self, f: &mut Forest<D, L>) {
        f.delete_tree(self.id);
        mem::forget(self)
    }
}

/// To attempt to guard against memory leaks, `drop` panics.
/// Do not drop your trees: `delete` them instead.
impl<D, L> Drop for Tree<D, L> {
    fn drop(&mut self) {
        // If the thread is _already_ panicking, that's probably why
        // this tree didn't get recycled, so it's fine.
        if !thread::panicking() {
            panic!("Forest - a tree was not recycled! id:{}", self.id);
        }
    }
}
