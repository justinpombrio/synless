use super::node::Node;
use crate::language::Storage;
use crate::util::SynlessBug;

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

impl Location {
    pub fn before(node: Node, s: &Storage) -> Location {
        Location(LocationInner::BeforeNode(node).normalize(s))
    }

    pub fn after(node: Node, _s: &Storage) -> Location {
        // already normal form
        Location(LocationInner::AfterNode(node))
    }

    pub fn text_pos(self) -> Option<(Node, usize)> {
        if let LocationInner::InText(node, char_pos) = self.0 {
            Some((node, char_pos))
        } else {
            None
        }
    }

    pub fn left(self, _s: &Storage) -> Option<Node> {
        use LocationInner::{AfterNode, BeforeNode, BelowNode, InText};

        match self.0 {
            InText(_, _) => None,
            AfterNode(node) => Some(node),
            BeforeNode(_) => None,
            BelowNode(_) => None,
        }
    }

    pub fn right(self, s: &Storage) -> Option<Node> {
        use LocationInner::{AfterNode, BeforeNode, BelowNode, InText};

        match self.0 {
            InText(_, _) => None,
            AfterNode(node) => node.next_sibling(s),
            BeforeNode(node) => Some(node),
            BelowNode(_) => None,
        }
    }

    pub fn parent(self, s: &Storage) -> Option<Node> {
        use LocationInner::{AfterNode, BeforeNode, BelowNode, InText};

        match self.0 {
            InText(node, _) => None,
            AfterNode(node) => node.parent(s),
            BeforeNode(node) => node.parent(s),
            BelowNode(node) => Some(node),
        }
    }

    pub fn root(self, s: &Storage) -> Node {
        self.0.node().root(s)
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
    pub fn goto_bookmark(self, mark: Bookmark, s: &Storage) -> Option<Location> {
        let mark_node = mark.0.node();
        if mark_node.is_valid(s) && mark_node.root(s) == self.root(s) {
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
                let text_len = node.text(s).bug().as_str().len();
                InText(node, i.min(text_len))
            }
            AfterNode(_) => self,
            BeforeNode(node) => node.prev_sibling(s).map(AfterNode).unwrap_or(self),
            BelowNode(parent) => parent.last_child(s).map(AfterNode).unwrap_or(self),
        }
    }

    fn node(self) -> Node {
        use LocationInner::{AfterNode, BeforeNode, BelowNode, InText};

        match self {
            InText(node, _) | AfterNode(node) | BeforeNode(node) | BelowNode(node) => node,
        }
    }
}
