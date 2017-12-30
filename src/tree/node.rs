use syntax::{Construct, BoundSet, HOLE};
use syntax::Arity::*;

/// A node of a tree.
#[derive(Debug)]
pub struct Node<'l> {
    /// The syntactic construct that this is an instance of.
    pub(in tree) construct: &'l Construct,
    /// The last child to have been visited.
    pub(in tree) breadcrumb: usize,
    /// The bounds in which this tree fits, remembered for efficiency.
    pub(in tree) bounds: BoundSet
}
// (tested in tree/bounds.rs)

impl<'l> Node<'l> {

    // Public //

    pub fn is_extendable(&self) -> bool {
        self.construct.is_extendable()
    }

    pub fn construct(&self) -> &'l Construct {
        &self.construct
    }

    // Constructors //

    pub(in tree) fn new_forest(construct: &'l Construct,
                               children: Vec<&Node>) -> Node<'l>
    {
        let mut node = Node::new(construct);
        node.update_forest(children);
        node
    }

    pub(in tree) fn new_text(construct: &'l Construct,
                             text: &str) -> Node<'l>
    {
        let mut node = Node::new(construct);
        node.update_text(text);
        node.breadcrumb = text.chars().count();
        node
    }
 
    pub(in tree) fn new_hole() -> Node<'l> {
        let mut node = Node::new(&HOLE);
        node.update_forest(vec!());
        node
    }

    // MUST be initialized with `update` before being used.
    fn new(construct: &'l Construct) -> Node<'l> {
        Node{
            construct:  construct,
            breadcrumb: 0,
            bounds:     BoundSet::new()
        }
    }

    // Updating //

    pub(in tree) fn update_forest(&mut self, children: Vec<&Node>) {
        match self.construct.arity {
            ForestArity{..} => (),
            _             => panic!("update_forest: called on a text node")
        }
        let child_bounds: Vec<&BoundSet> = children.into_iter().map(|n| &n.bounds).collect();
        let arity = self.construct.arity.arity();
        let bounds = self.construct.syntax.bound(arity, child_bounds, false);
        
        self.bounds = bounds;
    }

    pub(in tree) fn update_text(&mut self, text: &str) {
        match self.construct.arity {
            TextArity => (),
            _         => panic!("update_text: called on a tree node")
        }
        let text_bound = BoundSet::literal(text);
        let arity = self.construct.arity.arity();
        let bounds = self.construct.syntax.bound(
            arity, vec!(&text_bound), text.is_empty());
        
        self.bounds = bounds;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_node_construction() {
        use syntax::Bound;
        use language::make_example_tree;
        use language::Language;

        let lang = Language::example_language();
        let tree = make_example_tree(&lang, false);
        assert_eq!(tree.node.bounds.bound, vec!(
            Bound{ width: 42, height: 0, indent: 42 },
            Bound{ width: 33, height: 1, indent: 33 },
            Bound{ width: 21, height: 2, indent: 1 },
            Bound{ width: 20, height: 3, indent: 1 },
            Bound{ width: 15, height: 4, indent: 1 },
            Bound{ width: 12, height: 5, indent: 1 }));
    }
}    
