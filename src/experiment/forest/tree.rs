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
pub struct Tree {
    pub (super) id: Id
}

#[derive(Clone, Copy)]
pub struct Bookmark {
    pub (super) id: Id
}

impl Tree {
    /// Constructs a new leaf.
    pub fn new_leaf<D, L>(f: &mut Forest<D, L>, leaf: L) -> Tree {
        Tree {
            id: f.create_leaf(leaf)
        }
    }
    
    /// Constructs a new branch.
    pub fn new_branch<D, L>(f: &mut Forest<D, L>, data: D, children: Vec<Tree>) -> Tree {
        let children = children.into_iter().map(|tree| {
            let id = tree.id;
            mem::forget(tree);
            id
        }).collect();
        Tree {
            id: f.create_branch(data, children)
        }
    }

    /// Trees must be `deleted`, or else they will leak memory.
    /// Call this method on all Trees that you do not surrender
    /// ownership of.
    pub fn delete<D, L>(self, f: &mut Forest<D, L>) {
        f.delete_tree(self.id);
        mem::forget(self)
    }
}

/// To attempt to guard against memory leaks, `drop` panics.
/// Do not drop your trees: `delete` them instead.
impl Drop for Tree {
    fn drop(&mut self) {
        // If the thread is _already_ panicking, that's probably why
        // this tree didn't get recycled, so it's fine.
        if !thread::panicking() {
            panic!("Forest - a tree was not recycled! id:{}", self.id);
        }
    }
}
