use std::fmt;

use crate::style::{Style, Shade};
use crate::geometry::{Pos, Col, Bound, Region};
use crate::layout::{LayoutRegion, Layout, BoundSet, Lay, lay_out};

use self::Layout::*;




/// Pretty-print entire document as plain text.
pub fn pretty_print<Doc>(f: &mut fmt::Formatter, doc: &Doc) -> fmt::Result
    where Doc: PrettyDocument
{
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
