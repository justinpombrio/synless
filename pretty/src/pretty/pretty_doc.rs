use crate::geometry::Region;
use crate::notation::Notation;
use crate::layout::{LayoutRegion, Layout, Bounds, Layouts,
                    compute_bounds, compute_layouts, text_bounds};
use super::pretty_screen::PrettyScreen;
use self::Layout::*;


pub trait PrettyDocument : Sized + Clone {
    /// The minimum number of children this node can have. (See `grammar::Arity`)
    fn arity(&self) -> usize;
    /// This node's parent, together with the index of this node (or `None` if
    /// this is the root node).
    fn parent(&self) -> Option<(Self, usize)>;
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
        let lay = Layouts::compute(&self.root()).fit_bound(screen.size()?);
        pp(self, screen, lay)
    }

    /// Locate the selected node, in the coordinate system of the entire document.
    fn locate_cursor<Screen>(&self, screen: &Screen) -> Result<Region, Screen::Error>
        where Screen: PrettyScreen
    {
        // Find the root of the Document, and the path from the root to the
        // selected node.
        let mut path = vec!();
        let mut root = self.clone();
        while let Some((parent, i)) = root.parent() {
            root = parent;
            path.push(i);
        }
        path.reverse();
        // Recursively compute the cursor region.
        let lay = Layouts::compute(&root).fit_bound(screen.size()?);
        Ok(loc_cursor(&root, &lay, &path))
    }

    /// Goto the root of the document.
    fn root(&self) -> Self {
        let mut root = self.clone();
        while let Some((parent, _)) = root.parent() {
            root = parent;
        }
        root
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
            let child_lay = Layouts::compute(child).fit_region(lay.region);
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

fn loc_cursor<Doc>(doc: &Doc, lay: &LayoutRegion, path: &[usize]) -> Region
    where Doc: PrettyDocument
{
    match path {
        [] => lay.region,
        [i, path..] => {
            let child_region = lay.find_child(*i)
                .expect("PrettyDocument::locate_cursor - got lost looking for cursor")
                .region;
            let child_lay = Layouts::compute(&doc.child(*i)).fit_region(child_region);
            loc_cursor(&doc.child(*i), &child_lay, path)
        }
    }
}
