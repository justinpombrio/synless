use super::forest::{Forest, NodeIndex};
use super::language_set::{Construct, DocCondition, LanguageSet, StyleLabel, ValidNotation};
use super::text::Text;
use crate::infra::SynlessBug;
use partial_pretty_printer as ppp;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct AstId(usize);

struct AstNode {
    id: AstId,
    construct: Construct,
    text: Option<Text>,
}

struct DocStorage {
    language_set: LanguageSet,
    forest: Forest<AstNode>,
}

/********************************************
 *         AstRef: implements PrettyDoc     *
 ********************************************/

// TODO: These will need more data, and should be moved somewhere.
type Style = ();

#[derive(Clone, Copy)]
struct AstRef<'d> {
    storage: &'d DocStorage,
    node_index: NodeIndex,
}

impl<'d> AstRef<'d> {
    fn node(self) -> &'d AstNode {
        self.storage.forest.data(self.node_index)
    }
}

impl<'d> ppp::PrettyDoc<'d> for AstRef<'d> {
    type Id = AstId;
    type Style = Style;
    type StyleLabel = StyleLabel;
    type Condition = DocCondition;

    fn id(self) -> AstId {
        self.node().id
    }

    fn notation(self) -> &'d ValidNotation {
        self.node().construct.notation(&self.storage.language_set)
    }

    fn condition(self, condition: &DocCondition) -> bool {
        // TODO
        false
    }

    fn lookup_style(self, style_label: StyleLabel) -> Style {
        // TODO
        ()
    }

    fn node_style(self) -> Style {
        // TODO
        ()
    }

    fn num_children(self) -> Option<usize> {
        if self.node().text.is_some() {
            None
        } else {
            Some(self.storage.forest.num_children(self.node_index))
        }
    }

    fn unwrap_text(self) -> &'d str {
        self.node().text.as_ref().bug().as_str()
    }

    fn unwrap_child(self, i: usize) -> AstRef<'d> {
        let mut child = self.storage.forest.first_child(self.node_index).bug();
        for _ in 0..i {
            child = self.storage.forest.next(child).bug();
        }
        AstRef {
            storage: self.storage,
            node_index: child,
        }
    }

    fn unwrap_last_child(self) -> AstRef<'d> {
        let first_child = self.storage.forest.first_child(self.node_index).bug();
        let last_child = self.storage.forest.last_sibling(first_child);
        AstRef {
            storage: self.storage,
            node_index: last_child,
        }
    }

    fn unwrap_prev_sibling(self, _: Self, _: usize) -> AstRef<'d> {
        AstRef {
            storage: self.storage,
            node_index: self.storage.forest.prev(self.node_index).bug(),
        }
    }
}

impl<'d> fmt::Debug for AstRef<'d> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AstRef({:?})", self.node_index)
    }
}

// Ast methods:
// - navigation, bookmarks
// - editing: text, children
// - grammar: get construct, arity
// - construction: hole, text, branch
// - PrettyDoc: get construct, notation
