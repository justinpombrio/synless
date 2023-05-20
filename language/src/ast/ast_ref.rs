use super::ast::{Id, NodeData};
use super::text::Text;
use crate::language::{Arity, LanguageSet};
use forest::{Bookmark, TreeRef};
use partial_pretty_printer::{PrettyDoc, ValidNotation};

/// An immutable reference to a node in an AST.
#[derive(Clone, Copy)]
pub struct AstRef<'f, 'l> {
    pub(super) lang: &'f LanguageSet<'l>,
    pub(super) tree_ref: TreeRef<'f, NodeData<'l>, Text>,
}

impl<'f, 'l> AstRef<'f, 'l> {
    /// Get the arity of this node.
    pub fn arity(self) -> &'l Arity {
        let node = self.tree_ref.data();
        &node.grammar.construct(node.construct_id).arity
    }

    /// Save a bookmark to return to later.
    pub fn bookmark(self) -> Bookmark {
        self.tree_ref.bookmark()
    }

    /// Return to a previously saved bookmark, as long as that bookmark's node
    /// is present somewhere in this tree. This will work even if the Tree has
    /// been modified since the bookmark was created. However, this method will
    /// return `None` if the bookmark's node has since been deleted, or if it is
    /// currently located in a different tree.
    pub fn lookup_bookmark(self, mark: Bookmark) -> Option<AstRef<'f, 'l>> {
        self.tree_ref.lookup_bookmark(mark).map(|tr| AstRef {
            lang: self.lang,
            tree_ref: tr,
        })
    }
}

impl<'d> PrettyDoc<'d> for AstRef<'d, 'd> {
    type Id = Id;

    fn id(self) -> Self::Id {
        self.tree_ref.data().id
    }

    fn notation(self) -> &'d ValidNotation {
        let node = self.tree_ref.data();
        // TODO: No HashMap lookups while pretty printing!
        let lang_name = &node.grammar.language_name();
        &self
            .lang
            .current_notation_set(lang_name)
            .lookup(node.construct_id)
    }

    /// Get this node's number of children, or `None` if it contains text instead.
    fn num_children(self) -> Option<usize> {
        if self.tree_ref.is_leaf() {
            None
        } else {
            Some(self.tree_ref.num_children())
        }
    }

    /// Get this node's text, or panic.
    fn unwrap_text(self) -> &'d str {
        assert!(self.arity().is_texty());
        self.tree_ref.leaf().as_str()
    }

    /// Get this node's i'th child, or panic.
    fn unwrap_child(self, i: usize) -> Self {
        AstRef {
            lang: self.lang,
            tree_ref: self.tree_ref.child(i),
        }
    }
}
