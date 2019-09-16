use super::pretty_doc::{CursorVisibility, PrettyDocument, ScrollApproach};
use super::pretty_window::PrettyWindow;
use super::viewport::Viewport;
use crate::geometry::{Col, Pos, Rect, Region};
use crate::layout::{Layout, LayoutElement};
use crate::style::Shade;

/// Render the document onto a `PrettyWindow`. This function behaves as if it did
/// the following:
///
/// 1. Pretty-print the entire document with width `width`.
/// 2. Position the document under the `PrettyWindow`, aligning `doc_pos` with
/// the `PrettyWindow`'s upper left corner.
/// 3. Render the portion of the document under the `PrettyWindow` onto the
/// `PrettyWindow`.
///
/// However, this function is more efficient than that, and does an amount of
/// work that is approximately proportional to the size of the `PrettyWindow`,
/// regardless of the size of the document.
fn pretty_print<D, W, E>(
    doc: D,
    window: &mut W,
    window_rect: Rect,
    scroll_approach: ScrollApproach,
    cursor_visibility: CursorVisibility,
) -> Result<(), W::Error>
where
    D: PrettyDocument,
    W: PrettyWindow,
{
    let cursor_region = locate_cursor(doc.clone(), window_rect.width());
    let mut viewport = Viewport::new(window, &scroll_approach, cursor_region, window_rect);

    // Shade the cursor.
    // TODO handle multiple levels of cursor shading
    if let CursorVisibility::Show = cursor_visibility {
        viewport.shade(cursor_region, Shade(0))?;
    }

    // Render the document.
    render(doc, &mut viewport, Pos::zero(), window_rect.width())
}

fn render<'a, D, W>(
    node: D,
    viewport: &mut Viewport<'a, W>,
    pos: Pos,
    width: Col,
) -> Result<(), W::Error>
where
    D: PrettyDocument,
    W: PrettyWindow,
{
    let layout = Layout::compute(&node, pos, width);
    for element in layout.elements() {
        let region = element.region();
        if !viewport.is_region_visible(region) {
            return Ok(()); // Not much to show!
        }
        match element {
            LayoutElement::Literal(_, string, style) => viewport.print(string, region, *style)?,
            LayoutElement::Text(_, style) => {
                let text = node
                    .text()
                    .expect("PrettyDocument::render - Expected text, found branch node");
                viewport.print(text.as_ref(), region, *style)?;
            }
            LayoutElement::Child(_, i) => {
                render(node.child(*i), viewport, region.pos, region.width())?;
            }
        }
    }
    Ok(())
}

/// Find the region covered by this sub-document, when the entire document is
/// pretty-printed with the given `width`.
fn locate_cursor<D: PrettyDocument>(doc: D, width: Col) -> Region {
    let mut pos = Pos::zero();
    let mut width = width;
    let (mut node, mut path) = find_path_to_cursor(doc.clone());
    while !path.is_empty() {
        let layout = Layout::compute(&mut node, pos, width);
        let i = path.pop().unwrap();
        let element = layout
            .child(i)
            .expect("locate_cursor - got lost looking for cursor");
        pos = element.region().pos;
        width = element.region().width();
        node = node.child(i);
    }
    Region {
        pos,
        bound: node.bounds().fit_width(width),
    }
}

/// Find the root of the Document, and the path from the root to the
/// selected node. The returned path is in reverse (root-most node at end).
fn find_path_to_cursor<D: PrettyDocument>(doc: D) -> (D, Vec<usize>) {
    let mut path = vec![];
    let mut node = doc;
    while let Some((parent, i)) = node.parent() {
        node = parent;
        path.push(i);
    }
    (node, path)
}
