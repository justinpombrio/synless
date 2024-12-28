use crate::engine::Search;
use crate::language::Storage;
use crate::style::{Condition, CursorKind, Style, StyleLabel, ValidNotation};
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
    cursor_loc: Option<Location>,
    node: Node,
    use_source_notation: bool,
    search: Option<&'d Search>,
}

impl<'d> DocRef<'d> {
    pub fn new_display(
        storage: &'d Storage,
        cursor_loc: Option<Location>,
        node: Node,
        search: &'d Option<Search>,
    ) -> DocRef<'d> {
        DocRef {
            storage,
            cursor_loc,
            node,
            use_source_notation: false,
            search: search.as_ref(),
        }
    }

    pub fn new_source(
        storage: &'d Storage,
        cursor_loc: Option<Location>,
        node: Node,
    ) -> DocRef<'d> {
        DocRef {
            storage,
            cursor_loc,
            node,
            use_source_notation: true,
            search: None,
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
        let s = self.storage;
        let construct = self.node.construct(s);
        let lang = self.node.language(s);

        #[allow(clippy::collapsible_else_if)]
        if self.use_source_notation {
            let notation = if construct.is_hole(s) || self.node.is_invalid_text(s) {
                lang.hole_source_notation(s)
            } else {
                lang.source_notation(s).map(|ns| ns.notation(s, construct))
            };
            notation.ok_or_else(|| PrettyDocError::NoSourceNotation(lang.name(s).to_owned()))
        } else {
            if construct.is_hole(s) {
                Ok(lang.hole_display_notation(s))
            } else {
                Ok(lang.display_notation(s).notation(s, construct))
            }
        }
    }

    fn condition(self, condition: &Condition) -> Result<bool, Self::Error> {
        Ok(match condition {
            Condition::IsEmptyText => self
                .node
                .text(self.storage)
                .map(|text| text.as_str().is_empty())
                .unwrap_or(false),
            Condition::IsInvalidText => self.node.is_invalid_text(self.storage),
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
            StyleLabel::Open => {
                if let Some(cursor_loc) = self.cursor_loc {
                    let parent = cursor_loc.parent_node(self.storage);
                    let node_at_cursor = cursor_loc.at_node(self.storage);
                    if parent == Some(self.node) && node_at_cursor.is_none() {
                        Style {
                            cursor: Some(CursorKind::BelowNode),
                            ..Style::const_default()
                        }
                    } else {
                        Style::default()
                    }
                } else {
                    Style::default()
                }
            }
            StyleLabel::Close => Style::default(),
            StyleLabel::Properties {
                fg_color,
                bg_color,
                bold,
                underlined,
                priority,
            } => Style {
                fg_color: fg_color.map(|c| (c, priority)),
                bg_color: bg_color.map(|c| (c, priority)),
                bold: bold.map(|b| (b, priority)),
                underlined: underlined.map(|b| (b, priority)),
                cursor: None,
                is_hole: false,
                is_highlighted: false,
                is_invalid: false,
            },
        })
    }

    fn node_style(self) -> Result<Style, Self::Error> {
        let cursor = self.cursor_loc.and_then(|cursor| {
            if cursor.at_node(self.storage) == Some(self.node) {
                Some(CursorKind::AtNode)
            } else if cursor.in_text_node(self.storage) == Some(self.node) {
                Some(CursorKind::InText)
            } else {
                None
            }
        });
        let is_hole = self.node.construct(self.storage).is_hole(self.storage);
        let is_highlighted = self
            .search
            .map(|search| search.highlight && search.matches(self.storage, self.node))
            .unwrap_or(false);
        let is_invalid = self.node.is_invalid_text(self.storage);

        Ok(Style {
            cursor,
            is_hole,
            is_highlighted,
            is_invalid,
            ..Style::const_default()
        })
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

impl fmt::Debug for DocRef<'_> {
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
