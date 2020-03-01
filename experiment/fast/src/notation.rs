use std::ops::{Add, BitOr};

// TODO consider adding LineBreak that inserts a single newline even when there
// are multiple copies of it in a row. Useful for rust import-style list
// wrapping.
#[derive(Clone, Debug)]
pub enum Notation {
    /// Literal text. Cannot contain a newline.
    Literal(String),
    /// Only consider single-line options of the contained notation.
    Flat(Box<Notation>),
    /// Start all lines in the contained notation from the column of the
    /// leftmost character of the first line.
    Align(Box<Notation>),
    /// Display a newline, followed by the contained notation indented to the
    /// right by the given number of spaces.
    Nest(usize, Box<Notation>),
    /// Display both notations. The first character of the right notation
    /// immediately follows the last character of the left notation. The right
    /// notation's indentation level is not affected.
    Concat(Box<Notation>, Box<Notation>),
    /// Display the left notation if it fits within the required width;
    /// otherwise the right.
    Choice(Box<Notation>, Box<Notation>),
}

impl Notation {
    pub fn nest(indent: usize, note: Notation) -> Self {
        Notation::Nest(indent, Box::new(note))
    }

    pub fn literal(lit: &str) -> Self {
        Notation::Literal(lit.to_owned())
    }

    pub fn concat(left: Notation, right: Notation) -> Self {
        Notation::Concat(Box::new(left), Box::new(right))
    }

    // TODO: build this into the notation. This can be exponentially large!
    pub fn repeat<L, J, S>(
        elements: Vec<Notation>,
        empty: Notation,
        lone: L,
        join: J,
        surround: S,
    ) -> Notation
    where
        L: Fn(Notation) -> Notation,
        J: Fn(Notation, Notation) -> Notation,
        S: Fn(Notation) -> Notation,
    {
        let mut iter = elements.into_iter();
        match iter.len() {
            0 => empty,
            1 => lone(iter.next().unwrap()),
            _ => {
                let mut iter = iter.rev();
                let mut accumulator = iter.next().unwrap();
                for elem in iter {
                    accumulator = join(elem, accumulator);
                }
                surround(accumulator)
            }
        }
    }
}

impl Add<Notation> for Notation {
    type Output = Notation;
    /// Shorthand for `Concat`.
    fn add(self, other: Notation) -> Notation {
        Notation::Concat(Box::new(self), Box::new(other))
    }
}

impl BitOr<Notation> for Notation {
    type Output = Notation;
    /// Shorthand for `Choice`.
    fn bitor(self, other: Notation) -> Notation {
        Notation::Choice(Box::new(self), Box::new(other))
    }
}
