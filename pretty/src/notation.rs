use std::iter;
use std::ops::{Add, BitOr, BitXor};

use crate::style::Style;
use crate::utility::error;

use self::Notation::*;

/// Describes how to display a syntactic construct.
#[derive(Clone, Debug)]
pub enum Notation {
    /// Display Nothing
    Empty,
    /// Display a literal string. Cannot contain a newline.
    Literal(String, Style),
    /// Display a piece of text. Must be used on a texty node.
    Text(Style),
    /// Display the `i`th child of this node.
    /// Must be used on a foresty node.
    /// `i` must be less than the node's arity number.
    Child(usize),
    /// Display the second notation after the first, with an indentation
    /// determined by its last line (and so forth recursively).
    Nest(Vec<Notation>),
    /// Display the second notation below the first (and so forth recursively).
    /// A.k.a. vertical concatentation.
    Vert(Vec<Notation>),
    /// Display the first notation if it fits on one, or else the second
    /// notation.
    IfFlat(Box<Notation>, Box<Notation>),
    /// Display the first notation in case this tree has empty text,
    /// otherwise show the second notation.
    IfEmptyText(Box<Notation>, Box<Notation>),
    /// Determines what to display based on the arity of this node.
    /// This is used for syntactic constructs that have extendable arity.
    Repeat(Box<RepeatInner>),
    /// Used in [`Repeat`](Repeat) to denote the accumulated Notation
    /// in `join`.
    Left,
    /// Used in [`Repeat`](Repeat) to denote the next child in `join`.
    Right,
    /// Used in [`Repeat`](Repeat) to denote the Notation inside of
    /// `surround`.
    Surrounded,
}

/// Determines what to display based on the arity of this node.
/// This is used for syntactic constructs that have extendable arity.
#[derive(Clone, Debug)]
pub struct RepeatInner {
    /// If the sequence is empty, use this notation.
    pub empty: Notation,
    /// If the sequence has length one, use this notation.
    pub lone: Notation,
    /// If the sequence has length 2 or more, (left-)fold elements together with
    /// this notation. [`Left`](Left) holds the notation so far, while
    /// [`Right`](Right) holds the next child to be folded.
    pub join: Notation,
    /// If the sequence has length 2 or more, surround the folded notation with
    /// this notation. [`Surrounded`](Surrounded) holds the folded notation.
    pub surround: Notation,
}

/// Construct `Literal("")`, which displays nothing.
pub fn empty() -> Notation {
    Literal("".to_string(), Style::plain())
}

/// Construct a [`Literal`](Literal).
pub fn literal(s: &str, style: Style) -> Notation {
    Literal(s.to_string(), style)
}

/// Construct a [`Text`](Text).
pub fn text(style: Style) -> Notation {
    Text(style)
}

/// Construct a [`Child`](Child).
pub fn child(index: usize) -> Notation {
    Child(index)
}

/// Construct a [`Nest`](Nest). You can also use
/// [`+`](enum.Notation.html#impl-Add<Notation>) for this.
pub fn nest(notations: Vec<Notation>) -> Notation {
    Nest(notations)
}

/// Construct a [`Vert`](Vert). You can also use
/// [`^`](enum.Notation.html#impl-BitXor<Notation>) for this.
pub fn vert(notations: Vec<Notation>) -> Notation {
    Vert(notations)
}

/// Construct an [`IfFlat`](IfFlat). You can also use
/// [`|`](enum.Notation.html#impl-BitOr<Notation>) for this.
pub fn if_flat(notation1: Notation, notation2: Notation) -> Notation {
    IfFlat(Box::new(notation1), Box::new(notation2))
}

/// Construct an [`IfEmptyText`](IfEmptyText).
pub fn if_empty_text(notation1: Notation, notation2: Notation) -> Notation {
    IfEmptyText(Box::new(notation1), Box::new(notation2))
}

/// Construct a [`Repeat`](Repeat).
pub fn repeat(repeat: RepeatInner) -> Notation {
    Repeat(Box::new(repeat))
}

impl Add<Notation> for Notation {
    ///
    type Output = Notation;
    /// Shorthand for [`Nest`](Nest).
    fn add(self, other: Notation) -> Notation {
        Nest(vec![self, other])
    }
}

impl BitXor<Notation> for Notation {
    ///
    type Output = Notation;
    /// Shorthand for [`Vert`](Vert).
    fn bitxor(self, other: Notation) -> Notation {
        Vert(vec![self, other])
    }
}

impl BitOr<Notation> for Notation {
    ///
    type Output = Notation;
    /// Shorthand for [`IfFlat`](IfFlat).
    fn bitor(self, other: Notation) -> Notation {
        IfFlat(Box::new(self), Box::new(other))
    }
}

impl Notation {
    /// Put a Notation into a more efficient normal form that avoids unnecessary
    /// nestings.
    pub fn normalize(self) -> Notation {
        // This is valid because `Nest` and `Vert` are associative.
        match self {
            Empty => Empty,
            Literal(string, style) => Literal(string, style),
            Text(style) => Text(style),
            Child(i) => Child(i),
            Nest(mut notations) => {
                if notations.len() == 0 {
                    error!("Notation: cannot have `Nest` of no notations")
                }
                if notations.len() == 1 {
                    return notations.pop().unwrap();
                }
                let mut flattened = Vec::new();
                Nest(notations).flatten_nests(&mut flattened);
                Nest(flattened)
            }
            Vert(mut notations) => {
                if notations.len() == 0 {
                    error!("Notation: cannot have `Vert` of no notations")
                }
                if notations.len() == 1 {
                    return notations.pop().unwrap();
                }
                let mut flattened = Vec::new();
                Vert(notations).flatten_verts(&mut flattened);
                Vert(flattened)
            }
            IfFlat(n1, n2) => IfFlat(Box::new(n1.normalize()), Box::new(n2.normalize())),
            IfEmptyText(n1, n2) => IfEmptyText(Box::new(n1.normalize()), Box::new(n2.normalize())),
            Repeat(box RepeatInner {
                empty,
                lone,
                join,
                surround,
            }) => Repeat(Box::new(RepeatInner {
                empty: empty.normalize(),
                lone: lone.normalize(),
                join: join.normalize(),
                surround: surround.normalize(),
            })),
            Left => Left,
            Right => Right,
            Surrounded => Surrounded,
        }
    }

    fn flatten_nests(self, flattened: &mut Vec<Notation>) {
        match self {
            Nest(notations) => notations
                .into_iter()
                .for_each(|n| n.flatten_nests(flattened)),
            other => flattened.push(other),
        }
    }

    fn flatten_verts(self, flattened: &mut Vec<Notation>) {
        match self {
            Vert(notations) => notations
                .into_iter()
                .for_each(|n| n.flatten_verts(flattened)),
            other => flattened.push(other),
        }
    }
}
