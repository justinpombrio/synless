use std::mem;
use std::ops::{Add, BitOr};

// INVARIANT: Choice vec is non-empty
#[derive(Clone, Debug)]
pub enum Notation {
    Literal(String),
    Newline,
    Indent(usize, Box<Notation>),
    NoWrap(Box<Notation>),
    /// Right child's requirement
    Concat(Box<Notation>, Box<Notation>, Requirement),
    Nest(Box<Notation>, Box<Notation>),
    Choice((Box<Notation>, Requirement), (Box<Notation>, Requirement)),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Requirement {
    pub single_line: Option<usize>,
    // first line length, last line length
    pub multi_line: Option<(usize, usize)>,
}

impl Notation {
    pub fn finalize(&mut self) -> Result<(), ()> {
        let req = self.validate()?;
        if !req.is_possible() {
            println!("impossible requirement at root!");
            Err(())
        } else {
            Ok(())
        }
    }

    fn validate(&mut self) -> Result<Requirement, ()> {
        match self {
            Notation::Literal(lit) => Ok(Requirement {
                single_line: Some(lit.len()),
                multi_line: None,
            }),
            Notation::Newline => Ok(Requirement {
                single_line: None,
                multi_line: Some((0, 0)),
            }),
            Notation::Indent(indent, note) => {
                let mut req = note.validate()?;
                if let Some((_, last_len)) = req.multi_line.as_mut() {
                    *last_len += *indent;
                }
                Ok(req)
            }
            Notation::NoWrap(note) => {
                let mut req = note.validate()?;
                req.multi_line = None;
                Ok(req)
            }
            Notation::Concat(left, right, uninit_right_req) => {
                let left_req = left.validate()?;
                let right_req = right.validate()?;
                *uninit_right_req = right_req;

                // Compute requirements
                let mut req = Requirement {
                    single_line: None,
                    multi_line: None,
                };
                let mut multi_line_options = vec![];

                if let (Some(ls), Some(rs)) = (left_req.single_line, right_req.single_line) {
                    req.single_line = Some(ls + rs);
                }
                if let (Some(ls), Some(rm)) = (left_req.single_line, right_req.multi_line) {
                    multi_line_options.push((ls + rm.0, rm.1))
                }
                if let (Some(lm), Some(rs)) = (left_req.multi_line, right_req.single_line) {
                    multi_line_options.push((lm.0, lm.1 + rs))
                }
                if let (Some(lm), Some(rm)) = (left_req.multi_line, right_req.multi_line) {
                    multi_line_options.push((lm.0, rm.1))
                }
                // TODO: check if minimizing by first_length == minimizing by last_length
                req.multi_line = multi_line_options.into_iter().min_by_key(|ml| ml.0);
                Ok(req)
            }
            Notation::Nest(left, right) => {
                let left_req = left.validate()?;
                let right_req = right.validate()?;
                let indent = left_req.single_line.ok_or(())?;
                let req = Requirement {
                    single_line: right_req.single_line.map(|w| indent + w),
                    multi_line: right_req.multi_line.map(|(f, l)| (indent + f, indent + l)),
                };
                Ok(req)
            }
            Notation::Choice((note1, uninit_req1), (note2, uninit_req2)) => {
                let req1 = note1.validate()?;
                let req2 = note2.validate()?;
                match (req1.is_possible(), req2.is_possible()) {
                    (true, true) => {
                        *uninit_req1 = req1;
                        *uninit_req2 = req2;
                        Ok(req1.best(&req2))
                    }
                    (true, false) => {
                        let dummy = Notation::Newline;
                        *self = mem::replace(note1, dummy);
                        Ok(req1)
                    }
                    (false, true) => {
                        let dummy = Notation::Newline;
                        *self = mem::replace(note2, dummy);
                        Ok(req2)
                    }
                    (false, false) => Err(()),
                }
            }
        }
    }

    pub fn indent(indent: usize, notation: Notation) -> Self {
        Notation::Indent(indent, Box::new(notation))
    }

    pub fn literal(lit: &str) -> Self {
        Notation::Literal(lit.to_owned())
    }

    pub fn concat(left: Notation, right: Notation) -> Self {
        Notation::Concat(Box::new(left), Box::new(right), Requirement::new())
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

impl Requirement {
    fn new() -> Self {
        Requirement {
            single_line: None,
            multi_line: None,
        }
    }

    pub fn has_single_line(&self) -> bool {
        self.single_line.is_some()
    }

    pub fn fits_single_line(&self, length: isize) -> bool {
        self.single_line
            .map(|l| (l as isize) <= length)
            .unwrap_or(false)
    }

    pub fn fits_multi_line(&self, first_length: isize, last_length: isize) -> bool {
        self.multi_line
            .map(|(fl, ll)| (fl as isize) <= first_length && (ll as isize) <= last_length)
            .unwrap_or(false)
    }

    fn is_possible(&self) -> bool {
        self.single_line.is_some() || self.multi_line.is_some()
    }

    /// Combine the best (smallest) parts of both Requirements.
    fn best(&self, other: &Self) -> Self {
        let single_line = match (self.single_line, other.single_line) {
            (Some(x), Some(y)) => Some(x.min(y)),
            (Some(x), None) | (None, Some(x)) => Some(x),
            (None, None) => None,
        };
        let multi_line = match (self.multi_line, other.multi_line) {
            (Some((f1, _)), Some((f2, _))) if f1 <= f2 => self.multi_line,
            (Some(_), Some(_)) => other.multi_line,
            (Some(_), None) => self.multi_line,
            (None, Some(_)) => other.multi_line,
            (None, None) => None,
        };
        Requirement {
            single_line,
            multi_line,
        }
    }
}

impl Add<Notation> for Notation {
    type Output = Notation;
    /// Shorthand for `Concat`.
    fn add(self, other: Notation) -> Notation {
        Notation::Concat(Box::new(self), Box::new(other), Requirement::new())
    }
}

impl BitOr<Notation> for Notation {
    type Output = Notation;
    /// Shorthand for `Choice`.
    fn bitor(self, other: Notation) -> Notation {
        Notation::Choice(
            (Box::new(self), Requirement::new()),
            (Box::new(other), Requirement::new()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal() {
        let mut lit = Notation::Literal("foobar".into());
        let req = lit.validate().unwrap();
        assert_eq!(
            req,
            Requirement {
                single_line: Some(6),
                multi_line: None,
            }
        );
    }

    #[test]
    fn test_newline() {
        let mut newline = Notation::Newline;
        let req = newline.validate().unwrap();
        assert_eq!(
            req,
            Requirement {
                single_line: None,
                multi_line: Some((0, 0)),
            }
        );
    }

    #[test]
    fn test_concat_literals() {
        let mut note = Notation::concat(Notation::literal("foo"), Notation::literal("bar"));
        let req = note.validate().unwrap();
        assert_eq!(
            req,
            Requirement {
                single_line: Some(6),
                multi_line: None,
            }
        );
        match note {
            Notation::Concat(_, _, reserved) => assert_eq!(
                reserved,
                Requirement {
                    single_line: Some(3),
                    multi_line: None
                }
            ),
            _ => panic!("Expected Notation::Concat variant"),
        }
    }
}
