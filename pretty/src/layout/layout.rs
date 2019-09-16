use super::bounds::Bounds;
use crate::geometry::{Bound, Col, Pos, Region};
use crate::notation::{Notation, NotationOps, RepeatInner};
use crate::style::Style;
use std::fmt;
use std::iter;

use Notation::*;

#[derive(Clone)]
/// A concrete plan for how to lay out a `Notation`, once the program
/// and screen width are known.
pub struct Layout {
    elements: Vec<LayoutElement>,
    children: Vec<Option<LayoutElement>>,
}

impl Layout {
    pub fn elements(&self) -> impl Iterator<Item = &LayoutElement> {
        self.elements.iter()
    }

    pub fn child(&self, i: usize) -> Option<&LayoutElement> {
        self.children[i].as_ref()
    }
}

/// Lay out a node in preparation for rendering it. Doing this requires knowing
///
/// 1. The Notation with which it is being displayed.
/// 2. The upper-left position to lay out at
/// 3. The available screen width.
/// 4. The Bounds of the node.
/// 5. The Bounds of each of its children.
/// 6. Whether it is empty text
pub fn compute_layout(
    notation: &Notation,
    pos: Pos,
    width: Col,
    child_bounds: &[Bounds],
    is_empty_text: bool,
) -> Layout {
    let mut computer = ComputeLayout::new(child_bounds, is_empty_text);
    computer.lay_out(notation, pos, width, None);
    computer.layout
}

/// Part of a [`Layout`](Layout): one thing to be written to the screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutElement {
    /// Display a literal string with the given style.
    Literal(Region, String, Style),
    /// Display a text node's text with the given style.
    Text(Region, Style),
    /// Display a child node. Its Bound must be supplied.
    Child(Region, usize),
}

impl LayoutElement {
    pub fn region(&self) -> Region {
        match self {
            LayoutElement::Literal(region, _, _) => *region,
            LayoutElement::Text(region, _) => *region,
            LayoutElement::Child(region, _) => *region,
        }
    }
}

struct ComputeLayout<'a> {
    layout: Layout,
    child_bounds: &'a [Bounds],
    is_empty_text: bool,
}

impl ComputeLayout<'_> {
    fn new(child_bounds: &[Bounds], is_empty_text: bool) -> ComputeLayout {
        ComputeLayout {
            layout: Layout {
                elements: vec![],
                children: iter::repeat(None).take(child_bounds.len()).collect(),
            },
            child_bounds,
            is_empty_text,
        }
    }

    fn lay_out(
        &mut self,
        notation: &Notation,
        pos: Pos,
        width: Col,
        in_repeat: Option<(&Notation, usize)>,
    ) -> Bound {
        match notation {
            Empty => Bound::empty(),
            Literal(string, style) => {
                let bound = Bound::literal(string);
                let region = Region { pos, bound };
                let element = LayoutElement::Literal(region, string.to_string(), *style);
                self.layout.elements.push(element);
                bound
            }
            Text(style) => {
                let bound = self.child_bounds[0].fit_width(width);
                let region = Region { pos, bound };
                let element = LayoutElement::Text(region, *style);
                self.layout.elements.push(element.clone());
                self.layout.children[0] = Some(element);
                bound
            }
            Child(i) => {
                let bound = self.child_bounds[*i].fit_width(width);
                let region = Region { pos, bound };
                let element = LayoutElement::Child(region, *i);
                self.layout.elements.push(element.clone());
                self.layout.children[*i] = Some(element);
                bound
            }
            Nest(notations) => {
                let mut pos = pos;
                let mut width = width;
                let mut total_bound = Bound::empty();
                for notation in notations {
                    let bound = self.lay_out(notation, pos, width, in_repeat);
                    pos = pos + bound.end();
                    width = width - bound.indent;
                    total_bound = Bound::nest(total_bound, bound);
                }
                total_bound
            }
            Vert(notations) => {
                // Assume the notation has been normalized, so `notations.len()` >= 2.
                let mut notations = notations.iter();
                let mut total_bound =
                    self.lay_out(notations.next().unwrap(), pos, width, in_repeat);
                let mut pos = pos;
                pos.row += total_bound.height;
                for notation in notations {
                    let bound = self.lay_out(notation, pos, width, in_repeat);
                    pos.row += bound.height;
                    total_bound = Bound::vert(total_bound, bound);
                }
                total_bound
            }
            IfFlat(left, right, flat_width) => {
                if flat_width.is_some() && flat_width.unwrap() <= width {
                    self.lay_out(left, pos, width, in_repeat)
                } else {
                    self.lay_out(right, pos, width, in_repeat)
                }
            }
            IfEmptyText(left, right) => {
                if self.is_empty_text {
                    self.lay_out(left, pos, width, in_repeat)
                } else {
                    self.lay_out(right, pos, width, in_repeat)
                }
            }
            Repeat(box RepeatInner {
                empty,
                lone,
                join,
                surround,
            }) => match self.child_bounds.len() {
                0 => self.lay_out(empty, pos, width, None),
                1 => self.lay_out(lone, pos, width, None),
                n => self.lay_out(surround, pos, width, Some((&join, n))),
            },
            Surrounded => {
                let (join, i) = in_repeat.expect("Exposed `Surround`");
                self.lay_out(join, pos, width, Some((join, i - 1)))
            }
            Left => {
                let (join, i) = in_repeat.expect("Exposed `Left`");
                if i == 1 {
                    self.lay_out(&Child(0), pos, width, None)
                } else {
                    self.lay_out(join, pos, width, Some((join, i - 1)))
                }
            }
            Right => {
                let (_join, i) = in_repeat.expect("Exposed `Right`");
                self.lay_out(&Child(i), pos, width, None)
            }
        }
    }
}

struct LayoutDebugPrinter {
    lines: Vec<Vec<char>>,
}

impl fmt::Debug for Layout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut printer = LayoutDebugPrinter::new();
        for element in &self.elements {
            printer.write_element(element);
        }
        for line in &printer.lines {
            writeln!(f)?;
            for ch in line {
                write!(f, "{}", ch)?;
            }
        }
        Ok(())
    }
}

impl LayoutDebugPrinter {
    fn new() -> LayoutDebugPrinter {
        LayoutDebugPrinter { lines: vec![] }
    }

    fn write_char(&mut self, ch: char, pos: Pos) {
        while pos.row as usize >= self.lines.len() {
            self.lines.push(vec![]);
        }
        let line = &mut self.lines[pos.row as usize];
        while pos.col as usize >= line.len() {
            line.push(' ');
        }
        line[pos.col as usize] = ch;
    }

    fn write_element(&mut self, element: &LayoutElement) {
        match element {
            LayoutElement::Literal(region, string, _) => {
                let mut pos = region.beginning();
                for ch in string.chars() {
                    self.write_char(ch, pos);
                    pos.col += 1;
                }
            }
            LayoutElement::Text(region, _) => {
                for pos in region.positions() {
                    self.write_char('0', pos);
                }
            }
            LayoutElement::Child(region, i) => {
                for pos in region.positions() {
                    self.write_char((('0' as u8) + (*i as u8)).into(), pos);
                }
            }
        }
    }
}
