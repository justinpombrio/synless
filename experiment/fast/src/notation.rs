use std::ops::{Add, BitOr};

// TODO consider adding LineBreak that inserts a single newline even when there
// are multiple copies of it in a row. Useful for rust import-style list
// wrapping.
#[derive(Clone, Debug)]
pub enum Notation {
    Literal(String),
    Newline,
    Indent(usize, Box<Notation>),
    Flat(Box<Notation>),
    Concat(Box<Notation>, Box<Notation>),
    Nest(Box<Notation>, Box<Notation>),
    Choice(Box<Notation>, Box<Notation>),
}

impl Notation {
    pub fn indent(indent: usize, notation: Notation) -> Self {
        Notation::Indent(indent, Box::new(notation))
    }

    pub fn literal(lit: &str) -> Self {
        Notation::Literal(lit.to_owned())
    }

    pub fn concat(left: Notation, right: Notation) -> Self {
        Notation::Concat(Box::new(left), Box::new(right))
    }

    pub fn repeat<O, F, M, L, S>(
        elements: Vec<Notation>,
        empty: Notation,
        lone: O,
        first: F,
        middle: M,
        last: L,
        surround: S,
    ) -> Notation
    where
        O: Fn(Notation) -> Notation,
        F: Fn(Notation) -> Notation,
        M: Fn(Notation) -> Notation,
        L: Fn(Notation) -> Notation,
        S: Fn(Notation) -> Notation,
    {
        let length = elements.len();
        let mut elem_iter = elements.into_iter();
        match length {
            0 => empty,
            1 => lone(elem_iter.next().unwrap()),
            _ => {
                let mut accumulator = Notation::Newline; // dummy value
                for (i, elem) in elem_iter.enumerate() {
                    if i == 0 {
                        accumulator = first(elem);
                    } else if i == length - 2 {
                        accumulator = accumulator + last(elem);
                    } else {
                        accumulator = accumulator + middle(elem);
                    }
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
