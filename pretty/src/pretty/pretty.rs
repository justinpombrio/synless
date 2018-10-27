use std::fmt;

use crate::style::{Style, Shade};
use crate::geometry::{Pos, Col, Bound, Region};
use crate::notation::Notation;
use crate::layout::{LayoutRegion, Layout, BoundSet, Lay, lay_out};

use self::Layout::*;


const DEFAULT_WIDTH: Col = 80;

pub trait PrettyDocument : Sized + Clone {
    fn arity(&self) -> usize;
    fn parent(&self) -> Option<Self>;
    fn child(&self, i: usize) -> Self;
    fn children(&self) -> Vec<Self>;
    fn notation(&self) -> &Notation;
    fn text(&self) -> Option<&str>;

    // TODO: have this return a reference instead?
    /// Compute the Bounds within which this document node can be displayed,
    /// given information about its children. **For efficiency, you should
    /// re-implement this method, and cache the bounds every time the document
    /// changes.**
    fn bounds(&self) -> Bounds {
        compute_bounds(self)
    }
}

pub trait PrettyScreen {
    fn size(&self) -> Bound;

    fn begin(&mut self);
    fn end(&mut self);

    fn empty(&mut self, info: PrettyInfo);
    fn literal(&mut self, info: PrettyInfo, text: &str, style: Style);
    fn text(&mut self, info: PrettyInfo, text: &str, style: Style, active_char: Option<usize>);
    fn horz(&mut self, info: PrettyInfo, left: Box<FnOnce()>, right: Box<FnOnce()>);
    fn vert(&mut self, info: PrettyInfo, left: Box<FnOnce()>, right: Box<FnOnce()>);
}

pub struct PrettyInfo {
    depth: Option<usize>,
    region: Region
}

#[derive(Clone)]
pub struct Bounds(BoundSet<()>);

/// Compute the possible bounds of this node. This is required in order to
/// pretty-print it. Note that:
///
/// 1. This depends on the Notation of this node, plus the Bounds of its
/// (immediate) children.
/// 2. This _does not_ depend on the width with which the document will be
/// pretty-printed.
pub fn compute_bounds<Doc: PrettyDocument>(doc: &Doc) -> Bounds {
    Bounds(generic_lay_out(doc))
}

fn generic_lay_out<Doc, L>(doc: &Doc) -> BoundSet<L>
    where Doc: PrettyDocument, L: Lay
{
    let children = doc.children();
    let is_empty_text = doc.text().is_some() && doc.text().unwrap().is_empty();
    let stx = doc.notation().expand(doc.arity(), children.len(), is_empty_text);
    let child_bounds = children.iter().map(|child| child.bounds().0).collect();
    lay_out(&child_bounds, &stx)
}


/// Pretty-print entire document as plain text.
pub fn pretty_print<Doc>(f: &mut fmt::Formatter, doc: &Doc) -> fmt::Result
    where Doc: PrettyDocument
{
    let width = match f.width() {
        None => DEFAULT_WIDTH,
        Some(width) => width as Col
    };
    let bound = Bound::infinite_scroll(width);
    let lay = generic_lay_out(doc).fit_bound(bound).1;
    pretty_print_rec(doc, f, lay)
}

fn pretty_print_rec<Doc>(doc: &Doc, f: &mut fmt::Formatter, lay: LayoutRegion) -> fmt::Result
    where Doc: PrettyDocument
{
    match lay.layout {
        Empty => {
            Ok(())
        }
        Literal(s, _) => {
            write!(f, "{}", s)
        }
        Text(_) => {
            write!(f, "{}", doc.text().expect("Expected text while transcribing; found branch node"))
        }
        Child(i) => {
            let lay = generic_lay_out(&doc.child(i)).fit_bound(lay.region.bound).1;
            pretty_print_rec(&doc.child(i), f, lay)
        }
        Horz(box lay1, box lay2) => {
            pretty_print_rec(doc, f, lay1)?;
            pretty_print_rec(doc, f, lay2)
        }
        Vert(box lay1, box lay2) => {
            let indent = lay.region.pos.col;
            pretty_print_rec(doc, f, lay1)?;
            write!(f, "\n{}", " ".repeat(indent as usize))?;
            pretty_print_rec(doc, f, lay2)
        }
    }
}
