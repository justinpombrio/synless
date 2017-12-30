use std::mem;

use tree::path::Path;
use tree::node::Node;
use tree::tree::Tree;
use tree::tree_ref::TreeRef;


/// A mutable reference to a subtree of a containing tree.
/// The containing tree is called the root.
/// Calling `.goto_child(i)` or `.goto_parent()` will change the
/// subtree, but leave the root the same.
///
/// Notice that, unlike for TreeRefs, navigation operations consume
/// the TreeMut.
pub struct TreeMut<'t, 'l : 't> {
    root: &'t mut Tree<'l>,
    path: Path
}

impl<'t, 'l> Tree<'l> {
    pub(crate) fn as_mut(&'t mut self) -> TreeMut<'t, 'l> {
        TreeMut{
            root: self,
            path: vec!()
        }
    }
}

impl<'t, 'l> TreeMut<'t, 'l> {
    fn tree(&self) -> &Tree<'l> {
        &self.root[&self.path]
    }

    fn tree_parent(&self) -> &Tree<'l> {
        &self.root[&self.path[0..self.path.len()-1]]
    }

    fn tree_mut(&mut self) -> &mut Tree<'l> {
        &mut self.root[&self.path]
    }

    fn ancestor_mut(&mut self, gen: usize) -> &mut Tree<'l> {
        // gen is the number of generations *back* to go.
        &mut self.root[&self.path[0 .. self.path.len() - gen]]
    }

    // All Trees //

    /// *This must be called every time this tree or one of its
    /// children is modifed.*
    pub fn update(&mut self) {
        let n = self.path.len();
        for gen in 0 .. n + 1 {
            self.ancestor_mut(gen).local_update();
        }
    }

    pub fn as_ref<'t2>(&'t2 self) -> TreeRef<'t2, 'l> {
        TreeRef::new(&self.root, self.path.clone())
    }

    pub fn replace(&mut self, tree: Tree<'l>) -> Tree<'l> {
        mem::replace(&mut self.tree_mut(), tree)
    }

    // Local Info //

    pub fn node(&self) -> &Node {
        &self.tree().node
    }

    pub fn breadcrumb(&self) -> usize {
        self.tree().node.breadcrumb
    }

    pub fn breadcrumb_mut(&mut self) -> &mut usize {
        &mut self.tree_mut().node.breadcrumb
    }

    pub fn len(&self) -> usize {
        self.tree().len()
    }

    pub fn path(&self) -> &Path { // for testing
        &self.path
    }

    // Texty Trees //

    pub fn is_texty(&self) -> bool {
        self.tree().is_texty()
    }

    pub fn text(&self) -> &String {
        self.tree().text()
    }

    /// DANGER: If the text is modified, its parent's node must be updated!
    pub fn text_mut(&mut self) -> &mut String {
        self.tree_mut().text_mut()
    }

    // Foresty Trees //

    pub fn is_foresty(&self) -> bool {
        self.tree().is_foresty()
    }
/*
    pub fn forest(&self) -> &Vec<Tree<'l>> {
        self.tree().forest()
    }
*/
    /// DANGER: If the forest is modified, its parent's node must be updated!
    pub fn forest_mut(&mut self) -> &mut Vec<Tree<'l>> {
        self.tree_mut().forest_mut()
    }

    pub fn num_children(&self) -> usize {
        self.tree().forest().len()
    }

    pub fn goto_child(&mut self, i: usize) {
        self.path.push(i);
    }

    // Trees with Parents //
    
    pub fn has_parent(&self) -> bool {
        !self.path.is_empty()
    }
    
    pub fn index(&self) -> usize {
        *self.path.last().expect("tree_mut: index")
    }

    pub fn num_siblings(&self) -> usize {
        self.tree_parent().forest().len()
    }

    pub fn goto_parent(&mut self) -> usize {
        self.path.pop().unwrap()
    }
}


#[cfg(test)]
mod tests {
    use tree::tree::example_tree;

    #[test]
    fn test_tree_mut_goto() {
        let mut tree = example_tree();
        let mut r = tree.as_mut();
        r.goto_child(1);
        r.goto_parent();
        r.goto_child(1);
        r.goto_child(0);
        r.goto_parent();
        assert_eq!(r.as_ref().root().len(), 2);
        assert_eq!(r.len(), 1);
    }
}
