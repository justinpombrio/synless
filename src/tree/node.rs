use super::forest;
use super::text::Text;
use crate::language::{Arity, Construct, Language, LanguageSpec, NotationSetSpec, Storage};
use crate::style::{Condition, StyleLabel, ValidNotation};
use crate::util::{bug, SynlessBug};
use partial_pretty_printer as ppp;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

pub struct NodeForest {
    forest: forest::Forest<NodeData>,
    next_id: usize,
}

/// The data stored inside a document node.
#[derive(Debug)]
struct NodeData {
    id: NodeId,
    construct: Construct,
    /// Is Some iff the node is texty.
    text: Option<Text>,
}

/// A node in a document. You'll need a &Storage to do anything with it.
///
/// _Ownership model:_ There is one "primary" Node reference to each tree (anywhere in the tree).
/// When a tree would have two primary references, it's copied instead.  When a tree would have
/// zero primary references, it's deleted.  There can be "temporary" references as well, but they
/// never outlive the primary reference. Bookmarks are neither "primary" nor "temporary"
/// references: they may outlive the primary reference but need to be checked for validity before
/// being used, as the node they reference may have been deleted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Node(forest::NodeIndex);

// TODO: Put Storage arg always first or always last :-(

impl Storage {
    fn forest(&self) -> &forest::Forest<NodeData> {
        &self.node_forest.forest
    }

    fn forest_mut(&mut self) -> &mut forest::Forest<NodeData> {
        &mut self.node_forest.forest
    }
}

impl Node {
    /****************
     * Constructors *
     ****************/

    pub fn new_hole(s: &mut Storage, lang: Language) -> Node {
        Node::new(s, lang.hole_construct(s))
    }

    /// Creates a new root node.
    pub fn new(s: &mut Storage, construct: Construct) -> Node {
        let id = s.node_forest.next_id();
        match construct.arity(s) {
            Arity::Texty => Node(s.forest_mut().new_node(NodeData {
                id,
                construct,
                text: Some(Text::new()),
            })),
            Arity::Listy(_) => Node(s.forest_mut().new_node(NodeData {
                id,
                construct,
                text: None,
            })),
            Arity::Fixed(sorts) => {
                let parent = s.forest_mut().new_node(NodeData {
                    id,
                    construct,
                    text: None,
                });
                let num_children = sorts.len(s);
                let hole_construct = construct.language().hole_construct(s);
                for _ in 0..num_children {
                    let child_id = s.node_forest.next_id();
                    let child = s.forest_mut().new_node(NodeData {
                        id: child_id,
                        construct: hole_construct,
                        text: None,
                    });
                    s.forest_mut().insert_last_child(parent, child);
                }
                Node(parent)
            }
        }
    }

    pub fn with_text(s: &mut Storage, construct: Construct, text: String) -> Option<Node> {
        if let Arity::Texty = construct.arity(s) {
            let id = s.node_forest.next_id();
            let mut contents = Text::new();
            contents.set(text);
            Some(Node(s.forest_mut().new_node(NodeData {
                id,
                construct,
                text: Some(contents),
            })))
        } else {
            None
        }
    }

    pub fn with_children(
        s: &mut Storage,
        construct: Construct,
        children: impl IntoIterator<Item = Node>,
    ) -> Option<Node> {
        let children = children.into_iter().collect::<Vec<_>>();
        let allowed = match construct.arity(s) {
            Arity::Texty => false,
            Arity::Listy(sort) => children
                .iter()
                .all(|child| sort.accepts(s, child.construct(s))),
            Arity::Fixed(sorts) => {
                if sorts.len(s) != children.len() {
                    false
                } else {
                    children
                        .iter()
                        .enumerate()
                        .all(|(i, child)| sorts.get(s, i).bug().accepts(s, child.construct(s)))
                }
            }
        };
        if allowed {
            let id = s.node_forest.next_id();
            let parent = s.forest_mut().new_node(NodeData {
                id,
                construct,
                text: None,
            });
            for child in children {
                s.forest_mut().insert_last_child(parent, child.0);
            }
            Some(Node(parent))
        } else {
            None
        }
    }

    /*************
     * Node Data *
     *************/

    pub fn id(self, s: &Storage) -> NodeId {
        s.forest().data(self.0).id
    }

    pub fn language(self, s: &Storage) -> Language {
        s.forest().data(self.0).construct.language()
    }

    pub fn construct(self, s: &Storage) -> Construct {
        s.forest().data(self.0).construct
    }

    pub fn arity(self, s: &Storage) -> Arity {
        s.forest().data(self.0).construct.arity(s)
    }

    /// ("ws" means "whitespace")
    pub fn is_comment_or_ws(self, s: &Storage) -> bool {
        s.forest().data(self.0).construct.is_comment_or_ws(s)
    }

    pub fn notation(self, s: &Storage) -> &ValidNotation {
        s.forest().data(self.0).construct.notation(s)
    }

    pub fn is_texty(self, s: &Storage) -> bool {
        s.forest().data(self.0).text.is_some()
    }

    /// Borrow the text of a texty node. `None` if it's not texty.
    pub fn text(self, s: &Storage) -> Option<&Text> {
        s.forest().data(self.0).text.as_ref()
    }

    /// Mutably borrow the text of a texty node. `None` if it's not texty.
    pub fn text_mut(self, s: &mut Storage) -> Option<&mut Text> {
        s.forest_mut().data_mut(self.0).text.as_mut()
    }

    /*************
     * Relatives *
     *************/

    /// Returns whether this is the root of a tree. Equivalent to `self.parent(s).is_none()`.
    pub fn is_at_root(self, s: &Storage) -> bool {
        s.forest().parent(self.0).is_none()
    }

    /// Determine the number of siblings that this node has, including itself.
    pub fn num_siblings(&self, s: &Storage) -> usize {
        if let Some(parent) = s.forest().parent(self.0) {
            s.forest().num_children(parent)
        } else {
            1
        }
    }

    /// Determine this node's index among its siblings. Returns `0` when at the root.
    pub fn sibling_index(self, s: &Storage) -> usize {
        s.forest().sibling_index(self.0)
    }

    /// Return the number of children this node has. For a Fixed node, this is
    /// its arity. For a Listy node, this is its current number of children.
    /// For text, this is None. Requires iterating over all the children.
    pub fn num_children(self, s: &Storage) -> Option<usize> {
        if self.is_texty(s) {
            None
        } else {
            Some(s.forest().num_children(self.0))
        }
    }

    /**************
     * Navigation *
     **************/

    pub fn parent(self, s: &Storage) -> Option<Node> {
        s.forest().parent(self.0).map(Node)
    }

    pub fn first_child(self, s: &Storage) -> Option<Node> {
        s.forest().first_child(self.0).map(Node)
    }

    pub fn last_child(self, s: &Storage) -> Option<Node> {
        s.forest()
            .first_child(self.0)
            .map(|n| Node(s.forest().last_sibling(n)))
    }

    pub fn nth_child(self, s: &Storage, n: usize) -> Option<Node> {
        s.forest().nth_child(self.0, n).map(Node)
    }

    pub fn next_sibling(self, s: &Storage) -> Option<Node> {
        s.forest().next(self.0).map(Node)
    }

    pub fn prev_sibling(self, s: &Storage) -> Option<Node> {
        s.forest().prev(self.0).map(Node)
    }

    pub fn first_sibling(self, s: &Storage) -> Node {
        Node(s.forest().first_sibling(self.0))
    }

    pub fn last_sibling(self, s: &Storage) -> Node {
        Node(s.forest().last_sibling(self.0))
    }

    pub fn root(self, s: &Storage) -> Node {
        Node(s.forest().root(self.0))
    }

    /// Check whether this node has been deleted.
    pub fn is_valid(self, s: &Storage) -> bool {
        s.forest().is_valid(self.0)
    }

    /**************
     * Acceptance *
     **************/

    /// Check if `other` is allowed where `self` currently is, according to our parent's arity.
    fn accepts_replacement(self, s: &Storage, other: Node) -> bool {
        if let Some(parent) = s.forest().parent(self.0) {
            let sort = match Node(parent).arity(s) {
                Arity::Fixed(sorts) => sorts.get(s, self.sibling_index(s)).bug(),
                Arity::Listy(sort) => sort,
                Arity::Texty => bug!("Texty parent!"),
            };
            let other_construct = s.forest().data(other.0).construct;
            sort.accepts(s, other_construct)
        } else {
            true
        }
    }

    fn is_listy_and_accepts_child(self, s: &Storage, other: Node) -> bool {
        let other_construct = s.forest().data(other.0).construct;
        match self.arity(s) {
            Arity::Fixed(_) => false,
            Arity::Listy(sort) => sort.accepts(s, other_construct),
            Arity::Texty => false,
        }
    }

    /************
     * Mutation *
     ************/

    /// Attempts to swap `self` and `other`, returning true if successful.
    /// Returns false and does nothing if either:
    ///
    /// - One node is an ancestor of another (so they would mangle the trees if swapped).
    /// - One of the nodes is incompatible with the arity of its new parent.
    #[must_use]
    pub fn swap(self, s: &mut Storage, other: Node) -> bool {
        if self.accepts_replacement(s, other) && other.accepts_replacement(s, self) {
            s.forest_mut().swap(self.0, other.0)
        } else {
            false
        }
    }

    /// Attempts to insert `new_sibling` to the left of `self`.
    /// Returns false and does nothing if either:
    ///
    /// - The `new_sibling` is incompatible with the arity of the parent.
    /// - The `new_sibling` is not a root.
    #[must_use]
    pub fn insert_before(self, s: &mut Storage, new_sibling: Node) -> bool {
        if let Some(parent) = self.parent(s) {
            if parent.is_listy_and_accepts_child(s, new_sibling) {
                s.forest_mut().insert_before(self.0, new_sibling.0)
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Attempts to insert `new_sibling` to the right of `self`.
    /// Returns false and does nothing if either:
    ///
    /// - The `new_sibling` is incompatible with the arity of the parent.
    /// - The `new_sibling` is not a root.
    #[must_use]
    pub fn insert_after(self, s: &mut Storage, new_sibling: Node) -> bool {
        if let Some(parent) = self.parent(s) {
            if parent.is_listy_and_accepts_child(s, new_sibling) {
                s.forest_mut().insert_after(self.0, new_sibling.0)
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Attempts to insert `new_child` as the last child of `self`.
    /// Returns false and does nothing if any of:
    ///
    /// - `self` is not listy.
    /// - The `new_child` is incompatible with the arity of `self`.
    /// - The `new_child` is not a root.
    /// - The `new_child` is the root of `self`.
    #[must_use]
    pub fn insert_last_child(self, s: &mut Storage, new_child: Node) -> bool {
        if self.is_listy_and_accepts_child(s, new_child) {
            s.forest_mut().insert_last_child(self.0, new_child.0)
        } else {
            false
        }
    }

    /// Attempts to remove `self` from its listy parent, making it a root.
    /// Returns false and does nothing if either:
    ///
    /// - The parent is not listy.
    /// - `self` is a root.
    #[must_use]
    pub fn detach(self, s: &mut Storage) -> bool {
        if let Some(parent) = self.parent(s) {
            match parent.arity(s) {
                Arity::Fixed(_) => false,
                Arity::Texty => false,
                Arity::Listy(_) => {
                    s.forest_mut().detach(self.0);
                    true
                }
            }
        } else {
            false
        }
    }

    /*************
     * Debugging *
     *************/

    pub fn display(self, s: &Storage) -> impl fmt::Display + '_ {
        NodeDisplay {
            storage: s,
            node: self,
        }
    }
}

pub struct NodeDisplay<'s> {
    storage: &'s Storage,
    node: Node,
}

impl<'s> fmt::Display for NodeDisplay<'s> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(")?;
        write!(
            f,
            "{}",
            self.node.construct(self.storage).name(self.storage)
        )?;
        if let Some(mut child) = self.node.first_child(self.storage) {
            write!(f, " {}", child.display(self.storage))?;
            while let Some(next_child) = child.next_sibling(self.storage) {
                child = next_child;
                write!(f, " {}", child.display(self.storage))?;
            }
        } else if let Some(text) = self.node.text(self.storage) {
            write!(f, " \"{}\"", text.as_str())?;
        }
        write!(f, ")")
    }
}

impl NodeForest {
    pub fn new() -> NodeForest {
        // Must never use this node!
        let invalid_dummy_node = {
            let mut text = Text::new();
            text.set("Dummy node that must never be seen!".to_owned());
            NodeData {
                id: NodeId(666),
                construct: Construct::invalid_dummy(),
                text: Some(text),
            }
        };

        NodeForest {
            forest: forest::Forest::new(invalid_dummy_node),
            next_id: 0,
        }
    }

    pub fn next_id(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        NodeId(id)
    }
}
