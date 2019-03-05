use std::cmp;

use self::Layout::*;
use super::pretty_screen::PrettyScreen;
use crate::geometry::{Bound, Col, Pos, Region};
use crate::layout::{
    compute_bounds, compute_layouts, text_bounds, Bounds, Layout, LayoutRegion, Layouts,
};
use crate::notation::Notation;
use crate::style::Style;

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
    /// The node's notation.
    fn notation(&self) -> &Notation;
    /// If the node contains text, that text. Otherwise `None`.
    fn text(&self) -> Option<Self::TextRef>;

    // TODO: have this return a reference instead?
    /// Get the Bounds within which this document node can be displayed,
    /// given information about its children. This can be computed via
    /// `Bounds::compute`. **However, it is a potentially expensive operation
    /// (at least when applied over the whole document), so for efficiency you
    /// should re-compute it only when the document is edited, and cache the
    /// result.**
    fn bounds(&self) -> Bounds;

    /// Render the document onto the screen. This method behaves as if it did
    /// the following:
    ///
    /// 1. Pretty-print the entire document with width `width`.
    /// 2. Transcribe the portion of the document under the screen onto the
    /// screen. (Remember that the screen is located at `screen.region()`.)
    ///
    /// However, this method is more efficient than that, and does an amount of
    /// work that is (more or less) proportional to the size of the screen,
    /// regardless of the size of the document.
    fn pretty_print<Screen>(&self, width: Col, screen: &mut Screen) -> Result<(), Screen::Error>
    where
        Screen: PrettyScreen,
    {
        let root = self.root();
        let bound = Bound::infinite_scroll(width);
        let lay = Layouts::compute(&root).fit_bound(bound);
        render(&root, screen, screen.region()?, &lay)
    }

    /// Find the region covered by this sub-document, when the entire document is
    /// pretty-printed with the given `width`.
    fn locate_cursor<Screen>(&self, width: Col) -> Result<Region, Screen::Error>
    where
        Screen: PrettyScreen,
    {
        // Find the root of the Document, and the path from the root to the
        // selected node.
        let mut path = vec![];
        let mut root = self.clone();
        while let Some((parent, i)) = root.parent() {
            root = parent;
            path.push(i);
        }
        path.reverse();
        // Recursively compute the cursor region.
        let lay = Layouts::compute(&root).fit_bound(Bound::infinite_scroll(width));
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

impl Bounds {
    /// _Compute_ the possible bounds of this node. This is required in order to
    /// pretty-print it. Note that:
    ///
    /// 1. This depends on the Notation of this node, plus the Bounds of its
    /// (immediate) children.
    /// 2. This _does not_ depend on the width with which the document will be
    /// pretty-printed.
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
        Some(text) => vec![text_bounds(text.as_ref())],
    }
}

fn expanded_notation<Doc: PrettyDocument>(doc: &Doc) -> Notation {
    let len = match doc.text() {
        None => doc.children().len(),
        Some(text) => text.as_ref().chars().count(),
    };
    doc.notation().expand(len)
}

fn loc_cursor<Doc>(doc: &Doc, lay: &LayoutRegion, path: &[usize]) -> Region
where
    Doc: PrettyDocument,
{
    match path {
        [] => lay.region,
        [i, path..] => {
            let child_region = lay
                .find_child(*i)
                .expect("PrettyDocument::locate_cursor - got lost looking for cursor")
                .region;
            let child_lay = Layouts::compute(&doc.child(*i)).fit_region(child_region);
            loc_cursor(&doc.child(*i), &child_lay, path)
        }
    }
}

// TODO: shading and highlighting
fn render<Doc, Screen>(
    doc: &Doc,
    screen: &mut Screen,
    screen_region: Region,
    lay: &LayoutRegion,
) -> Result<(), Screen::Error>
where
    Doc: PrettyDocument,
    Screen: PrettyScreen,
{
    if !lay.region.overlaps(screen_region) {
        // It's entirely offscreen. Nothing to show.
        return Ok(());
    }
    match &lay.layout {
        Empty => Ok(()),
        Literal(text, style) => {
            render_text(text.as_ref(), lay.region, screen, screen_region, *style)
        }
        Text(style) => {
            let text = doc
                .text()
                .expect("PrettyDocument::render - Expected text, found branch node");
            render_text(text.as_ref(), lay.region, screen, screen_region, *style)
        }
        Child(i) => {
            let child = &doc.child(*i);
            let child_lay = Layouts::compute(child).fit_region(lay.region);
            render(child, screen, screen_region, &child_lay)
        }
        Concat(box lay1, box lay2) => {
            render(doc, screen, screen_region, &lay1)?;
            render(doc, screen, screen_region, &lay2)
        }
        Horz(box lay1, box lay2) => {
            render(doc, screen, screen_region, &lay1)?;
            render(doc, screen, screen_region, &lay2)
        }
        Vert(box lay1, box lay2) => {
            render(doc, screen, screen_region, &lay1)?;
            render(doc, screen, screen_region, &lay2)
        }
    }
}

fn render_text<Screen>(
    text: &str,
    text_region: Region,
    screen: &mut Screen,
    screen_region: Region,
    style: Style,
) -> Result<(), Screen::Error>
where
    Screen: PrettyScreen,
{
    assert!(
        screen_region.is_rectangular(),
        "screen region must be rectangular"
    );

    let start_char = screen_region.pos.col.saturating_sub(text_region.pos.col);
    let end_char = cmp::min(
        text_region.width(),
        screen_region.end().col - text_region.pos.col,
    );

    let mut chars = text.char_indices();

    let start_byte = chars
        .nth(start_char as usize)
        .expect("PrettyDocument::render - issue with string indexing")
        .0;

    let end_byte = chars
        .nth((end_char - start_char - 1) as usize)
        .map(|x| x.0)
        .unwrap_or(text.len());

    let screen_offset = Pos {
        row: text_region.pos.row - screen_region.pos.row,
        col: text_region.pos.col.saturating_sub(screen_region.pos.col),
    };
    screen.print(screen_offset, &text[start_byte..end_byte], style)
}
