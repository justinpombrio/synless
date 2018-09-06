use std::mem;
use std::thread;

use super::forest::{Id, Forest};


pub struct Tree {
    pub id: Id
}

#[derive(Clone, Copy)]
pub struct Bookmark {
    pub id: Id
}

impl Tree {
    pub fn new_leaf<D, L>(f: &mut Forest<D, L>, leaf: L) -> Tree {
        Tree {
            id: f.create_leaf(leaf)
        }
    }
    
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

    pub fn delete<D, L>(self, f: &mut Forest<D, L>) {
        f.delete_tree(self.id);
        mem::forget(self)
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        // If the thread is _already_ panicking, that's probably why
        // this tree didn't get recycled, so it's fine.
        if !thread::panicking() {
            panic!("Forest - a tree was not recycled! id:{}", self.id);
        }
    }
}
