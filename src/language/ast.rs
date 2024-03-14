use super::forest::{Forest, NodeIndex};
use super::language_set::{Arity, Construct, Language, LanguageSet};
use super::text::Text;
use crate::infra::{bug, SynlessBug};
use crate::style::{Condition, StyleLabel, ValidNotation};
use partial_pretty_printer as ppp;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AstId(usize);

struct AstNode {
    id: AstId,
    construct: Construct,
    text: Option<Text>,
}

pub struct DocStorage {
    language_set: LanguageSet,
    forest: Forest<AstNode>,
    next_id: AstId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ast(NodeIndex);

// TODO: doc
pub struct Bookmark(NodeIndex);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Location {
    InText(Ast, usize),
    After(Ast),
    BeforeFirstChild(Ast),
}

impl DocStorage {
    fn next_id(&mut self) -> AstId {
        let id = self.next_id.0;
        self.next_id.0 += 1;
        AstId(id)
    }
}

impl Location {
    pub fn cursor_halves(self, s: &DocStorage) -> (Option<Ast>, Option<Ast>) {
        match self {
            Location::InText(..) => (None, None),
            Location::After(left_sibling) => (Some(left_sibling), left_sibling.next_sibling(s)),
            Location::BeforeFirstChild(parent) => (None, parent.first_child(s)),
        }
    }
}

// TODO: put these methods in any order whatsoever
impl Ast {
    pub fn new_hole(s: &mut DocStorage, lang: Language) -> Ast {
        Ast::new(s, lang.hole_construct(&s.language_set))
    }

    pub fn new(s: &mut DocStorage, construct: Construct) -> Ast {
        let id = s.next_id();
        match construct.arity(&s.language_set) {
            Arity::Texty => Ast(s.forest.new_node(AstNode {
                id,
                construct,
                text: Some(Text::new()),
            })),
            Arity::Listy(_) => Ast(s.forest.new_node(AstNode {
                id,
                construct,
                text: None,
            })),
            Arity::Fixed(sorts) => {
                let parent = s.forest.new_node(AstNode {
                    id,
                    construct,
                    text: None,
                });
                let num_children = sorts.len(&s.language_set);
                let hole_construct = construct.language().hole_construct(&s.language_set);
                for _ in 0..num_children {
                    let child_id = s.next_id();
                    let child = s.forest.new_node(AstNode {
                        id: child_id,
                        construct: hole_construct,
                        text: None,
                    });
                    s.forest.insert_last_child(parent, child);
                }
                Ast(parent)
            }
        }
    }

    pub fn id(self, s: &DocStorage) -> AstId {
        s.forest.data(self.0).id
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

    /// Returns `true` if this is the root of the tree, and `false` if
    /// it isn't (and thus this node has a parent).
    pub fn is_at_root(self, s: &DocStorage) -> bool {
        s.forest.parent(self.0).is_none()
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

    pub fn parent(self, s: &DocStorage) -> Option<Ast> {
        s.forest.parent(self.0).map(Ast)
    }

    pub fn first_child(self, s: &DocStorage) -> Option<Ast> {
        s.forest.first_child(self.0).map(Ast)
    }

    pub fn last_child(self, s: &DocStorage) -> Option<Ast> {
        s.forest
            .first_child(self.0)
            .map(|n| Ast(s.forest.last_sibling(n)))
    }

    pub fn next_sibling(self, s: &DocStorage) -> Option<Ast> {
        s.forest.next(self.0).map(Ast)
    }

    pub fn prev_sibling(self, s: &DocStorage) -> Option<Ast> {
        s.forest.prev(self.0).map(Ast)
    }

    pub fn first_sibling(self, s: &DocStorage) -> Ast {
        Ast(s.forest.first_sibling(self.0))
    }

    pub fn last_sibling(self, s: &DocStorage) -> Ast {
        Ast(s.forest.last_sibling(self.0))
    }

    pub fn root(self, s: &DocStorage) -> Ast {
        Ast(s.forest.root(self.0))
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
    pub fn goto_bookmark(self, mark: Bookmark, s: &DocStorage) -> Option<Ast> {
        if s.forest.is_valid(mark.0) && s.forest.root(self.0) == s.forest.root(mark.0) {
            Some(Ast(mark.0))
        } else {
            None
        }
    }

    /// Check if `other` is allowed where `self` currently is, according to its parent's arity.
    fn accepts_replacement(self, s: &DocStorage, other: Ast) -> bool {
        if let Some(parent) = s.forest.parent(self.0) {
            let sort = match Ast(parent).arity(s) {
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

    fn is_listy_and_accepts_child(self, s: &DocStorage, other: Ast) -> bool {
        let other_construct = s.forest.data(other.0).construct;
        match self.arity(s) {
            Arity::Fixed(_) => false,
            Arity::Listy(sort) => sort.accepts(&s.language_set, other_construct),
            Arity::Texty => false,
        }
    }

    // TODO: doc
    pub fn swap(self, s: &mut DocStorage, other: Ast) -> bool {
        if self.accepts_replacement(s, other) && other.accepts_replacement(s, self) {
            s.forest.swap(self.0, other.0)
        } else {
            false
        }
    }

    pub fn arity(self, s: &DocStorage) -> Arity {
        s.forest.data(self.0).construct.arity(&s.language_set)
    }

    pub fn is_comment_or_ws(self, s: &DocStorage) -> bool {
        s.forest
            .data(self.0)
            .construct
            .is_comment_or_ws(&s.language_set)
    }

    pub fn notation(self, s: &DocStorage) -> &ValidNotation {
        s.forest.data(self.0).construct.notation(&s.language_set)
    }

    /// Borrow the text of a texty node.
    pub fn text(self, s: &DocStorage) -> Option<&Text> {
        s.forest.data(self.0).text.as_ref()
    }

    /// Mutably borrow the text of a texty node.
    pub fn text_mut(self, s: &mut DocStorage) -> Option<&mut Text> {
        s.forest.data_mut(self.0).text.as_mut()
    }

    /// Go to this node's `n`'th child.
    /// Panics if `n` is out of bounds, or if this node is texty.
    pub fn nth_child(self, s: &DocStorage, n: usize) -> Ast {
        Ast(s.forest.nth_child(self.0, n).bug_msg("Ast::nth_child"))
    }

    // TODO: doc (new_sibling must be root)
    pub fn insert_before(self, s: &mut DocStorage, new_sibling: Ast) -> bool {
        if let Some(parent) = self.parent(s) {
            if parent.is_listy_and_accepts_child(s, new_sibling) {
                s.forest.insert_before(self.0, new_sibling.0);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    // TODO: doc (new_sibling must be root)
    pub fn insert_after(self, s: &mut DocStorage, new_sibling: Ast) -> bool {
        if let Some(parent) = self.parent(s) {
            if parent.is_listy_and_accepts_child(s, new_sibling) {
                s.forest.insert_after(self.0, new_sibling.0);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    // TODO: doc (new_child must be root)
    pub fn insert_last_child(self, s: &mut DocStorage, new_child: Ast) -> bool {
        if self.is_listy_and_accepts_child(s, new_child) {
            s.forest.insert_last_child(self.0, new_child.0);
            true
        } else {
            false
        }
    }

    // TODO: doc (parent must be listy)
    pub fn remove(self, s: &mut DocStorage) -> bool {
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
