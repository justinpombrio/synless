use crate::language::Storage;
use crate::style::{
    Condition, Style, StyleLabel, ValidNotation, CLOSE_STYLE, CURSOR_STYLE, HOLE_STYLE,
};
use crate::tree::{Location, Node, NodeId};
use crate::util::{error, SynlessBug, SynlessError};
use partial_pretty_printer as ppp;
use std::fmt;

#[derive(thiserror::Error, Debug)]
pub enum PrettyDocError {
    #[error("No source notation available for language '{0}'")]
    NoSourceNotation(String),
}

#[derive(Clone, Copy)]
pub struct DocRef<'d> {
    storage: &'d Storage,
    cursor_loc: Location,
    node: Node,
    use_source_notation: bool,
}

impl<'d> DocRef<'d> {
    pub fn new_display(storage: &'d Storage, cursor_loc: Location, node: Node) -> DocRef<'d> {
        DocRef {
            storage,
            cursor_loc,
            node,
            use_source_notation: false,
        }
    }

    pub fn new_source(storage: &'d Storage, cursor_loc: Location, node: Node) -> DocRef<'d> {
        DocRef {
            storage,
            cursor_loc,
            node,
            use_source_notation: true,
        }
    }
}

impl<'d> ppp::PrettyDoc<'d> for DocRef<'d> {
    type Id = NodeId;
    type Style = Style;
    type StyleLabel = StyleLabel;
    type Condition = Condition;
    type Error = PrettyDocError;

    fn id(self) -> Result<NodeId, Self::Error> {
        Ok(self.node.id(self.storage))
    }

    fn notation(self) -> Result<&'d ValidNotation, Self::Error> {
        if self.use_source_notation {
            self.node.source_notation(self.storage).ok_or_else(|| {
                let lang = self.node.language(self.storage);
                PrettyDocError::NoSourceNotation(lang.name(self.storage).to_owned())
            })
        } else {
            Ok(self.node.display_notation(self.storage))
        }
    }

    fn condition(self, condition: &Condition) -> Result<bool, Self::Error> {
        Ok(match condition {
            Condition::IsEmptyText => self
                .node
                .text(self.storage)
                .map(|text| text.as_str().is_empty())
                .unwrap_or(false),
            Condition::IsCommentOrWs => self.node.is_comment_or_ws(self.storage),
            Condition::NeedsSeparator => {
                if self.node.is_comment_or_ws(self.storage) {
                    return Ok(false);
                }
                let mut sibling = self.node;
                while let Some(next_sibling) = sibling.next_sibling(self.storage) {
                    if !next_sibling.is_comment_or_ws(self.storage) {
                        return Ok(true);
                    }
                    sibling = next_sibling;
                }
                false
            }
        })
    }

    fn lookup_style(self, style_label: StyleLabel) -> Result<Style, Self::Error> {
        Ok(match style_label {
            StyleLabel::Hole => HOLE_STYLE,
            StyleLabel::Open => Style::default(),
            StyleLabel::Close => {
                let parent = self.cursor_loc.parent_node(self.storage);
                let node_at_cursor = self.cursor_loc.node(self.storage);
                if parent == Some(self.node) && node_at_cursor.is_none() {
                    CLOSE_STYLE
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
        })
    }

    fn node_style(self) -> Result<Style, Self::Error> {
        let style = if self.cursor_loc.node(self.storage) == Some(self.node) {
            CURSOR_STYLE
        } else {
            Style::default()
        };
        Ok(style)
    }

    fn num_children(self) -> Result<Option<usize>, Self::Error> {
        Ok(self.node.num_children(self.storage))
    }

    fn unwrap_text(self) -> Result<&'d str, Self::Error> {
        Ok(self.node.text(self.storage).bug().as_str())
    }

    fn unwrap_child(self, n: usize) -> Result<Self, Self::Error> {
        Ok(DocRef {
            node: self.node.nth_child(self.storage, n).bug(),
            ..self
        })
    }

    fn unwrap_last_child(self) -> Result<Self, Self::Error> {
        Ok(DocRef {
            node: self.node.last_child(self.storage).bug(),
            ..self
        })
    }

    fn unwrap_prev_sibling(self, _: Self, _: usize) -> Result<Self, Self::Error> {
        Ok(DocRef {
            node: self.node.prev_sibling(self.storage).bug(),
            ..self
        })
    }
}

impl<'d> fmt::Debug for DocRef<'d> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "DocRef({:?}, {:?}, {})",
            self.node, self.cursor_loc, self.use_source_notation
        )
    }
}

impl From<ppp::PrintingError<PrettyDocError>> for SynlessError {
    fn from(error: ppp::PrintingError<PrettyDocError>) -> SynlessError {
        if let ppp::PrintingError::PrettyDoc(err) = &error {
            error!(Printing, "{}", err)
        } else {
            error!(Printing, "{}", error)
        }
    }
}
