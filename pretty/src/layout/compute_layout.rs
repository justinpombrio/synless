use super::boundset::BoundSet;
use super::compute_bounds::compute_bounds;
use super::notation_ops::{NotationOps, ResolvedNotation};
use crate::geometry::{Bound, Col, Pos, Region};
use crate::notation::Notation;
use crate::style::Style;
use std::iter;
use ResolvedNotation::*;

/// A concrete plan for how to lay out the `Notation`, once the program
/// and screen width are known.
#[derive(Clone, PartialEq, Eq)]
pub struct Layout {
    /// Everything to display on the screen.
    pub elements: Vec<LayoutElement>,
    /// Just the `Child` elements, organized by index, for easy access. Is
    /// `None` if that child is not being displayed.
    pub children: Vec<Option<LayoutElement>>,
}

/// Part of a [`Layout`](Layout): one thing to be written to the screen.
#[derive(Clone, PartialEq, Eq)]
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

/// Lay out a node in preparation for rendering it. Doing this requires knowing
///
/// 1. The Notation with which it is being displayed.
/// 2. The upper-left position to lay out at
/// 3. The available screen width.
/// 4. The Bounds of the Node's children.
/// 5. Whether the node is empty text
pub fn compute_layout(
    notation: &Notation,
    pos: Pos,
    width: Col,
    child_bounds: &[&BoundSet<()>],
    is_empty_text: bool,
) -> Layout {
    let boundset = compute_bounds(notation, child_bounds, is_empty_text);
    let (_, resolved_notation) = boundset.fit_width(width);
    let mut computer = ComputeLayout::new(child_bounds.len());
    computer.lay_out(resolved_notation, pos);
    computer.0
}

struct ComputeLayout(Layout);

impl ComputeLayout {
    fn new(num_children: usize) -> ComputeLayout {
        ComputeLayout(Layout {
            elements: vec![],
            children: iter::repeat(None).take(num_children).collect(),
        })
    }

    fn lay_out(&mut self, notation: &ResolvedNotation, pos: Pos) -> Bound {
        let mut pos = pos;
        match notation {
            Empty => Bound::empty(),
            Literal(string, style, bound) => {
                let region = Region { pos, bound: *bound };
                let element = LayoutElement::Literal(region, string.to_string(), *style);
                self.0.elements.push(element);
                *bound
            }
            Follow(left, right) => {
                let left_bound = self.lay_out(left, pos);
                pos = pos + left_bound.end();
                let right_bound = self.lay_out(right, pos);
                Bound::follow(left_bound, right_bound)
            }
            Vert(left, right) => {
                let left_bound = self.lay_out(left, pos);
                pos.row += left_bound.height;
                let right_bound = self.lay_out(right, pos);
                Bound::vert(left_bound, right_bound)
            }
            Text(style, bound) => {
                let region = Region { pos, bound: *bound };
                let element = LayoutElement::Text(region, *style);
                self.0.elements.push(element.clone());
                self.0.children[0] = Some(element);
                *bound
            }
            Child(i, bound) => {
                let region = Region { pos, bound: *bound };
                let element = LayoutElement::Child(region, *i);
                self.0.elements.push(element.clone());
                self.0.children[*i] = Some(element);
                *bound
            }
        }
    }
}
