use super::node::Node;
use crate::language::Storage;

/// A location between nodes, or within text, where a cursor could go.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Location {
    InText {
        node: Node,
        /// Between characters, so it can be equal to the len
        char_pos: usize,
    },
    InTree {
        parent: Option<Node>,
        left: Option<Node>,
        right: Option<Node>,
    },
}

impl Location {
    pub fn left(self) -> Option<Node> {
        match self {
            Location::InText { .. } => None,
            Location::InTree { left, .. } => left,
        }
    }

    pub fn right(self) -> Option<Node> {
        match self {
            Location::InText { .. } => None,
            Location::InTree { left, .. } => left,
        }
    }

    pub fn parent(self) -> Option<Node> {
        match self {
            Location::InText { .. } => None,
            Location::InTree { parent, .. } => parent,
        }
    }
}

impl Node {
    pub fn loc_before(self, s: &Storage) -> Location {
        Location::InTree {
            parent: self.parent(s),
            left: self.prev_sibling(s),
            right: Some(self),
        }
    }

    pub fn loc_after(self, s: &Storage) -> Location {
        Location::InTree {
            parent: self.parent(s),
            left: Some(self),
            right: self.next_sibling(s),
        }
    }
}
