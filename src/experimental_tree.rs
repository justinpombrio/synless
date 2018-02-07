use std::ops::{Index, IndexMut};

use self::Tree::{Forest, Text};

pub trait TreeNode {
    fn empty() -> Self;
    fn update_text(&mut self, text: &str);
    fn update_forest(&mut self, child_nodes: Vec<&Self>);
}

pub enum Tree<Node : TreeNode> {
    Forest(Node, Vec<Tree<Node>>),
    Text(Node, String)
}

impl<Node : TreeNode> Tree<Node> {
    
    // Tree Constructors //

    /// Construct a new foresty tree.
    pub fn new_forest(node: Node, children: Vec<Tree<Node>>) -> Tree<Node> {
        let mut tree = Forest(node, children);
        tree.update();
        tree
    }

    /// Construct a new texty tree.
    pub fn new_text(node: Node, text: String) -> Tree<Node> {
        let mut tree = Text(node, text);
        tree.update();
        tree
    }

    
    // Alll Trees //

    pub fn cursor(&mut self) -> Cursor<Node> {
        Cursor{
            path: vec!(self)
        }
    }

    
    // Texty Trees //

    /// Is this tree texty?
    pub fn is_texty(&self) -> bool {
        match self {
            &Forest(_, _) => false,
            &Text(_, _)   => true
        }
    }

    /// The text of a texty tree. Panics on foresty trees.
    pub fn text(&self) -> &String {
        match self {
            &Forest(_, _)      => error_not_texty(),
            &Text(_, ref text) => text
        }
    }

    /// The text of a texty tree. Panics on foresty trees.
    pub fn text_mut(&mut self) -> &mut String {
        match self {
            &mut Forest(_, _)          => error_not_texty(),
            &mut Text(_, ref mut text) => text
        }
    }


    // Foresty Trees //

    /// Is this tree foresty?
    pub fn is_foresty(&self) -> bool {
        match self {
            &Forest(_, _) => true,
            &Text(_, _)   => false
        }
    }

    /// Return the children of a foresty tree. Panics on texty trees.
    pub fn forest(&self) -> &Vec<Tree<Node>> {
        match self {
            &Forest(_, ref forest) => forest,
            &Text(_, _)            => error_not_foresty()
        }
    }

    /// Return the children of a foresty tree. Panics on texty trees.
    pub fn forest_mut(&mut self) -> &mut Vec<Tree<Node>> {
        match self {
            &mut Forest(_, ref mut forest) => forest,
            &mut Text(_, _)                => error_not_foresty()
        }
    }

    
    // Private //

    fn node(&self) -> &Node {
        match self {
            &Forest(ref node, _) => node,
            &Text(ref node, _) => node
        }
    }

    /// *This must be called every time this tree or one of its
    /// children is modifed.*
    fn update(&mut self) {
        match self {
            &mut Forest(ref mut node, ref forest) => {
                let nodes = forest.iter().map(|tree| tree.node()).collect();
                node.update_forest(nodes);
            }
            &mut Text(ref mut node, ref text) => {
                node.update_text(text);
            }
        }
    }
}


// Cursor //

pub struct Cursor<'t, Node : TreeNode + 't> {
    path: Vec<&'t mut Tree<Node>> // Only the last cell is active.
}

impl<'t, Node : TreeNode + 't> Cursor<'t, Node> {
    pub fn goto_parent(&mut self) {
        if self.path.len() <= 1 {
            error_root_parent();
        }
        self.path.pop();
    }

    pub fn goto_child(&mut self, i: usize) {
        unsafe {
            let raw_ptr: *mut Tree<Node> = self.tree_mut();
            let mut_ref: &mut Tree<Node> = raw_ptr.as_mut().unwrap();
            self.path.push(&mut mut_ref[i]);
        }
    }

    pub fn root(&self) -> &Tree<Node> {
        self.path.first().expect("empty cursor")
    }

    pub fn tree(&self) -> &Tree<Node> {
        self.path.last().expect("empty cursor")
    }

    pub fn tree_mut(&mut self) -> &mut Tree<Node> {
        self.path.last_mut().expect("empty cursor")
    }
}


// Index //

impl<Node : TreeNode> Index<usize> for Tree<Node> {
    type Output = Tree<Node>;
    fn index(&self, index: usize) -> &Tree<Node> {
        match self {
            &Forest(_, ref forest) => {
                match forest.get(index) {
                    None       => error_invalid_index(),
                    Some(tree) => tree
                }
            }
            &Text(_, _) => error_invalid_index()
        }
    }
}

impl<Node : TreeNode> IndexMut<usize> for Tree<Node> {
    fn index_mut(&mut self, index: usize) -> &mut Tree<Node> {
        match self {
            &mut Forest(_, ref mut forest) => {
                match forest.get_mut(index) {
                    None       => error_invalid_index(),
                    Some(tree) => tree
                }
            }
            &mut Text(_, _) => error_invalid_index()
        }
    }
}


// Errors //

fn error_not_texty() -> ! {
    panic!("tree - expected a texty tree!");
}

fn error_not_foresty() -> ! {
    panic!("tree - expected a foresty tree!");
}

fn error_invalid_index() -> ! {
    panic!("tree - invalid child index!");
}

fn error_root_parent() -> ! {
    panic!("tree cursor - cannot take parent of root!");
}



/*
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
*/
