use std::mem;

use super::forest::{Id, Forest};


pub struct Tree {
    pub id: Id
}

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
        let children = children.into_iter().map(|tree| tree.id).collect();
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
        panic!("Forest - a tree was not recycled! id:{}", self.id);
    }
}
