use geometry::*;
use syntax::{BoundSet, LayoutRegion};
use tree::path::{Path, extend_path};
use tree::node::Node;
use tree::tree::Tree;

/// A reference to a subtree of a containing tree.
/// The containing tree is called the root.
/// Obtain one by calling `.as_ref()` on a Tree.
/// Calling `.child(i)` will change the subtree, but leave the root
/// the same.
#[derive(Clone)]
pub struct TreeRef<'t, 'l : 't> {
    root: &'t Tree<'l>,
    path: Path
}

impl<'t, 'l> Tree<'l> {
    /// Obtain a TreeRef, whose root and subtree are both this tree.
    pub fn as_ref(&'t self) -> TreeRef<'t, 'l> {
        TreeRef{
            root: &self,
            path: vec!()
        }
    }
}

impl<'t, 'l> TreeRef<'t, 'l> {

    pub(in tree) fn new(root: &'t Tree<'l>, path: Path) -> TreeRef<'t, 'l> {
        TreeRef{
            root: root,
            path: path
        }
    }

    fn tree(&self) -> &Tree<'l> {
        &self.root[&self.path]
    }

    /// Lay out the document to fit within the Bound.
    pub fn lay_out(&self, region: Region) -> LayoutRegion {
        let child_bounds = if self.is_foresty() {
            let n = self.len();
            (0..n).map(|i| self.child(i).node().bounds.clone()).collect()
        } else {
            vec!(BoundSet::literal(self.text()))
        };
        let empty_text = self.is_texty() && self.text().is_empty();
        let syn = &self.node().construct.syntax;
        let arity = self.node().construct.arity.arity();
        let layouts = syn.lay_out(arity, &child_bounds, empty_text);
        let region = Region{
            pos: region.pos,
            bound: self.node().bounds.fit_bound(region.bound)
        };
        layouts.pick(region.bound).regionize(region)
    }

    /// The root (the containing tree).
    pub fn root(&self) -> TreeRef<'t, 'l> {
        TreeRef{
            root: self.root,
            path: vec!()
        }
    }

    /// The path from the root to this tree.
    pub fn path(&self) -> &Path {
        &self.path
    }

    // All Trees //

    /// This tree's node.
    pub fn node(&self) -> &Node<'l> {
        &self.tree().node
    }

    /// The number of children of a foresty tree,
    /// or the number of characters plus one of a texty tree.
    pub fn len(&self) -> usize {
        self.tree().len()
    }

    // Texty Trees //

    /// Is this tree texty?
    pub fn is_texty(&self) -> bool {
        self.tree().is_texty()
    }

    /// The text of a texty tree. Panics on foresty trees.
    pub fn text(&self) -> &String {
        self.tree().text()
    }

    // Foresty Trees //

    /// Is this tree foresty?
    pub fn is_foresty(&self) -> bool {
        self.tree().is_foresty()
    }

    /// Get the `i`th child.
    pub fn child(&self, i: usize) -> TreeRef<'t, 'l> {
        TreeRef{
            root: &self.root,
            path: extend_path(&self.path, i)
        }
    }
}


#[cfg(test)]
mod tests {
    use tree::tree::example_tree;

    #[test]
    fn test_tree_ref_child() {
        let tree = example_tree();
        let r = tree.as_ref().child(1).child(0);
        assert_eq!(r.root().len(), 2);
        assert_eq!(r.len(), 6);
    }
}
