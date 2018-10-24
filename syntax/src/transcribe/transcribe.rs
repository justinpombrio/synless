use std::fmt;

use crate::style::Style;
use crate::geometry::{Pos, Col, Bound};
use crate::syntax::Syntax;
use crate::layout::{LayoutRegion, Layout, BoundSet, Lay, lay_out};

use self::Layout::*;


const DEFAULT_WIDTH: Col = 80;

pub trait Document : Sized + Clone {
    fn arity(&self) -> usize;
    fn parent(&self) -> Option<Self>;
    fn child(&self, i: usize) -> Self;
    fn children(&self) -> Vec<Self>;
    fn syntax(&self) -> &Syntax;
    fn bounds(&self) -> &BoundSet<()>;
    fn text(&self) -> Option<&str>;

    /// Precompute the Bounds within which this document node can be displayed,
    /// given information about its children.
    fn bound(&self) -> BoundSet<()> {
        generic_lay_out(self)
    }

    /// Compute the possible Layouts for this node in the document,
    /// given information about its children.
    fn layouts(&self) -> BoundSet<LayoutRegion> {
        generic_lay_out(self)
    }
}

fn generic_lay_out<T: Document, L: Lay>(this: &T) -> BoundSet<L> {
    let children = this.children();
    let is_empty_text = this.text().is_some() && this.text().unwrap().is_empty();
    let stx = this.syntax().expand(this.arity(), children.len(), is_empty_text);
    let child_bounds = children.iter().map(|child| child.bounds()).collect();
    lay_out(&child_bounds, &stx)
}

pub trait Transcribe {
    fn size(&self) -> Pos;
    fn print_char(&mut self, ch: char, pos: Pos, style: Style);
}

/// Transcribe entire document as plain text.
pub fn transcribe_fmt<T>(f: &mut fmt::Formatter, this: &T) -> fmt::Result
    where T: Document
{
    let width = match f.width() {
        None => DEFAULT_WIDTH,
        Some(width) => width as Col
    };
    let bound = Bound::infinite_scroll(width);
    let lay = this.layouts().fit_bound(bound).1;
    transcribe_fmt_rec(this, f, lay)
}

fn transcribe_fmt_rec<T>(this: &T, f: &mut fmt::Formatter, lay: LayoutRegion) -> fmt::Result
    where T: Document
{
    match lay.layout {
        Empty => {
            Ok(())
        }
        Literal(s, _) => {
            write!(f, "{}", s)
        }
        Text(_) => {
            write!(f, "{}", this.text().expect("Expected text while transcribing; found branch node"))
        }
        Flush(box inner_lay) => {
            transcribe_fmt_rec(this, f, inner_lay)?;
            write!(f, "\n")?;
            let indent = lay.region.beginning().col;
            write!(f, "{}", " ".repeat(indent as usize))
        }
        Child(i) => {
            let lay = this.child(i).layouts().fit_bound(lay.region.bound).1;
            transcribe_fmt_rec(&this.child(i), f, lay)
        }
        Concat(box lay1, box lay2) => {
            transcribe_fmt_rec(this, f, lay1)?;
            transcribe_fmt_rec(this, f, lay2)
        }
    }
}
