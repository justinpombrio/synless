use crate::infra::SynlessBug;
use crate::language::{Ast, AstId, DocStorage, Location};
use crate::style::{Condition, CursorHalf, Style, StyleLabel, ValidNotation};
use partial_pretty_printer as ppp;
use std::fmt;

// TODO: handle text char highlighting

#[derive(Clone, Copy)]
pub struct DocRef<'d> {
    storage: &'d DocStorage,
    cursor_pos: Location,
    left_cursor: Option<Ast>,
    right_cursor: Option<Ast>,
    ast: Ast,
}

impl<'d> DocRef<'d> {
    pub fn new(storage: &'d DocStorage, cursor_pos: Location, ast: Ast) -> DocRef<'d> {
        let (left_cursor, right_cursor) = cursor_pos.cursor_halves(storage);
        DocRef {
            storage,
            cursor_pos,
            left_cursor,
            right_cursor,
            ast,
        }
    }

    fn with_ast(self, ast: Ast) -> Self {
        DocRef {
            storage: self.storage,
            cursor_pos: self.cursor_pos,
            left_cursor: self.left_cursor,
            right_cursor: self.right_cursor,
            ast,
        }
    }
}

impl<'d> ppp::PrettyDoc<'d> for DocRef<'d> {
    type Id = AstId;
    type Style = Style;
    type StyleLabel = StyleLabel;
    type Condition = Condition;

    fn id(self) -> AstId {
        self.ast.id(self.storage)
    }

    fn notation(self) -> &'d ValidNotation {
        self.ast.notation(self.storage)
    }

    fn condition(self, condition: &Condition) -> bool {
        match condition {
            Condition::IsEmptyText => self
                .ast
                .text(self.storage)
                .map(|text| text.as_str().is_empty())
                .unwrap_or(false),
            Condition::IsCommentOrWs => self.ast.is_comment_or_ws(self.storage),
            Condition::NeedsSeparator => {
                if self.ast.is_comment_or_ws(self.storage) {
                    return false;
                }
                let mut sibling = self.ast;
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
            StyleLabel::Open => {
                let mut style = Style::default();
                if self.cursor_pos == Location::BeforeFirstChild(self.ast) {
                    style.cursor = Some(CursorHalf::Left)
                }
                style
            }
            StyleLabel::Close => {
                let mut style = Style::default();
                let at_end = match self.cursor_pos {
                    Location::InText(..) => todo!(),
                    Location::BeforeFirstChild(parent) => {
                        parent == self.ast && self.ast.first_child(self.storage).is_none()
                    }
                    Location::After(sibling) => self.ast.last_child(self.storage) == Some(sibling),
                };
                // TODO: perhaps rewrite as:
                // if self.ast.gap_after_children(self.storage) == self.cursor_pos
                if at_end {
                    style.cursor = Some(CursorHalf::Right)
                }
                style
            }
            StyleLabel::Properties {
                color,
                bold,
                italic,
                underlined,
                priority,
            } => Style {
                color: color.map(|x| (x, priority)),
                bold: bold.map(|x| (x, priority)),
                italic: italic.map(|x| (x, priority)),
                underlined: underlined.map(|x| (x, priority)),
                cursor: None,
            },
        }
    }

    fn node_style(self) -> Style {
        let mut style = Style::default();
        if self.left_cursor == Some(self.ast) {
            style.cursor = Some(CursorHalf::Left);
        } else if self.right_cursor == Some(self.ast) {
            style.cursor = Some(CursorHalf::Right);
        }
        style
    }

    fn num_children(self) -> Option<usize> {
        self.ast.num_children(self.storage)
    }

    fn unwrap_text(self) -> &'d str {
        self.ast.text(self.storage).bug().as_str()
    }

    fn unwrap_child(self, i: usize) -> Self {
        let child = self.ast.nth_child(self.storage, i);
        self.with_ast(child)
    }

    fn unwrap_last_child(self) -> Self {
        let last_child = self.ast.last_child(self.storage).bug();
        self.with_ast(last_child)
    }

    fn unwrap_prev_sibling(self, _: Self, _: usize) -> Self {
        let sibling = self.ast.prev_sibling(self.storage).bug();
        self.with_ast(sibling)
    }
}

impl<'d> fmt::Debug for DocRef<'d> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DocRef({:?}, {:?})", self.ast, self.cursor_pos)
    }
}
