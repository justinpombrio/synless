use super::forest::{Id, Forest};
use super::tree::{Tree, Bookmark};


pub struct SubtreeMut<'a> {
    root: &'a mut Tree,
    id: Id
}

impl Tree {
    pub fn as_mut<D, L>(&mut self) -> SubtreeMut {
        SubtreeMut {
            id: self.id,
            root: self
        }
    }
}

impl<'a> SubtreeMut<'a> {
    
    // Data Access //

    pub fn is_leaf<D, L>(&self, f: &Forest<D, L>) -> bool {
        f.is_leaf(self.id)
    }
    
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

    // Data Mutation //

    pub fn data_mut<D, L>(&mut self, f: &'a mut Forest<D, L>) -> &'a mut D {
        f.data_mut(self.id)
    }

    // panics if this is not a leaf node
    pub fn leaf_mut<D, L>(&mut self, f: &'a mut Forest<D, L>) -> &'a L {
        f.leaf_mut(self.id)
    }

    pub fn replace_child<D, L>(&mut self, f: &'a mut Forest<D, L>, i: usize, tree: Tree) -> Tree {
        Tree {
            id: f.replace_child(self.id, i, tree.id)
        }
    }

    pub fn insert_child<D, L>(&mut self, f: &'a mut Forest<D, L>, i: usize, tree: Tree) {
        f.insert_child(self.id, i, tree.id)
    }

    pub fn remove_child<D, L>(&mut self, f: &'a mut Forest<D, L>, i: usize) -> Tree {
        Tree {
            id: f.remove_child(self.id, i)
        }
    }

    // Bookmarks //

    pub fn bookmark<D, L>(&mut self, _f: &mut Forest<D, L>) -> Bookmark {
        Bookmark {
            id: self.id
        }
    }

    pub fn goto_bookmark<D, L>(&mut self, f: &mut Forest<D, L>, mark: Bookmark) -> bool {
        if f.root(mark.id) == self.root.id {
            self.id = mark.id;
            true
        } else {
            false
        }
    }

    // Navigation //

    pub fn goto_parent<D, L>(&mut self, f: &mut Forest<D, L>) -> bool {
        match f.parent(self.id) {
            None => false,
            Some(parent) => {
                self.id = parent;
                true
            }
        }
    }

    // panics if size is out of bounds, or if this isn't a leaf
    pub fn goto_child<D, L>(&mut self, f: &mut Forest<D, L>, i: usize) {
        self.id = f.child(self.id, i);
    }
}
