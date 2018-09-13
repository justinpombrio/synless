use std::fmt;

use style::Style;
use syntax::syntax::Syntax;
use self::Layout::*;


/// A concrete plan for how to lay out the `Syntax`, once the program
/// and screen width are known.  For example, unlike in `Syntax`,
/// there is no Choice, because the choices have been resolved.
/// The outermost region always has position zero, but inner regions
/// are relative to this.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayoutRegion {
    pub layout: Layout,
    pub region: Region
}

/// The enum for a LayoutRegion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Layout {
    /// Display a literal string with the given style.
    Literal(String, Style),
    /// Display a text node's text with the given style.
    Text(Style),
    /// Display the layout, then a newline.
    Flush(Box<LayoutRegion>),
    /// Display the concatenation of the two layouts.
    /// The `Col` is the indent on the Bound of the first Layout.
    /// (This is redundant information, but convenient to have around.)
    Concat(Box<LayoutRegion>, Box<LayoutRegion>),
    /// Display a child node. Its Bound must be supplied.
    Child(usize)
}


impl Syntax {
    // TODO: save this info from `find_bounds`.
    /// Compute all possible layouts for a Syntax, given its arity and
    /// the Bound of its children.
    pub fn lay_out(&self,
                   arity: usize,
                   child_bounds: &Vec<BoundSet>,
                   is_empty_text: bool,
                   region: Region)
                   -> Layout
    {
        self.expand(arity, child_bounds.len(), is_empty_text)
            .lay(child_bounds)
    }
    
    fn lay(&self, child_bounds: &Vec<BoundSet>, region: Region) -> LayoutRegion {
        match self {
            Syntax::Literal(s, &style) => {
                LayoutRegion {
                    layout: Literal(s.clone(), style), // TODO: remove clone?
                    region: Region {
                        pos: region.pos,
                        bound: Bound::literal(s)
                    }
                }
            }
            Syntax::Concat(syn1, syn2) => {
                let lay1 = syn1.lay(child_bounds, region);
                let bound2 = region.subregion_from(lay1.region.end());
                let lay2 = syn2.lay(child_bounds, region2);
                LayoutRegion {
                    layout: Concat(Box::new(lay1), Box::new(lay2)),
                    region: region.subregion_to(lay2.region.end())
                }
            }
            Syntax::Choice(syn1, syn2) => {
                let lay1 = syn1.lay(child_bounds, region);
                let lay2 = syn2.lay(child_bounds, region);
                
                lay1.choice(lay2)

