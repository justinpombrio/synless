use super::node::Node;
use crate::language::{Arity, Construct, Storage};
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

#[derive(Debug, Clone, Copy)]
enum LocationInner {
    /// The usize is an index between chars (so it can be equal to the len)
    InText(Node, usize),
    AtNode(Node),
    /// Before the first child of `Node`; requires that `Node` be listy.
    BelowNode(Node),
}
use LocationInner::{AtNode, BelowNode, InText};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    Tree,
    Text,
}

impl Location {
    /****************
     * Constructors *
     ****************/

    pub fn at(_s: &Storage, node: Node) -> Location {
        Location(AtNode(node))
    }

    /// Returns the left-most location in the node's child sequence.
    /// (Returns `None` for a texty node or a fixed node with no children.)
    pub fn before_children(s: &Storage, node: Node) -> Option<Location> {
        match node.arity(s) {
            Arity::Texty => None,
            Arity::Fixed(_) => node.first_child(s).map(|child| Location(AtNode(child))),
            Arity::Listy(_) => Some(Location(BelowNode(node))),
        }
    }

    /// Returns the location at the node's first child, if any.
    pub fn at_first_child(s: &Storage, node: Node) -> Option<Location> {
        node.first_child(s).map(|child| Location(AtNode(child)))
    }

    /// Returns the right-most location in the node's child sequence.
    /// (Returns `None` for a texty node or a fixed node with no children.)
    pub fn after_children(s: &Storage, node: Node) -> Option<Location> {
        if !node.can_have_children(s) {
            return None;
        }
        if let Some(last_child) = node.last_child(s) {
            Some(Location(AtNode(last_child)))
        } else {
            Some(Location(BelowNode(node)))
        }
    }

    /// If the node is texty, returns the location at the start of its text, otherwise returns `None`.
    pub fn start_of_text(s: &Storage, node: Node) -> Option<Location> {
        if node.is_texty(s) {
            Some(Location(InText(node, 0)))
        } else {
            None
        }
    }

    /// If the node is texty, returns the location at the end of its text, otherwise returns `None`.
    pub fn end_of_text(s: &Storage, node: Node) -> Option<Location> {
        let text_len = node.text(s)?.num_chars();
        Some(Location(InText(node, text_len)))
    }

    /// Where to move the cursor after inserting this node.
    pub fn first_insert_loc(s: &Storage, node: Node) -> Location {
        match node.arity(s) {
            Arity::Texty => Location::end_of_text(s, node).bug(),
            Arity::Listy(_) => Location::before_children(s, node).bug(),
            Arity::Fixed(_) => {
                if let Some(child) = node.first_child(s) {
                    Location::first_insert_loc(s, child)
                } else {
                    Location(AtNode(node))
                }
            }
        }
    }

    /*************
     * Accessors *
     *************/

    pub fn mode(self) -> Mode {
        match self.0 {
            InText(_, _) => Mode::Text,
            _ => Mode::Tree,
        }
    }

    pub fn text_pos(self) -> Option<(Node, usize)> {
        if let InText(node, char_pos) = self.0 {
            Some((node, char_pos))
        } else {
            None
        }
    }

    pub fn text_pos_mut(&mut self) -> Option<(Node, &mut usize)> {
        if let InText(node, char_pos) = &mut self.0 {
            Some((*node, char_pos))
        } else {
            None
        }
    }

    /// Find a path from the root node to a node near this location, together with
    /// a `FocusTarget` specifying where this location is relative to that node.
    pub fn path_from_root(self, s: &Storage) -> (Vec<usize>, ppp::FocusTarget) {
        let mut path_to_root = Vec::new();
        let (mut node, target) = match self.0 {
            BelowNode(node) => {
                if let Some(first_child) = node.first_child(s) {
                    (first_child, ppp::FocusTarget::Start)
                } else {
                    // NOTE: This relies on the node's notation containing a `Notation::FocusMark`.
                    (node, ppp::FocusTarget::Mark)
                }
            }
            AtNode(node) => (node, ppp::FocusTarget::Start),
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

    /// Return the _one-indexed_ sibling index and number of siblings of this location. Being in
    /// text is treated the same as being at the texty node.
    pub fn sibling_index_info(self, s: &Storage) -> (usize, usize) {
        match self.0 {
            InText(node, _) | AtNode(node) => (node.sibling_index(s) + 1, node.num_siblings(s)),
            BelowNode(node) => (0, node.num_children(s).bug()),
        }
    }

    /**************
     * Navigation *
     **************/

    pub fn prev_cousin(self, s: &Storage) -> Option<Location> {
        if matches!(self.0, InText(_, _)) {
            return None;
        }
        if let Some(sibling) = self.prev_sibling(s) {
            return Some(sibling);
        }
        Location::after_children(s, self.parent_node(s)?.prev_cousin(s)?)
    }

    pub fn next_cousin(self, s: &Storage) -> Option<Location> {
        if matches!(self.0, InText(_, _)) {
            return None;
        }
        if let Some(sibling) = self.next_sibling(s) {
            return Some(sibling);
        }
        Location::before_children(s, self.parent_node(s)?.next_cousin(s)?)
    }

    pub fn prev_sibling(self, s: &Storage) -> Option<Location> {
        let node = match self.0 {
            AtNode(node) => node,
            InText(_, _) | BelowNode(_) => return None,
        };
        if let Some(sibling) = node.prev_sibling(s) {
            return Some(Location(AtNode(sibling)));
        }
        let parent = node.parent(s)?;
        if matches!(parent.arity(s), Arity::Listy(_)) {
            Some(Location(BelowNode(parent)))
        } else {
            None
        }
    }

    pub fn next_sibling(self, s: &Storage) -> Option<Location> {
        match self.0 {
            AtNode(node) => Some(Location(AtNode(node.next_sibling(s)?))),
            BelowNode(parent) => parent.first_child(s).map(|child| Location(AtNode(child))),
            InText(_, _) => None,
        }
    }

    /// Get the left-most location among this node's siblings.
    pub fn first_sibling(self, s: &Storage) -> Option<Location> {
        Location::before_children(s, self.parent_node(s)?)
    }

    /// Get the right-most location among this node's siblings.
    pub fn last_sibling(self, s: &Storage) -> Option<Location> {
        Location::after_children(s, self.parent_node(s)?)
    }

    /// Get the location at the next leaf node.
    pub fn next_leaf(self, s: &Storage) -> Option<Location> {
        let mut node = match self.0 {
            InText(_, _) => return None,
            AtNode(node) | BelowNode(node) => {
                if let Some(child) = node.first_child(s) {
                    return Some(Location(AtNode(child.first_leaf(s))));
                } else {
                    node
                }
            }
        };
        while node.next_sibling(s).is_none() {
            node = node.parent(s)?;
        }
        Some(Location(AtNode(node.next_sibling(s).bug().first_leaf(s))))
    }

    /// Get the location at the previous leaf node.
    pub fn prev_leaf(self, s: &Storage) -> Option<Location> {
        let mut node = match self.0 {
            InText(_, _) => return None,
            AtNode(node) | BelowNode(node) => node,
        };
        while node.prev_sibling(s).is_none() {
            node = node.parent(s)?;
        }
        Some(Location(AtNode(node.prev_sibling(s).bug().last_leaf(s))))
    }

    /// Get the location at the next texty node.
    pub fn next_text(mut self, s: &Storage) -> Option<Location> {
        loop {
            self = self.next_leaf(s)?;
            if self.node(s).bug().is_texty(s) {
                return Some(self);
            }
        }
    }

    /// Get the location at the previous texty node.
    pub fn prev_text(mut self, s: &Storage) -> Option<Location> {
        loop {
            self = self.prev_leaf(s)?;
            if self.node(s).bug().is_texty(s) {
                return Some(self);
            }
        }
    }

    /// Get the location of the next node of the given construct.
    pub fn next_construct(self, construct: Construct, s: &Storage) -> Option<Location> {
        let mut node = match self.0 {
            AtNode(node) | InText(node, _) | BelowNode(node) => node,
        };
        while let Some(next_node) = node.next_inorder(s) {
            if next_node.construct(s) == construct {
                return Some(Location::at(s, next_node));
            }
            node = next_node;
        }
        None
    }

    /// Get the location of the previous node of the given construct.
    pub fn prev_construct(self, construct: Construct, s: &Storage) -> Option<Location> {
        let mut node = match self.0 {
            BelowNode(node) if node.construct(s) == construct => {
                return Some(Location::at(s, node))
            }
            AtNode(node) | InText(node, _) | BelowNode(node) => node,
        };
        while let Some(prev_node) = node.prev_inorder(s) {
            if prev_node.construct(s) == construct {
                return Some(Location::at(s, prev_node));
            }
            node = prev_node;
        }
        None
    }

    /// Get the location at this node's parent.
    pub fn parent(self, s: &Storage) -> Option<Location> {
        let parent_node = self.parent_node(s)?;
        if parent_node.is_root(s) {
            return None;
        }
        Some(Location(AtNode(parent_node)))
    }

    /// If the location is in text, returns the location after that text node.
    pub fn exit_text(self) -> Option<Location> {
        if let InText(node, _) = self.0 {
            Some(Location(AtNode(node)))
        } else {
            None
        }
    }

    /*****************
     * Getting Nodes *
     *****************/

    pub fn node(self, _s: &Storage) -> Option<Node> {
        match self.0 {
            AtNode(node) => Some(node),
            InText(_, _) | BelowNode(_) => None,
        }
    }

    pub fn parent_node(self, s: &Storage) -> Option<Node> {
        match self.0 {
            InText(_, _) => None,
            AtNode(node) => node.parent(s),
            BelowNode(node) => Some(node),
        }
    }

    pub fn root_node(self, s: &Storage) -> Node {
        self.0.reference_node().root(s)
    }

    /************
     * Mutation *
     ************/

    /// In a listy sequence, inserts `new_node` to the right of this location and returns
    /// `Ok(None)`. In a fixed sequence, replaces the node at this location with `new_node` and
    /// returns `Ok(Some(old_node))`. Either way, moves `self` to the new node.
    ///
    /// If we cannot insert, returns `Err(())` and does not modify `self`. This can happen for any
    /// of the following reasons:
    ///
    /// - This location is in text.
    /// - This location is at the root.
    /// - The new node does not match the required sort.
    #[allow(clippy::result_unit_err)]
    pub fn insert(&mut self, s: &mut Storage, new_node: Node) -> Result<Option<Node>, ()> {
        let parent = self.parent_node(s).ok_or(())?;

        match parent.arity(s) {
            Arity::Texty => bug!("insert: texty parent"),
            Arity::Fixed(_) => {
                let old_node = self.node(s).ok_or(())?;
                if new_node.swap(s, old_node) {
                    *self = Location(AtNode(new_node));
                    Ok(Some(old_node))
                } else {
                    Err(())
                }
            }
            Arity::Listy(_) => {
                let success = match self.0 {
                    InText(_, _) => bug!("insert: bug in textiness check"),
                    AtNode(node) => node.insert_after(s, new_node),
                    BelowNode(_) => parent.insert_first_child(s, new_node),
                };
                if success {
                    *self = Location(AtNode(new_node));
                    Ok(None)
                } else {
                    Err(())
                }
            }
        }
    }

    /// Deletes the node at the cursor. If in a listy sequence, attempts to move the cursor left or
    /// right. Returns the node that was deleted and the location where the undo command should be
    /// executed from.
    #[must_use]
    pub fn delete(&mut self, s: &mut Storage, move_left: bool) -> Option<(Node, Location)> {
        let node = self.node(s)?;
        let parent = node.parent(s)?;
        match parent.arity(s) {
            Arity::Texty => bug!("texty parent"),
            Arity::Fixed(_) => {
                let hole = Node::new_hole(s, node.language(s));
                if node.swap(s, hole) {
                    *self = Location(AtNode(hole));
                    Some((node, *self))
                } else {
                    None
                }
            }
            Arity::Listy(_) => {
                let prev_loc = self.prev_sibling(s).bug();
                let opt_next_loc = self.next_sibling(s);
                if node.detach(s) {
                    *self = if move_left {
                        prev_loc
                    } else {
                        opt_next_loc.unwrap_or(prev_loc)
                    };
                    Some((node, prev_loc))
                } else {
                    None
                }
            }
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
        match self {
            InText(node, i) => {
                let text_len = node.text(s).bug().num_chars();
                InText(node, i.min(text_len))
            }
            AtNode(_) | BelowNode(_) => self,
        }
    }

    /// Get the node this location is defined relative to. May be at or above this
    /// location!
    fn reference_node(self) -> Node {
        match self {
            InText(node, _) | AtNode(node) | BelowNode(node) => node,
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
