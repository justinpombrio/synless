use crate::language::Storage;
use crate::style::{
    Condition, CursorHalf, Style, StyleLabel, ValidNotation, HOLE_STYLE, LEFT_CURSOR_STYLE,
    RIGHT_CURSOR_STYLE,
};
use crate::tree::{Location, Node, NodeId};
use crate::util::{bug, SynlessBug};
use partial_pretty_printer as ppp;
use std::fmt;
use std::sync::OnceLock;

fn get_text_notation() -> &'static ValidNotation {
    static TEXT_NOTATION: OnceLock<ValidNotation> = OnceLock::new();
    TEXT_NOTATION.get_or_init(|| {
        let notation = ppp::Notation::Text;
        notation.validate().bug()
    })
}

#[derive(Clone, Copy)]
pub struct DocRef<'d> {
    storage: &'d Storage,
    cursor_loc: Location,
    node: Node,
    text_pos: Option<CursorHalf>,
}

impl<'d> DocRef<'d> {
    pub fn new(storage: &'d Storage, cursor_loc: Location, node: Node) -> DocRef<'d> {
        DocRef {
            storage,
            cursor_loc,
            node,
            text_pos: None,
        }
    }

    fn on_virtual_text_parent(&self) -> bool {
        self.node.text(self.storage).is_some() && self.text_pos.is_none()
    }

    /// Char index to split this node's text at
    fn text_index(&self) -> usize {
        if let Some((node, char_pos)) = self.cursor_loc.text_pos() {
            if node == self.node {
                char_pos
            } else {
                0
            }
        } else {
            0
        }
    }
}

impl<'d> ppp::PrettyDoc<'d> for DocRef<'d> {
    type Id = (NodeId, u16);
    type Style = Style;
    type StyleLabel = StyleLabel;
    type Condition = Condition;

    fn id(self) -> (NodeId, u16) {
        let id = self.node.id(self.storage);
        match self.text_pos {
            None => (id, 0),
            Some(CursorHalf::Left) => (id, 1),
            Some(CursorHalf::Right) => (id, 2),
        }
    }

    fn notation(self) -> &'d ValidNotation {
        match self.text_pos {
            None => self.node.notation(self.storage),
            Some(_) => get_text_notation(),
        }
    }

    fn condition(self, condition: &Condition) -> bool {
        match condition {
            Condition::IsEmptyText => self
                .node
                .text(self.storage)
                .map(|text| text.as_str().is_empty())
                .unwrap_or(false),
            Condition::IsCommentOrWs => self.node.is_comment_or_ws(self.storage),
            Condition::NeedsSeparator => {
                if self.node.is_comment_or_ws(self.storage) {
                    return false;
                }
                let mut sibling = self.node;
                while let Some(next_sibling) = sibling.next_sibling(self.storage) {
                    if !next_sibling.is_comment_or_ws(self.storage) {
                        return true;
                    }
                    sibling = next_sibling;
                }
                false
            }
        }
    }

    fn lookup_style(self, style_label: StyleLabel) -> Style {
        match style_label {
            StyleLabel::Hole => HOLE_STYLE,
            StyleLabel::Open => {
                let parent = self.cursor_loc.parent_node(self.storage);
                let left = self.cursor_loc.left_node(self.storage);
                if parent == Some(self.node) && left.is_none() {
                    LEFT_CURSOR_STYLE
                } else {
                    Style::default()
                }
            }
            StyleLabel::Close => {
                let parent = self.cursor_loc.parent_node(self.storage);
                let right = self.cursor_loc.right_node(self.storage);
                if parent == Some(self.node) && right.is_none() {
                    RIGHT_CURSOR_STYLE
                } else {
                    Style::default()
                }
            }
            StyleLabel::Properties {
                fg_color,
                bg_color,
                bold,
                underlined,
                priority,
            } => Style {
                fg_color: fg_color.map(|x| (x, priority)),
                bg_color: bg_color.map(|x| (x, priority)),
                bold: bold.map(|x| (x, priority)),
                underlined: underlined.map(|x| (x, priority)),
                cursor: None,
                is_hole: false,
            },
        }
    }

    fn node_style(self) -> Style {
        if self.text_pos.is_some() {
            Style::default()
        } else if self.cursor_loc.left_node(self.storage) == Some(self.node) {
            LEFT_CURSOR_STYLE
        } else if self.cursor_loc.right_node(self.storage) == Some(self.node) {
            RIGHT_CURSOR_STYLE
        } else {
            Style::default()
        }
    }

    fn num_children(self) -> Option<usize> {
        if self.on_virtual_text_parent() {
            Some(2)
        } else {
            self.node.num_children(self.storage)
        }
    }

    fn unwrap_text(self) -> &'d str {
        let text = &self.node.text(self.storage).bug();
        match self.text_pos {
            None => bug!("DocRef::unwrap_text non-virtual text"),
            Some(CursorHalf::Left) => text.as_split_str(self.text_index()).0,
            Some(CursorHalf::Right) => text.as_split_str(self.text_index()).1,
        }
    }

    fn unwrap_child(self, i: usize) -> Self {
        if self.on_virtual_text_parent() {
            let cursor_half = match i {
                0 => CursorHalf::Left,
                1 => CursorHalf::Right,
                _ => bug!("DocRef::unwrap_child virtual text child OOB"),
            };
            DocRef {
                text_pos: Some(cursor_half),
                ..self
            }
        } else {
            let child = self.node.nth_child(self.storage, i).bug();
            DocRef {
                node: child,
                ..self
            }
        }
    }

    fn unwrap_last_child(self) -> Self {
        if self.on_virtual_text_parent() {
            DocRef {
                text_pos: Some(CursorHalf::Right),
                ..self
            }
        } else {
            let last_child = self.node.last_child(self.storage).bug();
            DocRef {
                node: last_child,
                ..self
            }
        }
    }

    fn unwrap_prev_sibling(self, _: Self, _: usize) -> Self {
        match self.text_pos {
            Some(CursorHalf::Left) => bug!("unwrap_prev_sibling: virtual text OOB"),
            Some(CursorHalf::Right) => DocRef {
                text_pos: Some(CursorHalf::Left),
                ..self
            },
            None => {
                let sibling = self.node.prev_sibling(self.storage).bug();
                DocRef {
                    node: sibling,
                    ..self
                }
            }
        }
    }
}

impl<'d> fmt::Debug for DocRef<'d> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "DocRef({:?}, {:?}, {:?})",
            self.node, self.cursor_loc, self.text_pos
        )
    }
}
