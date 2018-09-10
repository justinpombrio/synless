use std::ops::{Add, BitOr};

use geometry::*;
use style::Style;
use self::Syntax::*;


pub const MAX_WIDTH : Col = 100;

/// Describes how to display a syntactic construct.
#[derive(Clone, Debug)]
pub enum Syntax {
    /// Display a literal string.
    Literal(String, Style),
    /// Display a piece of text. Must be used on a texty node.
    Text(Style),
    /// Display a newline after this syntax.
    Flush(Box<Syntax>),
    /// Display the first syntax, followed immediately by the second syntax.
    Concat(Box<Syntax>, Box<Syntax>),
    /// Display this syntax, not permitting flushes/newlines.
    NoWrap(Box<Syntax>),
    /// Display either the first syntax, or the second, whichever is Best.
    Choice(Box<Syntax>, Box<Syntax>),
    /// Display the first syntax in case this tree has empty text,
    /// otherwise show the second syntax.
    IfEmptyText(Box<Syntax>, Box<Syntax>),
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
    /// If the sequence is empty, display this Syntax.
    pub empty:  Syntax,
    /// If the sequence has length one, display this Syntax.
    pub lone:   Syntax,
    /// If the sequence has length 2 or more, begin with this Syntax.
    pub first:  Syntax,
    /// If the sequence has length 2 or more, display this syntax for
    /// every node in the sequence except the first and last.
    pub middle: Syntax,
    /// If the sequence has length 2 or more, end with this Syntax.
    pub last:   Syntax
}

/// Construct `Literal("")`, which displays nothing.
pub fn empty() -> Syntax {
    Literal("".to_string(), Style::plain())
}

/// Construct a `Literal`.
pub fn literal(s: &str, style: Style) -> Syntax {
    Literal(s.to_string(), style)
}

/// Construct a `Text`.
pub fn text(style: Style) -> Syntax {
    Text(style)
}

/// Construct a `NoWrap`.
pub fn no_wrap(syn: Syntax) -> Syntax {
    NoWrap(Box::new(syn))
}

/// Construct a `Child`.
pub fn child(index: usize) -> Syntax {
    Child(index)
}

/// Construct a `Flush`.
pub fn flush(syn: Syntax) -> Syntax {
    Flush(Box::new(syn))
}

/// Construct a `Repeat`.
pub fn repeat(repeat: Repeat) -> Syntax {
    Rep(Box::new(repeat))
}

/// Construct a `Star` (for use in `Repeat`).
pub fn star() -> Syntax {
    Star
}

/// Construct an `IfEmptyText`.
pub fn if_empty_text(syn1: Syntax, syn2: Syntax) -> Syntax {
    IfEmptyText(Box::new(syn1), Box::new(syn2))
}

/// Construct a `Concat`. You can also use `+` for this.
pub fn concat(syn1: Syntax, syn2: Syntax) -> Syntax {
    Concat(Box::new(syn1), Box::new(syn2))
}

/// Construct a `Choice`. You can also use `|` for this.
pub fn choice(syn1: Syntax, syn2: Syntax) -> Syntax {
    Choice(Box::new(syn1), Box::new(syn2))
}

impl Add<Syntax> for Syntax {
    ///
    type Output = Syntax;
    /// Shorthand for `Concat`.
    fn add(self, other: Syntax) -> Syntax {
        Concat(Box::new(self), Box::new(other))
    }
}

impl BitOr<Syntax> for Syntax {
    ///
    type Output = Syntax;
    /// Shorthand for `Choice`.
    fn bitor(self, other: Syntax) -> Syntax {
        Choice(Box::new(self), Box::new(other))
    }
}

struct SyntaxExpander {
    arity: usize,
    num_args: usize,
    empty_text: bool
}

impl SyntaxExpander {
    fn expand(&self, syntax: &Syntax) -> Syntax {
        match syntax {
            &Literal(ref s, style) => Literal(s.clone(), style),
            &Text(_)       => syntax.clone(),
            &Child(_)      => syntax.clone(),
            &Flush(ref s)  => flush(self.expand(s)),
            &NoWrap(ref s) => no_wrap(self.expand(s)),
            &Concat(ref a, ref b) => self.expand(a) + self.expand(b),
            &Choice(ref a, ref b) => self.expand(a) | self.expand(b),
            &IfEmptyText(ref a, ref b) =>
                self.expand(if self.empty_text { a } else { b }),
            &Rep(ref repeat) => {
                let &Repeat{ ref empty,
                             ref lone,
                             ref first, ref middle, ref last } = &**repeat;
                match self.num_args - self.arity {
                    0 => empty.clone(),
                    1 => lone.clone().replace_star(self.arity),
                    _ => {
                        let mut syn = last.replace_star(self.num_args - 1);
                        for i in (self.arity + 1 .. self.num_args - 1).rev() {
                            syn = middle.replace_star(i) + syn;
                        }
                        syn = first.replace_star(self.arity) + syn;
                        syn
                    }
                }
            },
            &Star{..} => panic!("Invalid notation: star found outside of repeat")
        }
        
    }
}

impl Syntax {
    // Eliminate any Repeats.
    pub(crate) fn expand(&self, arity: usize, num_args: usize, empty_text: bool)
                         -> Syntax
    {
        SyntaxExpander{
            arity: arity,
            num_args: num_args,
            empty_text: empty_text
        }.expand(self)
    }

    fn replace_star(&self, child: usize) -> Syntax {
        match self {
            &Literal(_, _) | &Text(_) | &Child(_) => self.clone(),
            &Flush(ref s)  => flush(s.replace_star(child)),
            &NoWrap(ref s) => no_wrap(s.replace_star(child)),
            &Concat(ref a, ref b) =>
                a.replace_star(child) + b.replace_star(child),
            &IfEmptyText(ref a, ref b) =>
                if_empty_text(a.replace_star(child), b.replace_star(child)),
            &Choice(ref a, ref b) =>
                a.replace_star(child) | b.replace_star(child),
            &Star => Child(child),
            &Rep(_) => panic!("Invalid notation: nested repeats not allowed")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lit(s: &str) -> Syntax {
        literal(s, Style::plain())
    }

    fn example_repeat_syntax() -> Syntax {
        child(0) +
            repeat(Repeat{
                empty:  lit("[]"),
                lone:   lit("[") + star() + lit("]"),
                first:  lit("[") + flush(star() + lit(",")),
                middle: flush(star() + lit(",")),
                last:   star() + lit("]")
            })
    }

    #[test]
    fn test_show_layout() {
        let syn = lit("abc") + (flush(lit("def")) + lit("g"));
        let lay = &syn.lay_out(0, &vec!(), false).fit_width(80);
        assert_eq!(format!("{:?}", lay), "abcdef\n   g");
    }

    #[test]
    fn test_expand_syntax() {
        let r = (flush(lit("abc")) + lit("de")).bound(0, vec!(), false);
        let syn = example_repeat_syntax();
        let zero = &syn
            .lay_out(1, &vec!(r.clone()), false)
            .fit_width(80);
        let one = &syn
            .lay_out(1, &vec!(r.clone(), r.clone()), false)
            .fit_width(80);
        let two = &syn
            .lay_out(1, &vec!(r.clone(), r.clone(), r.clone()), false)
            .fit_width(80);
        let three = &syn
            .lay_out(1, &vec!(r.clone(), r.clone(), r.clone(), r.clone()), false)
            .fit_width(80);
        let four = &syn
            .lay_out(1, &vec!(r.clone(), r.clone(), r.clone(), r.clone(), r.clone()), false)
            .fit_width(80);
        assert_eq!(format!("{:?}", zero), "000\n00[]");
        assert_eq!(format!("{:?}", one), "000\n00[111\n   11]");
        assert_eq!(format!("{:?}", two),
                   "000\n00[111\n   11,\n   222\n   22]");
        assert_eq!(format!("{:?}", three),
                   "000\n00[111\n   11,\n   222\n   22,\n   333\n   33]");
        assert_eq!(format!("{:?}", four),
                   "000\n00[111\n   11,\n   222\n   22,\n   333\n   33,\n   444\n   44]");
    }
}
