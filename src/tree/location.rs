use super::node::Node;
use crate::language::{Arity, Storage};
use crate::util::{bug, SynlessBug};
use partial_pretty_printer as ppp;
use std::fmt;
use std::str::FromStr;

// The node in this LocationInner may not be valid (may have been deleted!)
#[derive(Debug, Clone, Copy)]
pub struct Bookmark(LocationInner);

/// A location between nodes, or within text, where a cursor could go.
#[derive(Debug, Clone, Copy)]
pub struct Location(LocationInner);

/// This data type admits multiple representations of the same location. For example, a location
/// between nodes X and Y could be represented as either `AfterNode(X)` or `BeforeNode(Y)`. We
/// therefore keep locations in a _normal form_. The exception is Bookmarks, which might not be in
/// normal form (or even valid!) and must be checked and normalized before use. The rule for the
/// normal form is that `AfterNode` is used if possible, falling back to `BeforeNode` and then
/// `BelowNode`. This implies that `BelowNode` is only used in empty sequences.
#[derive(Debug, Clone, Copy)]
enum LocationInner {
    /// The usize is an index between chars (so it can be equal to the len)
    InText(Node, usize),
    AfterNode(Node),
    BeforeNode(Node),
    BelowNode(Node),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    Tree,
    Text,
}

impl Location {
    /****************
     * Constructors *
     ****************/

    pub fn before(s: &Storage, node: Node) -> Location {
        Location(LocationInner::BeforeNode(node).normalize(s))
    }

    pub fn after(_s: &Storage, node: Node) -> Location {
        // already normal form
        Location(LocationInner::AfterNode(node))
    }

    /// Returns the location at the beginning of the child sequence of the given node.
    /// (Returns `None` for a texty node, or a fixed node with no children.)
    pub fn before_children(s: &Storage, node: Node) -> Option<Location> {
        if !node.can_have_children(s) {
            return None;
        }
        if let Some(first_child) = node.first_child(s) {
            Some(Location::before(s, first_child))
        } else {
            Some(Location(LocationInner::BelowNode(node)))
        }
    }

    /// Returns the location at the end of the child sequence of the given node.
    /// (Returns `None` for a texty node, or a fixed node with no children.)
    pub fn after_children(s: &Storage, node: Node) -> Option<Location> {
        if !node.can_have_children(s) {
            return None;
        }
        if let Some(last_child) = node.last_child(s) {
            Some(Location::after(s, last_child))
        } else {
            Some(Location(LocationInner::BelowNode(node)))
        }
    }

    /// If the node is texty, returns the location at the start of its text, otherwise returns `None`.
    pub fn start_of_text(s: &Storage, node: Node) -> Option<Location> {
        if node.is_texty(s) {
            Some(Location(LocationInner::InText(node, 0)))
        } else {
            None
        }
    }

    /// If the node is texty, returns the location at the end of its text, otherwise returns `None`.
    pub fn end_of_text(s: &Storage, node: Node) -> Option<Location> {
        let text_len = node.text(s)?.num_chars();
        Some(Location(LocationInner::InText(node, text_len)))
    }

    /*************
     * Accessors *
     *************/

    pub fn mode(self) -> Mode {
        match self.0 {
            LocationInner::InText(_, _) => Mode::Text,
            _ => Mode::Tree,
        }
    }

    pub fn text_pos(self) -> Option<(Node, usize)> {
        if let LocationInner::InText(node, char_pos) = self.0 {
            Some((node, char_pos))
        } else {
            None
        }
    }

    pub fn text_pos_mut(&mut self) -> Option<(Node, &mut usize)> {
        if let LocationInner::InText(node, char_pos) = &mut self.0 {
            Some((*node, char_pos))
        } else {
            None
        }
    }

    /// Find a path from the root node to a node near this location, together with
    /// a `FocusTarget` specifying where this location is relative to that node.
    pub fn path_from_root(self, s: &Storage) -> (Vec<usize>, ppp::FocusTarget) {
        use LocationInner::*;

        let mut path_to_root = Vec::new();
        let (mut node, target) = match self.0 {
            BeforeNode(node) => (node, ppp::FocusTarget::Start),
            AfterNode(node) => (node, ppp::FocusTarget::End),
            // NOTE: This relies on the node's notation containing a `Notation::FocusMark`.
            BelowNode(node) => (node, ppp::FocusTarget::Mark),
            InText(node, char_pos) => (node, ppp::FocusTarget::Text(char_pos)),
        };
        while let Some(parent) = node.parent(s) {
            path_to_root.push(node.sibling_index(s));
            node = parent;
        }
        let path_from_root = {
            let mut path = path_to_root;
            path.reverse();
            path
        };
        (path_from_root, target)
    }

    /**************
     * Navigation *
     **************/

    pub fn prev_cousin(self, s: &Storage) -> Option<Location> {
        use LocationInner::{AfterNode, BeforeNode, BelowNode, InText};

        match self.0 {
            InText(_, _) => return None,
            AfterNode(node) => return Some(Location::before(s, node)),
            BeforeNode(_) | BelowNode(_) => (),
        }

        Location::after_children(s, self.parent_node(s)?.prev_cousin(s)?)
    }

    pub fn next_cousin(self, s: &Storage) -> Option<Location> {
        use LocationInner::{AfterNode, BeforeNode, BelowNode, InText};

        match self.0 {
            InText(_, _) => return None,
            BeforeNode(node) => return Some(Location::after(s, node)),
            AfterNode(node) => {
                if let Some(sibling) = node.next_sibling(s) {
                    return Some(Location::after(s, sibling));
                }
            }
            BelowNode(_) => (),
        }

        Location::before_children(s, self.parent_node(s)?.next_cousin(s)?)
    }

    pub fn prev_sibling(self, s: &Storage) -> Option<Location> {
        use LocationInner::{AfterNode, BeforeNode, BelowNode, InText};

        match self.0 {
            AfterNode(node) => Some(Location::before(s, node)),
            InText(_, _) | BeforeNode(_) | BelowNode(_) => None,
        }
    }

    pub fn next_sibling(self, s: &Storage) -> Option<Location> {
        use LocationInner::{AfterNode, BeforeNode, BelowNode, InText};

        match self.0 {
            AfterNode(node) => Some(Location::after(s, node.next_sibling(s)?)),
            BeforeNode(node) => Some(Location::after(s, node)),
            InText(_, _) | BelowNode(_) => None,
        }
    }

    pub fn first(self, s: &Storage) -> Option<Location> {
        use LocationInner::{AfterNode, BeforeNode, BelowNode, InText};

        match self.0 {
            InText(_, _) => None,
            AfterNode(node) => Some(Location::before(s, node.first_sibling(s))),
            BeforeNode(_) | BelowNode(_) => Some(self),
        }
    }

    pub fn last(self, s: &Storage) -> Option<Location> {
        use LocationInner::{AfterNode, BeforeNode, BelowNode, InText};

        match self.0 {
            InText(_, _) => None,
            BeforeNode(node) | AfterNode(node) => Some(Location::after(s, node.last_sibling(s))),
            BelowNode(_) => Some(self),
        }
    }

    pub fn before_parent(self, s: &Storage) -> Option<Location> {
        Some(Location::before(s, self.parent_node(s)?))
    }

    pub fn after_parent(self, s: &Storage) -> Option<Location> {
        Some(Location::after(s, self.parent_node(s)?))
    }

    /// Returns the next location in an inorder tree traversal.
    pub fn inorder_next(self, s: &Storage) -> Option<Location> {
        if let Some(right_node) = self.right_node(s) {
            if let Some(loc) = Location::before_children(s, right_node) {
                Some(loc)
            } else {
                Some(Location::after(s, right_node))
            }
        } else {
            self.after_parent(s)
        }
    }

    /// Returns the previous location in an inorder tree traversal.
    pub fn inorder_prev(self, s: &Storage) -> Option<Location> {
        if let Some(left_node) = self.left_node(s) {
            if let Some(loc) = Location::after_children(s, left_node) {
                Some(loc)
            } else {
                Some(Location::before(s, left_node))
            }
        } else {
            self.before_parent(s)
        }
    }

    /// If the location is in text, returns the location after that text node.
    pub fn exit_text(self) -> Option<Location> {
        if let LocationInner::InText(node, _) = self.0 {
            // already in normal form
            Some(Location(LocationInner::AfterNode(node)))
        } else {
            None
        }
    }

    /**********************
     * Navigation to Node *
     **********************/

    pub fn left_node(self, _s: &Storage) -> Option<Node> {
        use LocationInner::{AfterNode, BeforeNode, BelowNode, InText};

        match self.0 {
            AfterNode(node) => Some(node),
            InText(_, _) | BeforeNode(_) | BelowNode(_) => None,
        }
    }

    pub fn right_node(self, s: &Storage) -> Option<Node> {
        use LocationInner::{AfterNode, BeforeNode, BelowNode, InText};

        match self.0 {
            AfterNode(node) => node.next_sibling(s),
            BeforeNode(node) => Some(node),
            InText(_, _) | BelowNode(_) => None,
        }
    }

    pub fn parent_node(self, s: &Storage) -> Option<Node> {
        use LocationInner::{AfterNode, BeforeNode, BelowNode, InText};

        match self.0 {
            InText(_, _) => None,
            BeforeNode(node) | AfterNode(node) => node.parent(s),
            BelowNode(node) => Some(node),
        }
    }

    pub fn root_node(self, s: &Storage) -> Node {
        self.0.reference_node().root(s)
    }

    /************
     * Mutation *
     ************/

    /// In a listy sequence, inserts `new_node` at this location and returns `Ok(None)`. In a fixed
    /// sequence, replaces the node after this location with `new_node` and returns
    /// `Ok(Some(old_node))`. Either way, moves `self` to after the new node.
    ///
    /// If we cannot insert, returns `Err(())` and does not modify `self`. This can happen for any
    /// of the following reasons:
    ///
    /// - This location is in text.
    /// - This location is before or after a root node.
    /// - This location is after the last node in a fixed sequence.
    /// - The new node does not match the required sort.
    #[allow(clippy::result_unit_err)]
    pub fn insert(&mut self, s: &mut Storage, new_node: Node) -> Result<Option<Node>, ()> {
        use LocationInner::*;

        let parent = self.parent_node(s).ok_or(())?;

        match parent.arity(s) {
            Arity::Texty => bug!("insert: texty parent"),
            Arity::Fixed(_) => {
                let old_node = self.right_node(s).ok_or(())?;
                if new_node.swap(s, old_node) {
                    *self = Location::after(s, new_node);
                    Ok(Some(old_node))
                } else {
                    Err(())
                }
            }
            Arity::Listy(_) => {
                let success = match self.0 {
                    InText(_, _) => bug!("insert: bug in textiness check"),
                    AfterNode(left_node) => left_node.insert_after(s, new_node),
                    BeforeNode(right_node) => right_node.insert_before(s, new_node),
                    BelowNode(_) => parent.insert_last_child(s, new_node),
                };
                if success {
                    *self = Location::after(s, new_node);
                    Ok(None)
                } else {
                    Err(())
                }
            }
        }
    }

    /// In a listy sequence, delete the node before (after) the cursor. In a fixed sequence,
    /// replace the node before (after) the cursor with a hole, and move the cursor before (after)
    /// it.
    #[must_use]
    pub fn delete_neighbor(&mut self, s: &mut Storage, delete_before: bool) -> Option<Node> {
        let parent = self.parent_node(s)?;
        let node = if delete_before {
            self.left_node(s)?
        } else {
            self.right_node(s)?
        };

        match parent.arity(s) {
            Arity::Fixed(_) => {
                // NOTE: Think about what language the hole should be in once we start supporting
                // multi-language docs
                let hole = Node::new_hole(s, node.language(s));
                if node.swap(s, hole) {
                    *self = if delete_before {
                        Location::before(s, hole)
                    } else {
                        Location::after(s, hole)
                    };
                    Some(node)
                } else {
                    None
                }
            }
            Arity::Listy(_) => {
                let prev_node = node.prev_sibling(s);
                let next_node = node.next_sibling(s);
                if node.detach(s) {
                    *self = match (prev_node, next_node) {
                        (Some(prev), _) => Location(LocationInner::AfterNode(prev)),
                        (None, Some(next)) => Location(LocationInner::BeforeNode(next)),
                        (None, None) => Location(LocationInner::BelowNode(parent)),
                    };
                    Some(node)
                } else {
                    None
                }
            }
            Arity::Texty => bug!("delete_neighbor: texty parent"),
        }
    }

    /*************
     * Bookmarks *
     *************/

    /// Save a bookmark to return to later.
    pub fn bookmark(self) -> Bookmark {
        Bookmark(self.0)
    }

    /// Get the location of a previously saved bookmark, as long as that
    /// bookmark's node is present somewhere in this tree. This will
    /// work even if the Tree has been modified since the bookmark was
    /// created. However, it will return `None` if the bookmark's node
    /// has since been deleted, or if it is currently located in a
    /// different tree.
    pub fn validate_bookmark(self, s: &Storage, mark: Bookmark) -> Option<Location> {
        let mark_node = mark.0.reference_node();
        if mark_node.is_valid(s) && mark_node.root(s) == self.root_node(s) {
            Some(Location(mark.0.normalize(s)))
        } else {
            None
        }
    }
}

impl LocationInner {
    fn normalize(self, s: &Storage) -> LocationInner {
        use LocationInner::{AfterNode, BeforeNode, BelowNode, InText};

        match self {
            InText(node, i) => {
                let text_len = node.text(s).bug().num_chars();
                InText(node, i.min(text_len))
            }
            AfterNode(_) => self,
            BeforeNode(node) => node.prev_sibling(s).map(AfterNode).unwrap_or(self),
            BelowNode(parent) => parent.last_child(s).map(AfterNode).unwrap_or(self),
        }
    }

    /// Get the node this location is defined relative to. May be before, after, or above this
    /// location!
    fn reference_node(self) -> Node {
        use LocationInner::{AfterNode, BeforeNode, BelowNode, InText};

        match self {
            InText(node, _) | AfterNode(node) | BeforeNode(node) | BelowNode(node) => node,
        }
    }
}

#[derive(thiserror::Error, fmt::Debug)]
#[error("Failed to parse mode from string")]
pub struct ModeParseError;

impl FromStr for Mode {
    type Err = ModeParseError;

    fn from_str(s: &str) -> Result<Self, ModeParseError> {
        match s {
            "Tree" => Ok(Mode::Tree),
            "Text" => Ok(Mode::Text),
            _ => Err(ModeParseError),
        }
    }
}
