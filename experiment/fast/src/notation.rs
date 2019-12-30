use std::ops::{Add, BitOr};

// TODO consider adding LineBreak that inserts a single newline even when there
// are multiple copies of it in a row. Useful for rust import-style list
// wrapping.
#[derive(Clone, Debug)]
pub enum Notation {
    Literal(String),
    Flat(Box<Notation>),
    Align(Box<Notation>),
    Concat(Box<Notation>, Box<Notation>),
    Nest(Box<Notation>, usize, Box<Notation>),
    Choice(Box<Notation>, Box<Notation>),
}

impl Notation {
    pub fn nest(left: Notation, indent: usize, right: Notation) -> Self {
        Notation::Nest(Box::new(left), indent, Box::new(right))
    }

    pub fn literal(lit: &str) -> Self {
        Notation::Literal(lit.to_owned())
    }

    pub fn concat(left: Notation, right: Notation) -> Self {
        Notation::Concat(Box::new(left), Box::new(right))
    }

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
        let length = elements.len();
        let mut elem_iter = elements.into_iter();
        match length {
            0 => empty,
            1 => lone(elem_iter.next().unwrap()),
            _ => {
                let mut accumulator = elem_iter.next().unwrap();
                for elem in elem_iter {
                    accumulator = join(accumulator, elem);
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
