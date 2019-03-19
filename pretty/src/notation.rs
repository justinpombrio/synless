use std::ops::{Add, BitOr, BitXor};

use crate::style::Style;

use self::Notation::*;

// TODO: fix layout so that these properties hold

/// Describes how to display a syntactic construct.

#[derive(Clone, Debug)]
pub enum Notation {
    /// Display Nothing
    Empty,
    /// Display a literal string. Cannot contain a newline.
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
    Rep(Box<Repeat>),
    /// (For internal use.)
    WithMemoized(Box<Notation>, Box<Notation>),
    /// (For internal use.)
    Memoized,
}

/// Describes how to display the extra children of a syntactic
/// construct with extendable arity.
#[derive(Clone, Debug)]
pub struct Repeat {
    /// If the sequence is empty, use this notation.
    pub empty: Notation,
    /// If the sequence has length one, use this notation.
    pub lone: Notation,
    /// If the sequence has length 2 or more, (left-)fold elements together with this notation.
    /// `Child(0)` holds the notation so far, while `Child(1)` holds the next child to be folded.
    // TODO: should this use `Left` and `Right` instead of `Child(0)` and `Child(1)`?
    pub join: Notation,
    /// If the sequence has length 2 or more, surround the folded notation with this notation.
    /// `Child(0)` holds the folded notation.
    // TODO: should this use `Surrounded` instead of `Child(0)`?
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

/// Construct a [`NoWrap`](NoWrap).
pub fn no_wrap(note: Notation) -> Notation {
    NoWrap(Box::new(note))
}

/// Construct a [`Child`](Child).
pub fn child(index: usize) -> Notation {
    Child(index)
}

/// Construct a [`Rep`](Rep).
pub fn repeat(repeat: Repeat) -> Notation {
    Rep(Box::new(repeat))
}

/// Construct an [`IfEmptyText`](IfEmptyText).
pub fn if_empty_text(note1: Notation, note2: Notation) -> Notation {
    IfEmptyText(Box::new(note1), Box::new(note2))
}

/// Construct a [`Concat`](Concat). You can also use
/// [`+`](enum.Notation.html#impl-Add<Notation>) for this.
pub fn concat(note1: Notation, note2: Notation) -> Notation {
    Concat(Box::new(note1), Box::new(note2))
}

/// Construct a [`Horz`](Horz).
pub fn horz(note1: Notation, note2: Notation) -> Notation {
    Horz(Box::new(note1), Box::new(note2))
}

/// Construct a [`Vert`](Vert). You can also use
/// [`^`](enum.Notation.html#impl-BitXor<Notation>) for this.
pub fn vert(note1: Notation, note2: Notation) -> Notation {
    Vert(Box::new(note1), Box::new(note2))
}

/// Construct a [`Choice`](Choice). You can also use
/// [`|`](enum.Notation.html#impl-BitOr<Notation>) for this.
pub fn choice(note1: Notation, note2: Notation) -> Notation {
    Choice(Box::new(note1), Box::new(note2))
}

impl Add<Notation> for Notation {
    ///
    type Output = Notation;
    /// Shorthand for [`Concat`](Concat).
    fn add(self, other: Notation) -> Notation {
        Concat(Box::new(self), Box::new(other))
    }
}

impl BitXor<Notation> for Notation {
    ///
    type Output = Notation;
    /// Shorthand for [`Vert`](Vert).
    fn bitxor(self, other: Notation) -> Notation {
        Vert(Box::new(self), Box::new(other))
    }
}

impl BitOr<Notation> for Notation {
    ///
    type Output = Notation;
    /// Shorthand for [`Choice`](Choice).
    fn bitor(self, other: Notation) -> Notation {
        Choice(Box::new(self), Box::new(other))
    }
}

struct NotationExpander {
    len: usize,
}

impl NotationExpander {
    fn expand(&self, notation: &Notation) -> Notation {
        match notation {
            Empty => notation.clone(),
            Literal(s, style) => Literal(s.clone(), *style),
            Text(_) => notation.clone(),
            Child(_) => notation.clone(),
            NoWrap(s) => no_wrap(self.expand(s)),
            Concat(a, b) => self.expand(a) + self.expand(b),
            Horz(a, b) => horz(self.expand(a), self.expand(b)),
            Vert(a, b) => self.expand(a) ^ self.expand(b),
            Choice(a, b) => self.expand(a) | self.expand(b),
            IfEmptyText(a, b) => self.expand(if self.len == 0 { a } else { b }),
            WithMemoized(a, b) => WithMemoized(Box::new(self.expand(a)), Box::new(self.expand(b))),
            Memoized => Memoized,
            Rep(repeat) => {
                let Repeat {
                    empty,
                    lone,
                    join,
                    surround,
                } = &**repeat;
                match self.len {
                    0 => empty.clone(),
                    1 => lone.clone(),
                    _ => {
                        let mut notation = Child(0);
                        for i in 1..self.len {
                            notation = WithMemoized(
                                Box::new(notation),
                                Box::new(
                                    join.replace_child(1, &Child(i)).replace_child(0, &Memoized),
                                ),
                            );
                        }
                        surround.replace_child(0, &notation)
                    }
                }
            }
        }
    }
}

impl Notation {
    // Eliminate any Repeats.
    // If the node is texty, `len` is the length of the text.
    pub(crate) fn expand(&self, len: usize) -> Notation {
        NotationExpander { len: len }.expand(self)
    }

    fn replace_child(&self, sought: usize, replacement: &Notation) -> Notation {
        let (s, r) = (sought, replacement);
        match self {
            Child(i) if *i == sought => r.clone(),
            Child(_) | Empty | Literal(_, _) | Text(_) => self.clone(),
            NoWrap(a) => no_wrap(a.replace_child(s, r)),
            Concat(a, b) => a.replace_child(s, r) + b.replace_child(s, r),
            Horz(a, b) => horz(a.replace_child(s, r), b.replace_child(s, r)),
            Vert(a, b) => a.replace_child(s, r) ^ b.replace_child(s, r),
            IfEmptyText(a, b) => if_empty_text(a.replace_child(s, r), b.replace_child(s, r)),
            Choice(a, b) => a.replace_child(s, r) | b.replace_child(s, r),
            WithMemoized(a, b) => WithMemoized(
                Box::new(a.replace_child(s, r)),
                Box::new(b.replace_child(s, r)),
            ),
            Memoized => Memoized,
            Rep(_) => panic!("Invalid notation: nested repeats not allowed"),
        }
    }
}
