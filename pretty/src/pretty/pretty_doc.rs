use std::cmp;
use typed_arena::Arena;

use super::pretty_window::PrettyWindow;
use crate::geometry::{Col, Pos, Rect, Region, Row, Bound};
use crate::layout::{compute_bounds, compute_layout, BoundSet, Layout, LayoutElement, NotationOps, ResolvedNotation};
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
        let owned_arena = Arena::new();
        let arena = &owned_arena;
        {
            let root = self.root();
            let layout = layout(&root, Pos::zero(), width, arena);
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
            render(&root, pane, doc_rect, &layout, arena)?;

            // TODO handle multiple levels of cursor shading
            if let CursorVis::Show = cursor_visibility {
                let region = Region {
                    pos: cursor_region.pos - doc_pos,
                    ..cursor_region
                };
                pane.highlight(region, Some(Shade(0)), false)?;
            }
        }
        Ok(())
    }

    /// Find the region covered by this sub-document, when the entire document is
    /// pretty-printed with the given `width`.
    fn locate_cursor(&self, width: Col) -> Region {
        // TODO use the same arena as pretty_print()?
        let arena = Arena::new();

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
        let bound = root.bounds().fit_width(width);
        let region = Region {
            pos: Pos::zero(),
            bound,
        };
        loc_cursor(&root, region, &path, &arena)
    }

    /// Find the minimum height required to pretty-print the document with the given width.
    fn required_height(&self, width: Col) -> Row {
        let root = self.root();
        root.bounds().fit_width(width).height
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

/// Every node must keep an up-to-date `Bounds`, computed using
/// [`compute_bounds`](compute_bounds). It contains pre-computed information
/// that helps pretty-print a document.
#[derive(Debug, Clone)]
pub struct Bounds(BoundSet<()>);

impl Bounds {
    /// _Compute_ the possible bounds of this node. This is required in order to
    /// pretty-print it. Note that:
    ///
    /// 1. This depends on the Notation of this node, plus the Bounds of its
    /// (immediate) children.
    /// 2. This _does not_ depend on the width with which the document will be
    /// pretty-printed.
    pub fn compute<Doc: PrettyDocument>(doc: &Doc) -> Bounds {
        let owned_child_bounds = child_bounds(doc);
        let refs: Vec<_> = owned_child_bounds.iter().map(|b| &b.0).collect();
        Bounds(compute_bounds(doc.notation(), &refs, is_empty_text(doc), ()))
    }

    /// Construct an empty, uninitialized `Bounds`. You shouldn't use an
    /// uinitialized `Bounds` for anything! You should use a properly
    /// computed `Bounds` returned by `Bounds::compute()`, instead.
    pub fn uninitialized() -> Bounds {
        Bounds(BoundSet::new())
    }

    /// Pick the best (i.e., smallest) Bound that fits within the
    /// given width. Panics if none fit.
    fn fit_width(&self, width: Col) -> Bound {
        self.0.fit_width(width).0
    }
}

fn child_bounds<Doc: PrettyDocument>(doc: &Doc) -> Vec<Bounds> {
    match doc.text() {
        None => doc.children().iter().map(|child| child.bounds()).collect(),
        Some(text) => vec![Bounds(BoundSet::literal(text.as_ref(), Style::plain(), ()))],
    }
}

fn is_empty_text<Doc: PrettyDocument>(doc: &Doc) -> bool {
    if let Some(text) = doc.text() {
        text.as_ref().is_empty()
    } else {
        false
    }
}

fn layout<'a, Doc: PrettyDocument>(
    doc: &Doc, pos: Pos, width: Col, allocator: &'a Arena<ResolvedNotation<'a>>,
) -> Layout {
    let owned_child_bounds = child_bounds(doc);
    let refs: Vec<_> = owned_child_bounds.iter().map(|b| &b.0).collect();

    compute_layout(
        doc.notation(),
        pos,
        width,
        &refs,
        is_empty_text(doc),
        allocator,
    )
}

fn loc_cursor<'a, Doc>(doc: &Doc, region: Region, path: &[usize], allocator: &'a Arena<ResolvedNotation<'a>>) -> Region
where
    Doc: PrettyDocument,
{
    match path {
        [] => Region {
            pos: region.pos,
            bound: doc.bounds().fit_width(region.width()),
        },
        [i, path @ ..] => {
            let layout = layout(doc, region.pos, region.width(), allocator);
            let child_region = match layout.children.get(*i) {
                Some(Some(element)) => element.region(),
                Some(None) => panic!("PrettyDocument::locate_cursor - cursor is on an invisible node"),
                None => panic!("PrettyDocument::locate_cursor - lost child"),
            };
            loc_cursor(&doc.child(*i), child_region, path, allocator)
        }
    }
}

// TODO: shading and highlighting
fn render<'a, 'arena, Doc, Win>(
    doc: &Doc,
    pane: &mut Pane<'a, Win>,
    doc_rect: Rect,
    layout: &Layout,
    allocator: &'arena Arena<ResolvedNotation<'arena>>,
) -> Result<(), Win::Error>
where
    Doc: PrettyDocument,
    Win: PrettyWindow,
{
    for element in &layout.elements {
        if !element.region().overlaps_rect(doc_rect) {
            // It's entirely offscreen. Nothing to show.
            continue;
        }
        match element {
            LayoutElement::Literal(region, text, style) => {
                render_text(text, *region, pane, doc_rect, *style)?;
            }
            LayoutElement::Text(region, style) => {
                let text = doc
                    .text()
                    .expect("PrettyDocument::render - Expected text, found branch node");
                render_text(text.as_ref(), *region, pane, doc_rect, *style)?;
            }
            LayoutElement::Child(region, index) => {
                let child = &doc.child(*index);
                let owned_grandchild_bounds = child_bounds(child);
                let refs: Vec<_> = owned_grandchild_bounds.iter().map(|b| &b.0).collect();
                let layout = compute_layout(
                    child.notation(),
                    region.pos,
                    region.width(),
                    &refs,
                    is_empty_text(child),
                    allocator,
                );
                render(child, pane, doc_rect, &layout, allocator)?;
            }
        }
    }
    Ok(())
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
        .unwrap_or_else(|| text.len());

    let screen_offset = Pos {
        row: text_region.pos.row - doc_rect.pos().row,
        col: text_region.pos.col.saturating_sub(doc_rect.pos().col),
    };
    pane.print(screen_offset, &text[start_byte..end_byte], style)
}
