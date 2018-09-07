use std::iter::Iterator;

use super::forest::{Id, Forest};
use super::tree::{Tree, Bookmark};


/// An immutable reference to a Tree.
///
/// This reference will begin pointing at the root of the Tree, but
/// from there you can get references to other parts of the tree.
///
/// Essentially all operations require a reference to the Forest that
/// created the Tree as their first argument.
pub struct SubtreeRef<'a> {
    pub (super) root: &'a Tree,
    pub (super) id: Id
}

impl Tree {
    /// Obtain an immutable reference to this Tree.
    pub fn as_ref<D, L>(&self, _f: &Forest<D, L>) -> SubtreeRef {
        SubtreeRef {
            id: self.id,
            root: self
        }
    }
}

impl<'a> SubtreeRef<'a> {

    /// Returns `true` if this is a leaf node, and `false` if this is
    /// a branch node.
    pub fn is_leaf<D, L>(&self, f: &Forest<D, L>) -> bool {
        f.is_leaf(self.id)
    }

    /// Obtain a reference to the data value at this node.
    ///
    /// # Panics
    ///
    /// Panics if this is not a branch node. (Leaves do not have data.)
    pub fn data<D, L>(&self, f: &'a Forest<D, L>) -> &'a D {
        f.data(self.id)
    }

    /// Obtain a reference to the leaf value at this node.
    ///
    /// # Panics
    ///
    /// Panics if this is a branch node.
    pub fn leaf<D, L>(&self, f: &'a Forest<D, L>) -> &'a L {
        f.leaf(self.id)
    }

    /// Returns the number of children this node has.
    ///
    /// # Panics
    ///
    /// Panics if this is not a branch node.
    pub fn num_children<D, L>(&self, f: &Forest<D, L>) -> usize {
        f.children(self.id).len()
    }

    /// Save a bookmark to return to later.
    pub fn bookmark<D, L>(&self, _f: &Forest<D, L>) -> Bookmark {
        Bookmark {
            id: self.id
        }
    }

    /// Return to a previously saved bookmark, as long as that
    /// bookmark's node is present somewhere in this tree. This will
    /// work even if the Tree has been modified since the bookmark was
    /// created. However, it will return `None` if the bookmark's node
    /// has since been deleted, or if it is currently located in a
    /// different tree.
    pub fn lookup_bookmark<D, L>(&self, f: &Forest<D, L>, mark: Bookmark) -> Option<SubtreeRef<'a>> {
        if f.is_valid(mark.id) && f.root(mark.id) == self.root.id {
            Some(SubtreeRef {
                root: self.root,
                id: mark.id
            })
        } else {
            None
        }
    }

    /// Get the parent node. Returns `None` if we're already at the
    /// root of the tree.
    pub fn parent<D, L>(&self, f: &Forest<D, L>) -> Option<SubtreeRef<'a>> {
        match f.parent(self.id) {
            None => None,
            Some(parent) => Some(SubtreeRef {
                root: self.root,
                id: parent
            })
        }
    }

    /// Get the `i`th child, assuming that we're on a branch node.
    ///
    /// # Panics
    ///
    /// Panics if this is a leaf node, or if `i` is out of bounds.
    pub fn child<D, L>(&self, f: &Forest<D, L>, i: usize) -> SubtreeRef<'a> {
        let child = f.child(self.id, i);
        SubtreeRef {
            root: self.root,
            id: child
        }
    }

    /// Obtain an iterator over all of the (direct) children of this node.
    pub fn children<D, L>(&self, f: &Forest<D, L>) -> RefChildrenIter {
        let children = f.children(self.id).clone(); // TODO: avoid clone?
        RefChildrenIter {
            root: self.root,
            children: children,
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
