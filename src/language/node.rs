use super::forest::{Forest, NodeIndex};
use super::language_set::{Arity, Construct, Language, LanguageSet, LanguageSpec, NotationSetSpec};
use super::text::Text;
use super::LanguageError;
use crate::infra::{bug, SynlessBug};
use crate::style::{Condition, StyleLabel, ValidNotation};
use partial_pretty_printer as ppp;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

/// The data stored inside a document node.
struct NodeData {
    id: NodeId,
    construct: Construct,
    /// Is Some iff the node is texty.
    text: Option<Text>,
}

// TODO: Move DocStorage into language_set?
/// Stores all documents and languages.
pub struct DocStorage {
    language_set: LanguageSet,
    forest: Forest<NodeData>,
    next_id: NodeId,
}

/// A node in a document. You'll need a &DocStorage to do anything with it.
///
/// _Ownership model:_ There is one "primary" Node reference to each tree (anywhere in the tree).
/// When a tree would have two primary references, it's copied instead.  When a tree would have
/// zero primary references, it's deleted.  There can be "temporary" references as well, but they
/// never outlive the primary reference. Bookmarks are neither "primary" nor "temporary"
/// references: they may outlive the primary reference but need to be checked for validity before
/// being used, as the node they reference may have been deleted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Node(NodeIndex);

/// A long-lived reference to a node that might or might not still exist.
pub struct Bookmark(NodeIndex);

impl DocStorage {
    pub fn new() -> DocStorage {
        let invalid_dummy_node = {
            let mut text = Text::new();
            text.set("Dummy node that must never be seen!".to_owned());
            NodeData {
                id: NodeId(666),
                construct: Construct::invalid_dummy(),
                text: Some(text),
            }
        };

        DocStorage {
            language_set: LanguageSet::new(),
            forest: Forest::new(invalid_dummy_node),
            next_id: NodeId(0),
        }
    }

    pub fn add_language(&mut self, language_spec: LanguageSpec) -> Result<(), LanguageError> {
        self.language_set.add_language(language_spec)
    }

    pub fn add_notation_set(
        &mut self,
        language_name: &str,
        notation_set: NotationSetSpec,
    ) -> Result<(), LanguageError> {
        self.language_set
            .add_notation_set(language_name, notation_set)
    }

    fn next_id(&mut self) -> NodeId {
        let id = self.next_id.0;
        self.next_id.0 += 1;
        NodeId(id)
    }
}

impl Default for DocStorage {
    fn default() -> Self {
        DocStorage::new()
    }
}

impl Node {
    /****************
     * Constructors *
     ****************/

    pub fn new_hole(s: &mut DocStorage, lang: Language) -> Node {
        Node::new(s, lang.hole_construct(&s.language_set))
    }

    /// Creates a new root node.
    pub fn new(s: &mut DocStorage, construct: Construct) -> Node {
        let id = s.next_id();
        match construct.arity(&s.language_set) {
            Arity::Texty => Node(s.forest.new_node(NodeData {
                id,
                construct,
                text: Some(Text::new()),
            })),
            Arity::Listy(_) => Node(s.forest.new_node(NodeData {
                id,
                construct,
                text: None,
            })),
            Arity::Fixed(sorts) => {
                let parent = s.forest.new_node(NodeData {
                    id,
                    construct,
                    text: None,
                });
                let num_children = sorts.len(&s.language_set);
                let hole_construct = construct.language().hole_construct(&s.language_set);
                for _ in 0..num_children {
                    let child_id = s.next_id();
                    let child = s.forest.new_node(NodeData {
                        id: child_id,
                        construct: hole_construct,
                        text: None,
                    });
                    s.forest.insert_last_child(parent, child);
                }
                Node(parent)
            }
        }
    }

    /*************
     * Node Data *
     *************/

    pub fn id(self, s: &DocStorage) -> NodeId {
        s.forest.data(self.0).id
    }

    pub fn arity(self, s: &DocStorage) -> Arity {
        s.forest.data(self.0).construct.arity(&s.language_set)
    }

    /// ("ws" means "whitespace")
    pub fn is_comment_or_ws(self, s: &DocStorage) -> bool {
        s.forest
            .data(self.0)
            .construct
            .is_comment_or_ws(&s.language_set)
    }

    pub fn notation(self, s: &DocStorage) -> &ValidNotation {
        s.forest.data(self.0).construct.notation(&s.language_set)
    }

    /// Borrow the text of a texty node. `None` if it's not texty.
    pub fn text(self, s: &DocStorage) -> Option<&Text> {
        s.forest.data(self.0).text.as_ref()
    }

    /// Mutably borrow the text of a texty node. `None` if it's not texty.
    pub fn text_mut(self, s: &mut DocStorage) -> Option<&mut Text> {
        s.forest.data_mut(self.0).text.as_mut()
    }

    /*************
     * Relatives *
     *************/

    /// Returns whether this is the root of a tree. Equivalent to `self.parent(s).is_none()`.
    pub fn is_at_root(self, s: &DocStorage) -> bool {
        s.forest.parent(self.0).is_none()
    }

    /// Determine the number of siblings that this node has, including itself.
    pub fn num_siblings(&self, s: &DocStorage) -> usize {
        if let Some(parent) = s.forest.parent(self.0) {
            s.forest.num_children(parent)
        } else {
            1
        }
    }

    /// Determine this node's index among its siblings. Returns `0` when at the root.
    pub fn sibling_index(self, s: &DocStorage) -> usize {
        s.forest.sibling_index(self.0)
    }

    /// Return the number of children this node has. For a Fixed node, this is
    /// its arity. For a Listy node, this is its current number of children.
    /// For text, this is None.
    pub fn num_children(self, s: &DocStorage) -> Option<usize> {
        if s.forest.data(self.0).text.is_some() {
            None
        } else {
            Some(s.forest.num_children(self.0))
        }
    }

    /**************
     * Navigation *
     **************/

    pub fn parent(self, s: &DocStorage) -> Option<Node> {
        s.forest.parent(self.0).map(Node)
    }

    pub fn first_child(self, s: &DocStorage) -> Option<Node> {
        s.forest.first_child(self.0).map(Node)
    }

    pub fn last_child(self, s: &DocStorage) -> Option<Node> {
        s.forest
            .first_child(self.0)
            .map(|n| Node(s.forest.last_sibling(n)))
    }

    pub fn nth_child(self, s: &DocStorage, n: usize) -> Option<Node> {
        s.forest.nth_child(self.0, n).map(Node)
    }

    pub fn next_sibling(self, s: &DocStorage) -> Option<Node> {
        s.forest.next(self.0).map(Node)
    }

    pub fn prev_sibling(self, s: &DocStorage) -> Option<Node> {
        s.forest.prev(self.0).map(Node)
    }

    pub fn first_sibling(self, s: &DocStorage) -> Node {
        Node(s.forest.first_sibling(self.0))
    }

    pub fn last_sibling(self, s: &DocStorage) -> Node {
        Node(s.forest.last_sibling(self.0))
    }

    pub fn root(self, s: &DocStorage) -> Node {
        Node(s.forest.root(self.0))
    }

    /// Save a bookmark to return to later.
    pub fn bookmark(self) -> Bookmark {
        Bookmark(self.0)
    }

    /// Jump to a previously saved bookmark, as long as that
    /// bookmark's node is present somewhere in this tree. This will
    /// work even if the Tree has been modified since the bookmark was
    /// created. However, it will return `None` if the bookmark's node
    /// has since been deleted, or if it is currently located in a
    /// different tree.
    pub fn goto_bookmark(self, mark: Bookmark, s: &DocStorage) -> Option<Node> {
        if s.forest.is_valid(mark.0) && s.forest.root(self.0) == s.forest.root(mark.0) {
            Some(Node(mark.0))
        } else {
            None
        }
    }

    /**************
     * Acceptance *
     **************/

    /// Check if `other` is allowed where `self` currently is, according to our parent's arity.
    fn accepts_replacement(self, s: &DocStorage, other: Node) -> bool {
        if let Some(parent) = s.forest.parent(self.0) {
            let sort = match Node(parent).arity(s) {
                Arity::Fixed(sorts) => sorts.get(&s.language_set, self.sibling_index(s)).bug(),
                Arity::Listy(sort) => sort,
                Arity::Texty => bug!("Texty parent!"),
            };
            let other_construct = s.forest.data(other.0).construct;
            sort.accepts(&s.language_set, other_construct)
        } else {
            true
        }
    }

    fn is_listy_and_accepts_child(self, s: &DocStorage, other: Node) -> bool {
        let other_construct = s.forest.data(other.0).construct;
        match self.arity(s) {
            Arity::Fixed(_) => false,
            Arity::Listy(sort) => sort.accepts(&s.language_set, other_construct),
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
    pub fn swap(self, s: &mut DocStorage, other: Node) -> bool {
        if self.accepts_replacement(s, other) && other.accepts_replacement(s, self) {
            s.forest.swap(self.0, other.0)
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
    pub fn insert_before(self, s: &mut DocStorage, new_sibling: Node) -> bool {
        if let Some(parent) = self.parent(s) {
            if parent.is_listy_and_accepts_child(s, new_sibling) {
                s.forest.insert_before(self.0, new_sibling.0)
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
    pub fn insert_after(self, s: &mut DocStorage, new_sibling: Node) -> bool {
        if let Some(parent) = self.parent(s) {
            if parent.is_listy_and_accepts_child(s, new_sibling) {
                s.forest.insert_after(self.0, new_sibling.0)
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
    pub fn insert_last_child(self, s: &mut DocStorage, new_child: Node) -> bool {
        if self.is_listy_and_accepts_child(s, new_child) {
            s.forest.insert_last_child(self.0, new_child.0)
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
    pub fn detach(self, s: &mut DocStorage) -> bool {
        if let Some(parent) = self.parent(s) {
            match parent.arity(s) {
                Arity::Fixed(_) => false,
                Arity::Texty => false,
                Arity::Listy(_) => {
                    s.forest.detach(self.0);
                    true
                }
            }
        } else {
            false
        }
    }
}
