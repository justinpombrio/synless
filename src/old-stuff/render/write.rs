use std::fmt;
use std::fmt::Write;

use geometry::*;
use tree::TreeRef;
use syntax::{Bound, LayoutRegion};
use syntax::Layout::*;


impl<'l, 't> TreeRef<'l, 't> {
    pub fn write(self, width: Col) -> String {
        let mut s = String::new();
        write_doc(&mut s, self, width).unwrap();
        s
    }
}
// (tested in syntax/mod.rs)

fn write_doc(f: &mut Write, doc: TreeRef, width: Col) -> fmt::Result {
    let bound = Bound::infinite_scroll(width);
    let region = Region{ bound: bound, pos: Pos::zero() };
    write_tree(f, doc, region)
}

fn write_tree(f: &mut Write, tree: TreeRef, region: Region) -> fmt::Result {
    let layout = tree.lay_out(region);
    write_layout(f, tree, layout)
}

fn write_layout(f: &mut Write, tree: TreeRef, lay: LayoutRegion) -> fmt::Result {
    match lay.layout {
        Literal(s, _) => {
            write!(f, "{}", s)
        }
        Text(_) => {
            write!(f, "{}", tree.text())
        }
        Flush(box inner_lay) => {
            write_layout(f, tree, inner_lay)?;
            write!(f, "\n")?;
            let indent = lay.region.beginning().col;
            write!(f, "{}", " ".repeat(indent as usize))
        }
        Child(index) => {
            write_tree(f, tree.child(index), lay.region)
        }
        Concat(box layout1, box layout2) => {
            write_layout(f, tree.clone(), layout1)?;
            write_layout(f, tree, layout2)
        }
    }
}
