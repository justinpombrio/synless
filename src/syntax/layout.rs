use std::fmt;

use geometry::*;
use style::Style;
use syntax::syntax::Syntax;
use syntax::BoundSet;
use self::BasicLayout::*;


#[derive(Clone)]
pub enum BasicLayout {
    Literal(String, Style),
    Text(Style, Bound),
    Flush(Box<BasicLayout>),
    Concat(Box<BasicLayout>, Box<BasicLayout>),
    Child(usize, Bound)
}

#[derive(Clone)]
struct BoundedLayout {
    bound: Bound,
    layout: BasicLayout
}

impl BasicLayout {
    fn debug_print(&self, f: &mut fmt::Formatter, indent: Col)
        -> Result<Col, fmt::Error>
    {
        match self {
            &Literal(ref s, _) => {
                write!(f, "{}", s)?;
                Ok(indent + s.chars().count() as Col)
            }
            &Text(_, bound) => {
                bound.debug_print(f, 't', indent)?;
                Ok(indent + bound.indent)
            }
            &Flush(ref lay)  => {
                lay.debug_print(f, indent)?;
                write!(f, "\n")?;
                write!(f, "{}", " ".repeat(indent as usize))?;
                Ok(indent)
            }
            &Child(index, bound) => {
                let ch = format!("{}", index).pop().unwrap();
                bound.debug_print(f, ch, indent)?;
                Ok(indent + bound.indent)
            }
            &Concat(ref lay1, ref lay2) => {
                let indent = lay1.debug_print(f, indent)?;
                lay2.debug_print(f, indent)
            }
        }
    }
}

impl fmt::Debug for BasicLayout{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.debug_print(f, 0)?;
        Ok(())
    }
}


impl BoundedLayout {
    fn dominates(&self, other: &BoundedLayout) -> bool {
        self.bound.dominates(other.bound)
    }

    fn child(h: usize, bound: Bound) -> BoundedLayout {
        BoundedLayout{
            bound: bound,
            layout: Child(h, bound)
        }
    }

    fn literal(s: &str, style: Style) -> BoundedLayout {
        BoundedLayout{
            bound: Bound::literal(s),
            layout: Literal(s.to_string(), style)
        }
    }

    fn text(style: Style, bound: Bound) -> BoundedLayout {
        BoundedLayout{
            bound: bound,
            layout: Text(style, bound)
        }
    }

    fn flush(&self) -> BoundedLayout {
        BoundedLayout{
            bound: self.bound.flush(),
            layout: Flush(Box::new(self.layout.clone()))
        }
    }

    fn concat(&self, other: &BoundedLayout) -> BoundedLayout {
        BoundedLayout{
            bound: self.bound.concat(other.bound),
            layout: Concat(Box::new(self.layout.clone()),
                           Box::new(other.layout.clone()))
        }
    }
}

/// A set of Layouts. If one is strictly smaller than another, only
/// the smaller one will be kept.
#[derive(Clone)]
pub struct LayoutSet {
    layouts: Vec<BoundedLayout>
}

impl LayoutSet {
    fn new() -> LayoutSet {
        LayoutSet{
            layouts: vec!()
        }
    }

    fn singleton(layout: BoundedLayout) -> LayoutSet {
        let mut set = LayoutSet::new();
        set.insert(layout);
        set
    }

    /// Filter out Layouts that don't fit within the given width.
    /// Panics if none are left.
    #[cfg(test)]
    pub(crate) fn fit_width(&self, width: Col) -> BasicLayout {
        let layout = self.layouts.iter().filter(|layout| {
            layout.bound.width <= width
        }).nth(0);
        match layout {
            Some(layout) => layout.layout.clone(),
            None => panic!("No layout fits within given width")
        }
    }

    /// Pick the best (i.e., smallest) layout that fits within the
    /// given Bound. Panics if none fit.
    pub(crate) fn pick(&self, bound: Bound) -> BasicLayout {
        let layout = self.layouts.iter().filter(|layout| {
            layout.bound.dominates(bound)
        }).nth(0);
        match layout {
            Some(layout) => layout.layout.clone(),
            None => panic!("layout::pick - does not fit in bound")
        }
    }

    // TODO: efficiency (can go from O(n) to O(sqrt(n)))
    // MUST FILTER IDENTICALLY TO BoundSet::insert
    fn insert(&mut self, layout: BoundedLayout) {
        if layout.bound.too_wide() {
            return;
        }
        for l in &self.layouts {
            if l.dominates(&layout) {
                return;
            }
        }
        self.layouts.retain(|l| !layout.dominates(l));
        self.layouts.push(layout);
    }

    fn child(child: usize, set: &BoundSet) -> LayoutSet {
        LayoutSet{
            layouts: set.into_iter().map(|bound| {
                BoundedLayout::child(child, bound)
            }).collect()
        }
    }

    fn text(style: Style, set: &BoundSet) -> LayoutSet {
        LayoutSet{
            layouts: set.into_iter().map(|bound| {
                BoundedLayout::text(style, bound)
            }).collect()
        }
    }

    fn flush(&self) -> LayoutSet {
        let mut set = LayoutSet::new();
        for layout in &self.layouts {
            set.insert(layout.flush())
        }
        set
    }

    fn concat(&self, other: &LayoutSet) -> LayoutSet {
        let mut set = LayoutSet::new();
        for layout1 in &self.layouts {
            for layout2 in &other.layouts {
                set.insert(layout1.concat(layout2))
            }
        }
        set
    }

    fn no_wrap(mut self) -> LayoutSet {
        self.layouts.retain(|layout| {
            layout.bound.height == 0
        });
        self
    }

    fn choice(self, other: LayoutSet) -> LayoutSet {
        let mut set = LayoutSet::new();
        for layout in self.layouts {
            set.insert(layout);
        }
        for layout in other.layouts {
            set.insert(layout);
        }
        set
    }
}


impl Syntax {
    /// Compute all possible layouts for a Syntax, given its arity and
    /// the Bound of its children.
    pub fn lay_out(&self,
                   arity: usize,
                   child_bounds: &Vec<BoundSet>,
                   empty_text: bool)
                   -> LayoutSet
    {
        self.expand(arity, child_bounds.len(), empty_text)
            .lay(child_bounds)
    }
    
    fn lay(&self, child_bounds: &Vec<BoundSet>) -> LayoutSet {
        match self {
            &Syntax::Literal(ref s, style) => {
                LayoutSet::singleton(BoundedLayout::literal(s, style))
            }
            &Syntax::Text(style) => {
                LayoutSet::text(style, &child_bounds[0])
            }
            &Syntax::Child(h) => {
                LayoutSet::child(h, &child_bounds[h])
            }
            &Syntax::Flush(ref syn) => {
                let lay = syn.lay(child_bounds);
                lay.flush()
            }
            &Syntax::Concat(ref syn1, ref syn2) => {
                let lay1 = syn1.lay(child_bounds);
                let lay2 = syn2.lay(child_bounds);
                lay1.concat(&lay2)
            }
            &Syntax::NoWrap(ref syn) => {
                let lay = syn.lay(child_bounds);
                lay.no_wrap()
            }
            &Syntax::Choice(ref syn1, ref syn2) => {
                let lay1 = syn1.lay(child_bounds);
                let lay2 = syn2.lay(child_bounds);
                lay1.choice(lay2)
            }
            &Syntax::IfEmptyText(_, _) => panic!("lay_out: unexpected Syntax::IfEmptyText"),
            &Syntax::Rep(_) => panic!("lay_out: unexpected Syntax::Repeat"),
            &Syntax::Star   => panic!("lay_out: unexpected Syntax::Star"),
        }
    }
}
