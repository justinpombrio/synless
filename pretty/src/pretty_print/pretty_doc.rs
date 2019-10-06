use crate::geometry::{Col, Pos, Row};
use crate::layout::{compute_bounds, compute_layout, compute_text_bounds, Bounds, Layout};
use crate::notation::Notation;

/// A "document" that supports the necessary methods to be pretty-printed.
pub trait PrettyDocument: Sized + Clone {
    type TextRef: AsRef<str>;

    /// This node's parent, together with the index of this node (or `None` if
    /// this is the root node).
    fn parent(&self) -> Option<(Self, usize)>;
    /// The node's `i`th child. `i` will always be valid.
    fn child(&self, i: usize) -> Self;
    /// All of the node's (immediate) children.
    fn children(&self) -> Vec<Self>;
    /// Mutable reference to the node's notation.
    fn notation_mut(&mut self) -> &mut Notation;
    /// The node's notation.
    fn notation(&self) -> &Notation;
    /// If the node contains text, that text. Otherwise `None`.
    fn text(&self) -> Option<Self::TextRef>;

    /// Get the Bounds within which this document node can be displayed,
    /// given information about its children. This can be computed via
    /// `Bounds::compute`. **However, it is an expensive operation, so you
    /// should cache it.** The results of `Bounds::compute` are valid until the
    /// node, _or any of its descendants_, are edited.
    fn bounds(&self) -> Bounds;

    /// Goto the root of the document.
    fn root(&self) -> Self {
        let mut root = self.clone();
        while let Some((parent, _)) = root.parent() {
            root = parent;
        }
        root
    }
    /// If this node contains text, and that text is the empty string.
    fn is_empty_text(&self) -> bool {
        match self.text() {
            None => false,
            Some(text) => text.as_ref().is_empty(),
        }
    }
    /// Gather the bounds of the children of this node.
    fn child_bounds(&self) -> Vec<Bounds> {
        match self.text() {
            None => self
                .children()
                .iter()
                .map(|child| child.bounds().to_owned())
                .collect(),
            Some(text) => vec![compute_text_bounds(text.as_ref())],
        }
    }
    /// Find the minimum height required to pretty-print this document with the
    /// given width.
    fn required_height(&self, width: Col) -> Row {
        self.bounds().fit_width(width).height
    }
}

impl Bounds {
    /// _Compute_ the possible bounds of this node. This is required in order to
    /// pretty-print it. Note that:
    ///
    /// 1. This depends on the Notation of this node, plus the Bounds of its
    /// (immediate) children.
    /// 2. This _does not_ depend on the width with which the document will be
    /// pretty-printed.
    pub fn compute<Doc: PrettyDocument>(doc: &mut Doc) -> Bounds {
        let bounds = doc.child_bounds();
        let is_empty_text = doc.is_empty_text();
        compute_bounds(doc.notation_mut(), &bounds, is_empty_text)
    }
}

impl Layout {
    // TODO: move this and Bounds.compute into their respective files, making
    // them depend on PrettyDocument?
    /// Lay-out a document node.
    pub fn compute<Doc: PrettyDocument>(doc: &Doc, pos: Pos, width: Col) -> Layout {
        let child_bounds = doc.child_bounds();
        let is_empty_text = doc.is_empty_text();
        let notation = doc.notation();
        compute_layout(notation, pos, width, &child_bounds, is_empty_text)
    }
}
