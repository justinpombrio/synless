use std::ops::{Index, IndexMut};

use tree::path::Path;

use self::Children::{ForestChildren, TextChildren};


/// A document being edited, represented as a Tree.
/// Because how else would you represent it?
///
/// Every tree is either *foresty*, if it has trees for children,
/// or *texty*, if it contains text.
#[derive(Debug)]
pub struct Tree<'l> {
    pub node: Node<'l>,
    children: Children<'l>
}

#[derive(Debug)]
enum Children<'l> {
    ForestChildren(Vec<Tree<'l>>),
    TextChildren(String) // TODO: Allow Tree children interspersed
}


// Children //

impl<'l> Children<'l> {

    // TODO: efficiency
    // Returns apparent length: text has a phantom character at the end
    fn len(&self) -> usize {
        match self {
            &ForestChildren(ref trees)  => trees.len(),
            &TextChildren(ref childs) => childs.chars().count() + 1
        }
    }

    fn is_texty(&self) -> bool {
        match self {
            &ForestChildren(_) => false,
            &TextChildren(_) => true
        }
    }

    fn is_foresty(&self) -> bool {
        match self {
            &ForestChildren(_) => true,
            &TextChildren(_) => false
        }
    }
}


// Trees //

impl<'l> Tree<'l> {

    // Tree Constructors

    /// Construct a new hole.
    pub fn new_hole() -> Tree<'l> {
        Tree{
            node: Node::new_hole(),
            children: ForestChildren(vec!())
        }
    }

    /// Construct a tree with empty children/text.
    pub fn new(construct: &'l Construct) -> Tree<'l> {
        if construct.arity.is_text() {
            Tree::new_text(construct, "")
        } else {
            let arity = construct.arity.arity();
            let mut children = vec!();
            for _ in 0..arity {
                children.push(Tree::new_hole());
            }
            Tree::new_forest(construct, children)
        }
    }

    /// Construct a new foresty tree.
    pub fn new_forest(construct: &'l Construct, children: Vec<Tree<'l>>) -> Tree<'l> {
        let node = {
            let child_nodes = children.iter().map(|t| &t.node).collect();
            Node::new_forest(construct, child_nodes)
        };
        Tree{
            node: node,
            children: ForestChildren(children)
        }
    }

    /// Construct a new texty tree.
    pub fn new_text(construct: &'l Construct, text: &str) -> Tree<'l> {
        let node = Node::new_text(construct, text);
        Tree{
            node: node,
            children: TextChildren(text.to_string())
        }
    }

    /// *This must be called every time this tree or one of its
    /// children is modifed, on this tree and every tree of to the root.*
    pub fn local_update(&mut self) {
        match &self.children {
            &ForestChildren(ref forest) => {
                let child_nodes = forest.into_iter().map(|n| &n.node).collect();
                self.node.update_forest(child_nodes);
            }
            &TextChildren(ref text) => self.node.update_text(text)
        }
    }

    // All Trees //

    /// The number of children of a foresty tree,
    /// or the number of characters plus one of a texty tree.
    /// (It is plus one because there is a phantom character at the
    /// end of text to make it easier to edit.)
    pub fn len(&self) -> usize {
        self.children.len()
    }

    // Texty Trees //

    /// Is this tree texty?
    pub fn is_texty(&self) -> bool {
        self.children.is_texty()
    }

    /// The text of a texty tree. Panics on foresty trees.
    pub fn text(&self) -> &String {
        match &self.children {
            &ForestChildren(_)        => error_not_texty(),
            &TextChildren(ref text) => text
        }
    }

    /// The text of a texty tree. Panics on foresty trees.
    pub fn text_mut(&mut self) -> &mut String {
        match &mut self.children {
            &mut ForestChildren(_)            => error_not_texty(),
            &mut TextChildren(ref mut text) => text
        }
    }

    // Foresty Trees //

    /// Is this tree foresty?
    pub fn is_foresty(&self) -> bool {
        self.children.is_foresty()
    }

    /// Return the children of a foresty tree. Panics on texty trees.
    pub fn forest(&self) -> &Vec<Tree<'l>> {
        match &self.children {
            &ForestChildren(ref forest) => forest,
            &TextChildren(_)          => error_not_foresty()
        }
    }

    /// Return the children of a foresty tree. Panics on texty trees.
    pub fn forest_mut(&mut self) -> &mut Vec<Tree<'l>> {
        match &mut self.children {
            &mut ForestChildren(ref mut forest) => forest,
            &mut TextChildren(_)              => error_not_foresty()
        }
    }
}


// Index //

impl<'l> Index<usize> for Children<'l> {
    type Output = Tree<'l>;
    fn index(&self, index: usize) -> &Tree<'l> {
        match self {
            &ForestChildren(ref forest) => {
                match forest.get(index) {
                    None       => error_invalid_path(),
                    Some(tree) => tree
                }
            }
            &TextChildren(_) => error_invalid_path()
        }
    }
}

impl<'l> Index<usize> for Tree<'l> {
    ///
    type Output = Tree<'l>;
    /// Get the i'th child, or error.
    fn index(&self, i: usize) -> &Tree<'l> {
        self.children.index(i)
    }
}

impl<'a, 'l> Index<&'a [usize]> for Tree<'l> {
    ///
    type Output = Tree<'l>;
    /// Lookup the path (represented as a slice), or error.
    fn index(&self, path: &[usize]) -> &Tree<'l> {
        match path.split_first() {
            None             => self,
            Some((&i, path)) => self[i].index(path)
        }
    }
}

impl<'a, 'l> Index<&'a Path> for Tree<'l> {
    ///
    type Output = Tree<'l>;
    /// Lookup the path, or error.
    fn index(&self, path: &Path) -> &Tree<'l> {
        self.index(path.as_slice())
    }
}


// Index Mut //

impl<'l> IndexMut<usize> for Children<'l> {
    fn index_mut(&mut self, i: usize) -> &mut Tree<'l> {
        match self {
            &mut ForestChildren(ref mut forest) => {
                match forest.get_mut(i) {
                    None       => error_invalid_path(),
                    Some(tree) => tree
                }
            }
            &mut TextChildren(_) => error_invalid_path()
        }
    }
}

impl<'l> IndexMut<usize> for Tree<'l> {
    /// Get a mutable reference to the `i`th child, or error.
    fn index_mut(&mut self, index: usize) -> &mut Tree<'l> {
        self.children.index_mut(index)
    }
}

impl<'a, 'l> IndexMut<&'a [usize]> for Tree<'l> {
    /// Get a mutable reference to the path (represented as a slice),
    /// or error.
    fn index_mut(&mut self, path: &[usize]) -> &mut Tree<'l> {
        match path.split_first() {
            None             => self,
            Some((&i, path)) => self[i].index_mut(path)
        }
    }
}

impl<'a, 'l> IndexMut<&'a Path> for Tree<'l> {
    /// Get a mutable reference to the path, or error.
    fn index_mut(&mut self, path: &Path) -> &mut Tree<'l> {
        self.index_mut(path.as_slice())
    }
}


// Errors //

fn error_invalid_path() -> ! {
    panic!("tree - encountered invalid path!");
}

fn error_not_texty() -> ! {
    panic!("tree - expected a texty tree!");
}

fn error_not_foresty() -> ! {
    panic!("tree - expected a foresty tree!");
}

#[cfg(test)]
pub fn example_tree() -> Tree<'static> {
    Tree::new_forest(&TEST_FOREST, vec!(
        Tree::new_text(&TEST_TEXT, "hi"),
        Tree::new_forest(&TEST_FOREST, vec!(
            Tree::new_text(&TEST_TEXT, "world")))))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trees() {
        let mut tree = example_tree();
        assert_eq!(tree.len(), 2);
        assert_eq!(tree[0].len(), 3);
        assert_eq!(tree[1].len(), 1);
        assert_eq!(tree[&vec!()].len(), 2);
        assert_eq!(tree[&vec!(0)].len(), 3);
        assert_eq!(tree[&vec!(1)].len(), 1);
        assert_eq!(tree[&vec!(1, 0)].len(), 6);

        {
            let root = &tree;
            let hi = &tree[0];

            assert!(root.is_foresty());
            assert!(!root.is_texty());
            assert!(!hi.is_foresty());
            assert!(hi.is_texty());

            assert_eq!(hi.text().as_str(), "hi");
            assert_eq!(root.forest().len(), 2);
        }
        {
            let world = &mut tree[1][0];
            assert_eq!(world.text_mut().as_str(), "world");
        }
        {
            let sub = &mut tree[1];
            assert_eq!(sub.forest_mut().len(), 1);
        }
    }

    #[test]
    #[should_panic(expected = "expected a texty tree")]
    fn test_text_panic() {
        example_tree().text();
    }

    #[test]
    #[should_panic(expected = "expected a foresty tree")]
    fn test_forest_panic() {
        example_tree()[0].forest();
    }
}
