use forest::{Bookmark, TreeRef};
use language::Arity;
use pretty::{Bounds, Notation, PrettyDocument};

use crate::ast::ast::{Ast, Node, ReadText};
use crate::text::Text;

impl<'f, 'l> Ast<'l> {
    pub fn borrow(&'f self) -> AstRef<'f, 'l> {
        AstRef {
            tree_ref: self.tree.borrow(),
        }
    }
}

/// An immutable reference to a node in an AST.
#[derive(Clone)]
pub struct AstRef<'f, 'l> {
    tree_ref: TreeRef<'f, Node<'l>, Text>,
}

impl<'f, 'l> AstRef<'f, 'l> {
    /// Get the parent node and this node's index. Returns `None` if we're
    /// already at the root of the tree.
    pub fn parent(&self) -> Option<(AstRef<'f, 'l>, usize)> {
        match self.tree_ref.parent() {
            None => None,
            Some(tree_ref) => {
                let index = self.tree_ref.index();
                Some((AstRef { tree_ref: tree_ref }, index))
            }
        }
    }

    /// Get the arity of this node.
    pub fn arity(&self) -> Arity {
        self.tree_ref.data().construct.arity.clone()
    }

    /// Get the children of a Fixed, Flexible, or Mixed node.
    ///
    /// # Panics
    ///
    /// Panics if the arity of this node is `Text`.
    pub fn children(&self) -> impl Iterator<Item = AstRef<'f, 'l>> {
        self.tree_ref.children().map(|tr| AstRef { tree_ref: tr })
    }

    /// Get the `i`th child of a Fixed, Flexible, or Mixed node.
    ///
    /// # Panics
    ///
    /// Panics if the arity of this node is `Text`.
    pub fn child(&self, i: usize) -> AstRef<'f, 'l> {
        AstRef {
            tree_ref: self.tree_ref.child(i),
        }
    }

    /// Get a shared reference to the text at this node.
    ///
    /// # Panics
    ///
    /// Panics if the arity of this node is not `Text`.
    pub fn text(&self) -> ReadText<'f, 'l> {
        ReadText(self.tree_ref.leaf())
    }

    /// Save a bookmark to return to later.
    pub fn bookmark(&self) -> Bookmark {
        self.tree_ref.bookmark()
    }

    /// Return to a previously saved bookmark, as long as that bookmark's node
    /// is present somewhere in this tree. This will work even if the Tree has
    /// been modified since the bookmark was created. However, this method will
    /// return `None` if the bookmark's node has since been deleted, or if it is
    /// currently located in a different tree.
    pub fn lookup_bookmark(&self, mark: Bookmark) -> Option<AstRef<'f, 'l>> {
        self.tree_ref
            .lookup_bookmark(mark)
            .map(|tr| AstRef { tree_ref: tr })
    }
}

impl<'f, 'l> PrettyDocument for AstRef<'f, 'l> {
    type TextRef = ReadText<'f, 'l>;

    fn notation(&self) -> &Notation {
        &self.tree_ref.data().notation
    }

    fn bounds(&self) -> Bounds {
        self.tree_ref.data().bounds.clone()
    }

    fn text(&self) -> Option<Self::TextRef> {
        if self.arity() == Arity::Text {
            Some(self.text())
        } else {
            None
        }
    }

    fn parent(&self) -> Option<(AstRef<'f, 'l>, usize)> {
        self.parent()
    }

    // TODO: probably panics on mixed nodes.
    fn child(&self, i: usize) -> AstRef<'f, 'l> {
        self.child(i)
    }

    fn children(&self) -> Vec<AstRef<'f, 'l>> {
        self.children().collect()
    }
}
