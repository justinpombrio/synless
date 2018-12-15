use crate::notation::Notation;
use crate::layout::{LayoutRegion, Layout, Bounds, Layouts,
                    compute_bounds, compute_layouts, text_bounds};
use super::pretty_screen::PrettyScreen;
use self::Layout::*;


/// A "document" that supports the necessary methods to be pretty-printed.
///
/// To pretty-print, you need:
///
/// 1. A document that implements PrettyDocument, and
/// 2. A screen that implements PrettyScreen.
pub trait PrettyDocument : Sized + Clone {
    /// The minimum number of children this node can have. (See `grammar::Arity`)
    fn parent(&self) -> Option<Self>;
    /// The node's `i`th child. `i` will always be valid.
    fn child(&self, i: usize) -> Self;
    /// All of the node's (immediate) children.
    fn children(&self) -> Vec<Self>;
    /// The node's notation.
    fn notation(&self) -> &Notation;
    /// If the node contains text, that text. Otherwise `None`.
    fn text(&self) -> Option<&str>;

    // TODO: have this return a reference instead?
    /// Get the Bounds within which this document node can be displayed,
    /// given information about its children. **For efficiency, you should
    /// cache the result of `Bounds::compute` every time the document changes.**
    fn bounds(&self) -> Bounds;

    /// Pretty-print entire document.
    fn pretty_print<Screen>(&self, screen: &mut Screen) -> Result<(), Screen::Error>
        where Screen: PrettyScreen
    {
        // TODO: wrong
        let lay = Layouts::compute(self).fit_bound(screen.size()?);
        pp(self, screen, lay)
    }
}


/// _Compute_ the possible bounds of this node. This is required in order to
/// pretty-print it. Note that:
///
/// 1. This depends on the Notation of this node, plus the Bounds of its
/// (immediate) children.
/// 2. This _does not_ depend on the width with which the document will be
/// pretty-printed.
impl Bounds {
    pub fn compute<Doc: PrettyDocument>(doc: &Doc) -> Bounds {
        compute_bounds(&child_bounds(doc), &expanded_notation(doc))
    }
}

impl Layouts {
    pub fn compute<Doc: PrettyDocument>(doc: &Doc) -> Layouts {
        compute_layouts(&child_bounds(doc), &expanded_notation(doc))
    }
}

fn child_bounds<Doc: PrettyDocument>(doc: &Doc) -> Vec<Bounds> {
    match doc.text() {
        None => doc.children().iter().map(|child| child.bounds()).collect(),
        Some(text) => vec!(text_bounds(text))
    }
}

fn expanded_notation<Doc: PrettyDocument>(doc: &Doc) -> Notation {
    let len = match doc.text() {
        None       => doc.children().len(),
        Some(text) => text.chars().count()
    };
    doc.notation().expand(len)
}

// TODO: shading and highlighting
fn pp<Doc, Screen>(doc: &Doc, screen: &mut Screen, lay: LayoutRegion)
                   -> Result<(), Screen::Error>
    where Screen: PrettyScreen, Doc: PrettyDocument
{
    match lay.layout {
        Empty => {
            Ok(())
        }
        Literal(text, style) => {
            screen.print(lay.region.pos, &text, style)
        }
        Text(style) => {
            let text = doc.text()
                .expect("Expected text while transcribing; found branch node");
            screen.print(lay.region.pos, text, style)
        }
        Child(i) => {
            let child = &doc.child(i);
            // TODO: shouldn't need to shift layout here?
            let mut child_lay = Layouts::compute(child).fit_bound(lay.region.bound);
            child_lay.shift_by(lay.region.pos);
            pp(child, screen, child_lay)
        }
        Concat(box lay1, box lay2) => {
            pp(doc, screen, lay1)?;
            pp(doc, screen, lay2)
        }
        Horz(box lay1, box lay2) => {
            pp(doc, screen, lay1)?;
            pp(doc, screen, lay2)
        }
        Vert(box lay1, box lay2) => {
            pp(doc, screen, lay1)?;
            pp(doc, screen, lay2)
        }
    }
}
