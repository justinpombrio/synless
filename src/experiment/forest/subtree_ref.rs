use std::iter::Iterator;

use super::forest::{Id, Forest};
use super::tree::{Tree, Bookmark};


pub struct SubtreeRef<'a> {
    pub (super) root: &'a Tree,
    pub (super) id: Id
}

impl Tree {
    pub fn as_ref<D, L>(&self, _f: &Forest<D, L>) -> SubtreeRef {
        SubtreeRef {
            id: self.id,
            root: self
        }
    }
}

impl<'a> SubtreeRef<'a> {

    // Data Access //

    pub fn is_leaf<D, L>(&self, f: &Forest<D, L>) -> bool {
        f.is_leaf(self.id)
    }
    
    // panics if this is not a branch node
    pub fn data<D, L>(&self, f: &'a Forest<D, L>) -> &'a D {
        f.data(self.id)
    }

    // panics if this is not a leaf node
    pub fn leaf<D, L>(&self, f: &'a Forest<D, L>) -> &'a L {
        f.leaf(self.id)
    }

    // panics if this is not a branch node
    pub fn num_children<D, L>(&self, f: &Forest<D, L>) -> usize {
        f.children(self.id).len()
    }

    // Bookmarks //

    pub fn bookmark<D, L>(&self, _f: &Forest<D, L>) -> Bookmark {
        Bookmark {
            id: self.id
        }
    }

    pub fn lookup_bookmark<D, L>(&self, f: &Forest<D, L>, mark: Bookmark) -> Option<SubtreeRef<'a>> {
        if f.root(mark.id) == self.root.id {
            Some(SubtreeRef {
                root: self.root,
                id: mark.id
            })
        } else {
            None
        }
    }

    // Navigation //
    
    pub fn parent<D, L>(&self, f: &Forest<D, L>) -> Option<SubtreeRef<'a>> {
        match f.parent(self.id) {
            None => None,
            Some(parent) => Some(SubtreeRef {
                root: self.root,
                id: parent
            })
        }
    }

    // panics if size is out of bounds, or if this is a leaf
    pub fn child<D, L>(&self, f: &Forest<D, L>, i: usize) -> SubtreeRef<'a> {
        let child = f.child(self.id, i);
        SubtreeRef {
            root: self.root,
            id: child
        }
    }

    pub fn children<D, L>(&self, f: &Forest<D, L>) -> RefChildrenIter {
        let children = f.children(self.id).clone(); // TODO: avoid clone?
        RefChildrenIter {
            root: self.root,
            children: children.clone(),
            index: 0
        }
    }
}

pub struct RefChildrenIter<'a> {
    root: &'a Tree,
    children: Vec<Id>,
    index: usize
}

impl<'a> Iterator for RefChildrenIter<'a> {
    type Item = SubtreeRef<'a>;
    fn next(&mut self) -> Option<SubtreeRef<'a>> {
        if self.index >= self.children.len() {
            None
        } else {
            let subtree = SubtreeRef {
                root: self.root,
                id: self.children[self.index]
            };
            self.index += 1;
            Some(subtree)
        }
    }
}
