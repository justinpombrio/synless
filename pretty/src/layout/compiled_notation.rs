use crate::geometry::{Col, MAX_WIDTH};
use crate::layout::boundset::BoundSet;
use crate::notation::Notation;
use crate::notation_ops::{apply_notation, NotationOps};
use crate::style::Style;

use self::CompiledNotation::*;

#[derive(Debug, Clone)]
pub enum CompiledNotation {
    Empty,
    Literal(String, Style),
    Text(Style),
    Child(usize),
    Nest(Vec<CompiledNotation>),
    Vert(Vec<CompiledNotation>),
    IfFlat(Box<CompiledNotation>, Box<CompiledNotation>, Option<Col>),
}

pub fn compute_bounds(children: &[Bounds], is_empty_text: bool, notation: &Notation) -> Bounds {
    apply_notation(children, is_empty_text, notation)
}

struct Compiler<'a> {
    children: &'a [Bounds],
    is_empty_text: bool,
}

impl<'a> Compiler<'a> {
    fn compile(&self, notation: &Notation) -> CompiledNotation {
        match notation {
            Notation::Empty => Empty,
            Notation::Literal(string, style) => Literal(string, style),
            Notation::Text(style) => Text(style),
            Notation::Child(i) => Child(i),
            Notation::Nest(notations) => {
                Nest(notations.iter().map(|n| self.compile(n)).collect())
            }
            Notation::Vert(notations) => {
                Vert(notations.iter().map(|n| self.compile(n)).collect())
            }
            Notation::IfFlat(notation1, notation2) => {
                let notation1 = self.compile(notation1);
                let notation2 = self.compile(notation2);
                // flat_width `None` will be filled out later.
                IfFlat(notation1, notation2, None)
            }
            Notation::IfEmptyText(notation1, notation2) => {
                if self.is_empty_text {
                    self.compile(notation1)
                } else {
                    self.compile(notation2)
                }
            }
            Notation::Repeat(box RepeatInner {
                empty,
                lone,
                join,
                surround,
            }) => match self.children.len() {
                0 => self.compile(empty),
                1 => self.compile(lone),
                n => {
                    let mut notation = Child(n-1);
                    for i in (0..n - 1).rev() {
                        let join_result = self.compile(join);
                        let left_result = self.compile(Notation::Child(i));
                        let mut
                        notation = Let(
                            CompiledNotationKey::Left,
                            Box::new(Child(i)),
                            Box::new(Let(
                                CompiledNotationKey::Right,
                                Box::new(notation),
                                Box::new(join),
                            )),
                        );
                    }
                    notation = Let(
                        CompiledNotationKey::Surrounded,
                        Box::new(self.compile(surround).0),
                        Box::new(notation),
                    );
                    (notation,
                }
            },
            _ => unimplemented!(),
            
        }
    }
}

impl NotationOps for Option<Col> {
    fn empty() -> Option<Col> {
        Some(0)
    }

    fn literal(string: &str, _style: Style) -> Option<Col> {
        Some(string.chars().count() as Col)
    }

    fn text(child: &Option<Col>, _style: Style) -> Option<Col> {
        child.clone()
    }

    fn child(children: &[Option<Col>], i: usize) -> Option<Col> {
        children[i].clone()
    }

    fn nest(fw1: Option<Col>, fw2: Option<Col>) -> Option<Col> {
        let fw = fw1? + fw2?;
        if fw <= MAX_WIDTH {
            Some(fw)
        } else {
            None
        }
    }

    fn vert(_fw1: Option<Col>, _fw2: Option<Col>) -> Option<Col> {
        None
    }

    fn if_flat(fw1: Option<Col>, fw2: Option<Col>) -> Option<Col> {
        fw1.or(fw2)
    }
}

#[derive(Debug, Clone)]
pub struct Bounds {
    notation: CompiledNotation,
    flat_width: Option<Col>,
    bound_set: BoundSet,
}

impl Bounds {
    pub(super) fn notation(&self) -> &CompiledNotation {
        &self.notation
    }
}

impl NotationOps for Bounds {
    fn empty() -> Bounds {
        Bounds {
            notation: Empty,
            flat_width: <Option<Col>>::empty(),
            bound_set: BoundSet::empty(),
        }
    }

    fn literal(string: &str, style: Style) -> Bounds {
        Bounds {
            notation: Literal(string.to_string(), style),
            flat_width: <Option<Col>>::literal(string, style),
            bound_set: BoundSet::literal(string, style),
        }
    }

    fn text(child: &Bounds, style: Style) -> Bounds {
        Bounds {
            notation: Text(style),
            flat_width: child.flat_width,
            bound_set: child.bound_set.clone(),
        }
    }

    fn child(children: &[Bounds], i: usize) -> Bounds {
        Bounds {
            notation: Child(i),
            flat_width: children[i].flat_width,
            bound_set: children[i].bound_set.clone(),
        }
    }

    // INEFFICIENT: implement `nests()` instead
    fn nest(result1: Bounds, result2: Bounds) -> Bounds {
        Bounds {
            notation: Nest(vec![result1.notation, result2.notation]),
            flat_width: <Option<Col>>::nest(result1.flat_width, result2.flat_width),
            bound_set: BoundSet::nest(result1.bound_set, result2.bound_set),
        }
    }

    // INEFFICIENT: implement `verts()` instead
    fn vert(result1: Bounds, result2: Bounds) -> Bounds {
        Bounds {
            notation: Vert(vec![result1.notation, result2.notation]),
            flat_width: <Option<Col>>::vert(result1.flat_width, result2.flat_width),
            bound_set: BoundSet::vert(result1.bound_set, result2.bound_set),
        }
    }

    fn if_flat(result1: Bounds, result2: Bounds) -> Bounds {
        let flat_width = <Option<Col>>::if_flat(result1.flat_width, result2.flat_width);
        let bound_set = BoundSet::if_flat(result1.bound_set, result2.bound_set);
        let notation = IfFlat(
            Box::new(result1.notation),
            Box::new(result2.notation),
            flat_width,
        );
        Bounds {
            flat_width,
            notation,
            bound_set,
        }
    }
}

/*
impl ComileNotation<'_> {
    fn compile(&self, notation: &Notation) -> Bounds {
        }
    }
}
*/
