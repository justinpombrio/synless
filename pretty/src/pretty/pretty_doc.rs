use std::cmp;

use self::Layout::*;
use super::pretty_window::PrettyWindow;
use crate::geometry::{Bound, Col, Pos, Rect, Region, Row};
use crate::layout::{
    compute_bounds, compute_layouts, text_bounds, Bounds, Layout, LayoutRegion, Layouts,
};
use crate::notation::Notation;
use crate::pane::{CursorVis, Pane};
use crate::style::{Shade, Style};

/// What part of the document to show.
#[derive(Debug, Clone, Copy)]
pub enum DocPosSpec {
    /// Put this row and column of the document at the top left corner of the Pane.
    Fixed(Pos),
    /// Put the beginning of the document at the top left corner of the
    /// Pane. Equivalent to `Fixed(Pos{row: 0, col: 0})`.
    Beginning,
    /// Position the document such that the top of the cursor is at this height,
    /// where 1 is the top line of the Pane and 0 is the bottom line.
    CursorHeight { fraction: f32 },
}

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

    /// Render the document onto a `Pane`. This method behaves as if it did
    /// the following:
    ///
    /// 1. Pretty-print the entire document with width `width`.
    /// 2. Position the document under the `Pane`, aligning `doc_pos` with the `Pane`'s upper left corner.
    /// 3. Render the portion of the document under the `Pane` onto the `Pane`.
    ///
    /// However, this method is more efficient than that, and does an amount of
    /// work that is (more or less) proportional to the size of the `Pane`,
    /// regardless of the size of the document.
    fn pretty_print<'a, T>(
        &self,
        width: Col,
        pane: &mut Pane<'a, T>,
        doc_pos_spec: DocPosSpec,
        cursor_visibility: CursorVis,
    ) -> Result<(), T::Error>
    where
        T: PrettyWindow,
    {
        let root = self.root();
        let bound = Bound::infinite_scroll(width);
        let lay = Layouts::compute(&root).fit_bound(bound);

        let cursor_region = self.locate_cursor(width);
        let doc_pos = match doc_pos_spec {
            DocPosSpec::CursorHeight { fraction } => {
                let fraction = f32::max(0.0, f32::min(1.0, fraction));
                let offset_from_top =
                    f32::round((pane.rect.height() - 1) as f32 * (1.0 - fraction)) as Row;
                Pos {
                    col: 0,
                    row: u32::saturating_sub(cursor_region.pos.row, offset_from_top),
                }
            }
            DocPosSpec::Fixed(pos) => pos,
            DocPosSpec::Beginning => Pos { row: 0, col: 0 },
        };

        let doc_rect = Rect::new(doc_pos, pane.rect().size());
        render(&root, pane, doc_rect, &lay)?;

        // TODO handle multiple levels of cursor shading
        if let CursorVis::Show = cursor_visibility {
            let region = Region {
                pos: cursor_region.pos - doc_pos,
                ..cursor_region
            };
            pane.highlight(region, Some(Shade(0)), false)?;
        }
        Ok(())
    }

    /// Find the region covered by this sub-document, when the entire document is
    /// pretty-printed with the given `width`.
    fn locate_cursor(&self, width: Col) -> Region {
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
        loc_cursor(&root, &lay, &path)
    }

    /// Find the minimum height required to pretty-print the document with the given width.
    fn required_height(&self, width: Col) -> Row {
        let root = self.root();
        let bound = Bound::infinite_scroll(width);
        let lay = Layouts::compute(&root).fit_bound(bound);
        lay.region.height()
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
        [i, path @ ..] => {
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
fn render<'a, Doc, Win>(
    doc: &Doc,
    pane: &mut Pane<'a, Win>,
    doc_rect: Rect,
    lay: &LayoutRegion,
) -> Result<(), Win::Error>
where
    Doc: PrettyDocument,
    Win: PrettyWindow,
{
    if !lay.region.overlaps_rect(doc_rect) {
        // It's entirely offscreen. Nothing to show.
        return Ok(());
    }
    match &lay.layout {
        Empty => Ok(()),
        Literal(text, style) => render_text(text.as_ref(), lay.region, pane, doc_rect, *style),
        Text(style) => {
            let text = doc
                .text()
                .expect("PrettyDocument::render - Expected text, found branch node");
            render_text(text.as_ref(), lay.region, pane, doc_rect, *style)
        }
        Child(i) => {
            let child = &doc.child(*i);
            let child_lay = Layouts::compute(child).fit_region(lay.region);
            render(child, pane, doc_rect, &child_lay)
        }
        Concat(box lay1, box lay2) => {
            render(doc, pane, doc_rect, &lay1)?;
            render(doc, pane, doc_rect, &lay2)
        }
        Horz(box lay1, box lay2) => {
            render(doc, pane, doc_rect, &lay1)?;
            render(doc, pane, doc_rect, &lay2)
        }
        Vert(box lay1, box lay2) => {
            render(doc, pane, doc_rect, &lay1)?;
            render(doc, pane, doc_rect, &lay2)
        }
    }
}

fn render_text<'a, Win>(
    text: &str,
    text_region: Region,
    pane: &mut Pane<'a, Win>,
    doc_rect: Rect,
    style: Style,
) -> Result<(), Win::Error>
where
    Win: PrettyWindow,
{
    if text.is_empty() {
        return Ok(()); // not much to show!
    }

    let start_char = doc_rect.pos().col.saturating_sub(text_region.pos.col);
    let end_char = cmp::min(text_region.width(), doc_rect.cols.1 - text_region.pos.col);
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
        row: text_region.pos.row - doc_rect.pos().row,
        col: text_region.pos.col.saturating_sub(doc_rect.pos().col),
    };
    pane.print(screen_offset, &text[start_byte..end_byte], style)
}
