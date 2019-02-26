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

    // TODO: There can be only one. Rename `render` to `pretty_print` and delete this.
    /// Pretty-print entire document.
    fn pretty_print<Screen>(&self, screen: &mut Screen) -> Result<(), Screen::Error>
    where
        Screen: PrettyScreen,
    {
        let bound = Bound::infinite_scroll(screen.region()?.width());
        let lay = Layouts::compute(&self.root()).fit_bound(bound);
        pp(self, screen, lay)
    }

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
    fn render<Screen>(&self, width: Col, screen: &mut Screen) -> Result<(), Screen::Error>
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

// TODO: shading and highlighting
fn pp<Doc, Screen>(doc: &Doc, screen: &mut Screen, lay: LayoutRegion) -> Result<(), Screen::Error>
where
    Screen: PrettyScreen,
    Doc: PrettyDocument,
{
    match lay.layout {
        Empty => Ok(()),
        Literal(text, style) => screen.print(lay.region.pos, text.as_ref(), style),
        Text(style) => {
            let text = doc
                .text()
                .expect("PrettyDocument::pretty_print - Expected text, found branch node");
            screen.print(lay.region.pos, text.as_ref(), style)
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
            pp(child, screen, child_lay)
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
    let pos = Pos {
        row: text_region.pos.row,
        col: cmp::min(text_region.pos.col, screen_region.pos.col),
    };
    let offset = screen_region.pos.col.saturating_sub(text_region.pos.col);
    let string_offset = text
        .char_indices()
        .nth(offset as usize)
        .expect("PrettyDocument::render - issue with string offset")
        .0;
    screen.print(pos, &text[string_offset..], style)
}
