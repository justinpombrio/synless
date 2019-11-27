use std::mem;
use std::ops::{Add, BitOr};

// INVARIANT: Choice vec is non-empty
#[derive(Clone, Debug)]
pub enum Notation {
    Literal(String),
    Newline,
    Indent(usize, Box<Notation>),
    NoWrap(Box<Notation>),
    Concat(Box<Notation>, Box<Notation>, ChoosyChild),
    Nest(Box<Notation>, Box<Notation>),
    Choice((Box<Notation>, Requirement), (Box<Notation>, Requirement)),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Compatibility {
    single_line: Option<()>,
    // choosy first, choosy last
    multi_line: Option<(bool, bool)>,
}

// INVARIANT: At least one of the fields must be Some.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Requirement {
    single_line: Option<usize>,
    // first line length, last line length
    multi_line: Option<(usize, usize)>,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum ChoosyChild {
    Left,
    Right,
    Neither,
    Uninitialized,
}

impl Notation {
    pub fn finalize(&mut self) -> Result<(), ()> {
        let (req, _compat) = self.validate()?;
        if !req.is_possible() {
            println!("no choices at root!");
            Err(())
        } else {
            Ok(())
        }
    }

    fn validate(&mut self) -> Result<(Requirement, Compatibility), ()> {
        match self {
            Notation::Literal(lit) => Ok((
                Requirement {
                    single_line: Some(lit.len()),
                    multi_line: None,
                },
                Compatibility {
                    single_line: Some(()),
                    multi_line: None,
                },
            )),
            Notation::Newline => Ok((
                Requirement {
                    single_line: None,
                    multi_line: Some((0, 0)),
                },
                Compatibility {
                    single_line: None,
                    multi_line: Some((false, false)),
                },
            )),
            Notation::Indent(indent, note) => {
                let (mut req, compat) = note.validate()?;
                if let Some((_, last_len)) = req.multi_line.as_mut() {
                    *last_len += *indent;
                }
                Ok((req, compat))
            }
            Notation::NoWrap(note) => {
                let (mut req, mut compat) = note.validate()?;
                req.multi_line = None;
                compat.multi_line = None;
                Ok((req, compat))
            }
            Notation::Concat(left, right, choosy) => {
                let (left_req, left_compat) = left.validate()?;
                let (right_req, right_compat) = right.validate()?;

                // Compute compatibility
                let mut compat = left_compat;
                if right_compat.single_line.is_none() {
                    compat.single_line = None;
                }
                match compat.multi_line.as_mut() {
                    None => compat.multi_line = right_compat.multi_line,
                    Some((first_choosy, last_choosy)) => {
                        if let Some((first, last)) = right_compat.multi_line {
                            *first_choosy |= first;
                            *last_choosy |= last;
                        }
                    }
                }

                // Compute choosiness
                let choosy_left = left_compat.multi_line.map(|(_, l)| l).unwrap_or(false);
                let choosy_right = right_compat.multi_line.map(|(f, _)| f).unwrap_or(false);
                *choosy = match (choosy_left, choosy_right) {
                    (true, true) => return Err(()),
                    (true, false) => ChoosyChild::Left,
                    (false, true) => ChoosyChild::Right,
                    (false, false) => ChoosyChild::Neither,
                };

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
                Ok((req, compat))
            }
            Notation::Nest(left, right) => {
                let (left_req, _) = left.validate()?;
                let (right_req, right_compat) = right.validate()?;
                let indent = left_req.single_line.ok_or(())?;
                let req = Requirement {
                    single_line: right_req.single_line.map(|w| indent + w),
                    multi_line: right_req.multi_line.map(|(f, l)| (indent + f, indent + l)),
                };
                Ok((req, right_compat))
            }
            Notation::Choice((note1, uninit_req1), (note2, uninit_req2)) => {
                let (req1, compat1) = note1.validate()?;
                let (req2, compat2) = note2.validate()?;
                match (req1.is_possible(), req2.is_possible()) {
                    (true, true) => {
                        *uninit_req1 = req1;
                        *uninit_req2 = req2;
                        Ok((req1.best(&req2), compat1.worst(&compat2)))
                    }
                    (true, false) => {
                        let dummy = Notation::Newline;
                        *self = mem::replace(note1, dummy);
                        Ok((req1, compat1))
                    }
                    (false, true) => {
                        let dummy = Notation::Newline;
                        *self = mem::replace(note2, dummy);
                        Ok((req2, compat2))
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
        Notation::Concat(Box::new(left), Box::new(right), ChoosyChild::Uninitialized)
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

impl Compatibility {
    /// Combine the worst parts of both Compatibilities.
    fn worst(&self, other: &Self) -> Self {
        let single_line = self.single_line.or(other.single_line);
        let multi_line = match (self.multi_line, other.multi_line) {
            (Some((f1, l1)), Some((f2, l2))) => Some((f1 | f2, l1 | l2)),
            (Some(x), None) | (None, Some(x)) => Some(x),
            (None, None) => None,
        };
        Compatibility {
            single_line,
            multi_line,
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

    pub fn fits_single_line(&self, length: usize) -> bool {
        self.single_line.map(|l| l <= length).unwrap_or(false)
    }

    pub fn fits_multi_line(&self, first_length: usize, last_length: usize) -> bool {
        self.multi_line
            .map(|(fl, ll)| fl <= first_length && ll <= last_length)
            .unwrap_or(false)
    }

    fn is_possible(&self) -> bool {
        self.single_line.is_some() || self.multi_line.is_some()
    }

    /// Combine the best parts of both Requirements.
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
        Notation::Concat(Box::new(self), Box::new(other), ChoosyChild::Uninitialized)
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
        let (req, compat) = lit.validate().unwrap();
        assert_eq!(
            req,
            Requirement {
                single_line: Some(6),
                multi_line: None,
            }
        );
        assert_eq!(
            compat,
            Compatibility {
                single_line: Some(()),
                multi_line: None,
            }
        );
    }

    #[test]
    fn test_newline() {
        let mut newline = Notation::Newline;
        let (req, compat) = newline.validate().unwrap();
        assert_eq!(
            req,
            Requirement {
                single_line: None,
                multi_line: Some((0, 0)),
            }
        );
        assert_eq!(
            compat,
            Compatibility {
                single_line: None,
                multi_line: Some((false, false)),
            }
        );
    }

    #[test]
    fn test_concat_literals() {
        let mut note = Notation::concat(Notation::literal("foo"), Notation::literal("bar"));
        let (req, compat) = note.validate().unwrap();
        assert_eq!(
            req,
            Requirement {
                single_line: Some(6),
                multi_line: None,
            }
        );
        assert_eq!(
            compat,
            Compatibility {
                single_line: Some(()),
                multi_line: None,
            }
        );
        match note {
            Notation::Concat(_, _, choosy) => assert_eq!(choosy, ChoosyChild::Neither),
            _ => panic!("Expected Notation::Concat variant"),
        }
    }
}
