use std::ops::{Add, BitOr, BitXor};

use crate::style::Style;

use self::Notation::*;

/// Describes how to display a syntactic construct.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Notation {
    /// Display Nothing
    Empty,
    /// Display a literal string. Cannot contain a newline.
    Literal(String, Style),
    /// Display a piece of text. Must be used on a texty node.
    Text(Style),
    /// Display the second notation after the first, and so forth.
    Follow(Vec<Notation>),
    /// Display the second notation below the first (vertical concatenation),
    /// and so forth.
    Vert(Vec<Notation>),
    /// Display this notation, not permitting flushes/newlines.
    NoWrap(Box<Notation>),
    /// Display exactly one of the notations, whichever is Best.
    Choice(Vec<Notation>),
    /// Display the first notation in case this tree has empty text,
    /// otherwise show the second notation.
    IfEmptyText(Box<Notation>, Box<Notation>),
    /// Display the `i`th child of this node.
    /// Must be used on a foresty node.
    /// `i` must be less than the node's arity number.
    Child(usize),
    /// Determines what to display based on the arity of this node.
    /// Used for syntactic constructs that have extendable arity.
    Repeat(Box<RepeatInner>),
    /// Used in [`Repeat`](Repeat) to refer to the accumulated Notation
    /// in `join`.
    Left,
    /// Used in [`Repeat`](Repeat) to refer to the next child in `join`.
    Right,
    /// Used in [`Repeat`](Repeat) to refer to the Notation inside of
    /// `surround`.
    Surrounded,
}

/// Describes how to display the extra children of a syntactic
/// construct with extendable arity.
#[derive(Clone, Debug, PartialEq, Eq)]
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

impl Add<Notation> for Notation {
    ///
    type Output = Notation;
    /// Shorthand for [`Concat`](Concat).
    fn add(self, other: Notation) -> Notation {
        Follow(vec![self, other])
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
    /// Shorthand for [`Choice`](Choice).
    fn bitor(self, other: Notation) -> Notation {
        Choice(vec![self, other])
    }
}

// TODO: also perform some optimizations, like removing `Empty`s.
impl Notation {
    /// Put a Notation into a more efficient normal form that avoids unnecessary
    /// nestings, and pre-computes some important information.
    pub fn normalize(self) -> Notation {
        // This is valid because `Follow` and `Vert` are associative.
        match self {
            Empty => Empty,
            Literal(string, style) => Literal(string, style),
            Text(style) => Text(style),
            Child(i) => Child(i),
            Follow(notations) => {
                if notations.is_empty() {
                    return Empty;
                }
                if notations.len() == 1 {
                    return notations.into_iter().next().unwrap().normalize();
                }
                let mut flattened = Vec::new();
                Follow(notations).flatten_follows(&mut flattened);
                Follow(flattened)
            }
            Vert(notations) => {
                if notations.is_empty() {
                    panic!("Notation: cannot have `Vert` of no notations")
                }
                if notations.len() == 1 {
                    return notations.into_iter().next().unwrap().normalize();
                }
                let mut flattened = Vec::new();
                Vert(notations).flatten_verts(&mut flattened);
                Vert(flattened)
            }
            NoWrap(n) => NoWrap(Box::new(n.normalize())),
            IfEmptyText(n1, n2) => IfEmptyText(Box::new(n1.normalize()), Box::new(n2.normalize())),
            Choice(notations) => Choice(notations.into_iter().map(|n| n.normalize()).collect()),
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

    fn flatten_follows(self, flattened: &mut Vec<Notation>) {
        match self {
            Follow(notations) => notations
                .into_iter()
                .map(|n| n.normalize())
                .for_each(|n| n.flatten_follows(flattened)),
            other => flattened.push(other.normalize()),
        }
    }

    fn flatten_verts(self, flattened: &mut Vec<Notation>) {
        match self {
            Vert(notations) => notations
                .into_iter()
                .map(|n| n.normalize())
                .for_each(|n| n.flatten_verts(flattened)),
            other => flattened.push(other),
        }
    }
}

#[cfg(test)]
mod notation_tests {
    use super::*;

    fn txt(s: &str) -> Notation {
        Notation::Literal(s.to_string(), Style::plain())
    }

    #[test]
    fn test_normalize_basic() {
        let n = txt("a") + (txt("b") + txt("c"));
        let expected = Notation::Follow(vec![txt("a"), txt("b"), txt("c")]);
        assert_eq!(n.normalize(), expected);
    }

    #[test]
    fn test_normalize_one_follow() {
        let n = Notation::Follow(vec![txt("a") ^ (txt("b") ^ txt("c"))]);
        let expected = Notation::Vert(vec![txt("a"), txt("b"), txt("c")]);
        assert_eq!(n.normalize(), expected);
    }

    #[test]
    fn test_normalize() {
        let n = txt("a") + (txt("b") ^ (txt("c") ^ txt("d")));
        let expected = Notation::Follow(vec![
            txt("a"),
            Notation::Vert(vec![txt("b"), txt("c"), txt("d")]),
        ]);
        assert_eq!(n.normalize(), expected);
    }

    #[test]
    fn test_normalize_nested_follow() {
        let n = txt("a") + Notation::Vert(vec![txt("b") + txt("c")]);
        let expected = Notation::Follow(vec![txt("a"), txt("b"), txt("c")]);
        assert_eq!(n.normalize(), expected);
    }
}
