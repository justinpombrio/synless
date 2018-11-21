use std::ops::{Add, BitOr, BitXor};

use crate::style::Style;

use self::Notation::*;


/// Describes how to display a syntactic construct.
#[derive(Clone, Debug)]
pub enum Notation {
    /// Display Nothing
    Empty,
    /// Display a literal string.
    Literal(String, Style),
    /// Display a piece of text. Must be used on a texty node.
    Text(Style),
    /// Display the second notation after the first (standard concatenation).
    Concat(Box<Notation>, Box<Notation>),
    /// Display the second notation to the right of the first (horizontal
    /// concatenation).
    Horz(Box<Notation>, Box<Notation>),
    /// Display the second notation below the first (vertical concatenation).
    Vert(Box<Notation>, Box<Notation>),
    /// Display this notation, not permitting flushes/newlines.
    NoWrap(Box<Notation>),
    /// Display either the first notation, or the second, whichever is Best.
    Choice(Box<Notation>, Box<Notation>),
    /// Display the first notation in case this tree has empty text,
    /// otherwise show the second notation.
    IfEmptyText(Box<Notation>, Box<Notation>),
    /// Display the `i`th child of this node.
    /// Must be used on a foresty node.
    /// `i` must be less than the node's arity number.
    Child(usize),
    /// Determines what to display based on the arity of this node.
    /// Used for syntactic constructs that have extendable arity.
    // TODO: Does this need to be boxed?
    Rep(Box<Repeat>),
    /// A node with extendable arity can have more children than its
    /// arity number. Within a `Rep`, `Star` represents the children
    /// beyond this arity. It does not make sense outside of a `Rep`.
    Star
}

/// Describes how to display the extra children of a syntactic
/// construct with extendable arity.
#[derive(Clone, Debug)]
pub struct Repeat {
    /// If the sequence is empty, use this notation.
    pub empty:  Notation,
    /// If the sequence has length one, use this notation.
    pub lone:   Notation,
    /// If the sequence has length 2 or more, begin with this notation.
    pub first:  Notation,
    /// If the sequence has length 2 or more, display this notation for
    /// every node in the sequence except the first and last.
    pub middle: Notation,
    /// If the sequence has length 2 or more, end with this notation.
    pub last:   Notation
}

/// Construct `Literal("")`, which displays nothing.
pub fn empty() -> Notation {
    Literal("".to_string(), Style::plain())
}

/// Construct a `Literal`.
pub fn literal(s: &str, style: Style) -> Notation {
    Literal(s.to_string(), style)
}

/// Construct a `Text`.
pub fn text(style: Style) -> Notation {
    Text(style)
}

/// Construct a `NoWrap`.
pub fn no_wrap(note: Notation) -> Notation {
    NoWrap(Box::new(note))
}

/// Construct a `Child`.
pub fn child(index: usize) -> Notation {
    Child(index)
}

/// Construct a `Repeat`.
pub fn repeat(repeat: Repeat) -> Notation {
    Rep(Box::new(repeat))
}

/// Construct a `Star` (for use in `Repeat`).
pub fn star() -> Notation {
    Star
}

/// Construct an `IfEmptyText`.
pub fn if_empty_text(note1: Notation, note2: Notation) -> Notation {
    IfEmptyText(Box::new(note1), Box::new(note2))
}

/// Construct a `Concat`. You can also use `+` for this.
pub fn concat(note1: Notation, note2: Notation) -> Notation {
    Concat(Box::new(note1), Box::new(note2))
}

/// Construct a `Horz`.
pub fn horz(note1: Notation, note2: Notation) -> Notation {
    Horz(Box::new(note1), Box::new(note2))
}

/// Construct a `Vert`. You can also use `^` for this.
pub fn vert(note1: Notation, note2: Notation) -> Notation {
    Vert(Box::new(note1), Box::new(note2))
}

/// Construct a `Choice`. You can also use `|` for this.
pub fn choice(note1: Notation, note2: Notation) -> Notation {
    Choice(Box::new(note1), Box::new(note2))
}

impl Add<Notation> for Notation {
    ///
    type Output = Notation;
    /// Shorthand for `Concat`.
    fn add(self, other: Notation) -> Notation {
        Concat(Box::new(self), Box::new(other))
    }
}

impl BitXor<Notation> for Notation {
    ///
    type Output = Notation;
    /// Shorthand for `Vert`.
    fn bitxor(self, other: Notation) -> Notation {
        Vert(Box::new(self), Box::new(other))
    }
}

impl BitOr<Notation> for Notation {
    ///
    type Output = Notation;
    /// Shorthand for `Choice`.
    fn bitor(self, other: Notation) -> Notation {
        Choice(Box::new(self), Box::new(other))
    }
}

struct NotationExpander {
    len: usize
}

impl NotationExpander {
    fn expand(&self, notation: &Notation) -> Notation {
        match notation {
            &Empty         => notation.clone(),
            &Literal(ref s, style) => Literal(s.clone(), style),
            &Text(_)       => notation.clone(),
            &Child(_)      => notation.clone(),
            &NoWrap(ref s) => no_wrap(self.expand(s)),
            &Concat(ref a, ref b) => self.expand(a) + self.expand(b),
            &Horz(ref a, ref b)   => horz(self.expand(a), self.expand(b)),
            &Vert(ref a, ref b)   => self.expand(a) ^ self.expand(b),
            &Choice(ref a, ref b) => self.expand(a) | self.expand(b),
            &IfEmptyText(ref a, ref b) =>
                self.expand(if self.len == 0 { a } else { b }),
            &Rep(ref repeat) => {
                let &Repeat{ ref empty,
                             ref lone,
                             ref first, ref middle, ref last } = &**repeat;
                match self.len {
                    0 => empty.clone(),
                    1 => lone.clone().replace_star(0),
                    _ => {
                        let mut note = last.replace_star(self.len - 1);
                        for i in (1 .. self.len - 1).rev() {
                            note = middle.replace_star(i) + note;
                        }
                        note = first.replace_star(0) + note;
                        note
                    }
                }
            },
            &Star{..} => panic!("Invalid notation: star found outside of repeat")
        }
        
    }
}

impl Notation {
    // Eliminate any Repeats.
    // If the node is texty, `len` is the length of the text.
    pub(crate) fn expand(&self, len: usize) -> Notation {
        NotationExpander{
            len: len
        }.expand(self)
    }

    fn replace_star(&self, child: usize) -> Notation {
        match self {
            &Empty => Empty,
            &Literal(_, _) | &Text(_) | &Child(_) => self.clone(),
            &NoWrap(ref s) => no_wrap(s.replace_star(child)),
            &Concat(ref a, ref b) =>
                a.replace_star(child) + b.replace_star(child),
            &Horz(ref a, ref b) =>
                horz(a.replace_star(child), b.replace_star(child)),
            &Vert(ref a, ref b) =>
                a.replace_star(child) ^ b.replace_star(child),
            &IfEmptyText(ref a, ref b) =>
                if_empty_text(a.replace_star(child), b.replace_star(child)),
            &Choice(ref a, ref b) =>
                a.replace_star(child) | b.replace_star(child),
            &Star => Child(child),
            &Rep(_) => panic!("Invalid notation: nested repeats not allowed")
        }
    }
}
