use std::mem;

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

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
struct Compatibility {
    single_line: Option<usize>,
    multi_line: Option<MultiLine>,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
struct MultiLine {
    first_length: usize,
    last_length: usize,
    // Note: Choices inside NoWraps don't count as choosy
    choosy_first: bool,
    choosy_last: bool,
}

// INVARIANT: At least one of the fields must be Some.
#[derive(Clone, Copy, Debug)]
pub struct Requirement {
    single_line: Option<usize>,
    multi_line: Option<(usize, usize)>,
}

impl Requirement {
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
        let compat = self.finalize_helper()?;
        if compat.is_possible() {
            println!("no choices at root!");
            Err(())
        } else {
            Ok(())
        }
    }

    fn finalize_helper(&mut self) -> Result<Compatibility, ()> {
        match self {
            Notation::Literal(lit) => Ok(Compatibility {
                single_line: Some(lit.len()),
                multi_line: None,
            }),
            Notation::Newline => Ok(Compatibility {
                single_line: None,
                multi_line: Some(MultiLine {
                    first_length: 0,
                    last_length: 0,
                    choosy_first: false,
                    choosy_last: false,
                }),
            }),
            Notation::Indent(_i, note) => note.finalize_helper(),
            Notation::NoWrap(note) => {
                let mut compat = note.finalize_helper()?;
                compat.multi_line = None;
                Ok(compat)
            }
            Notation::Concat(left, right, choosy) => {
                let left_compat = left.finalize_helper()?;
                let right_compat = right.finalize_helper()?;
                // Initialize defaults
                let mut compat = Compatibility {
                    single_line: None,
                    multi_line: None,
                };
                let mut multi_line_options = vec![];
                *choosy = ChoosyChild::Neither;
                // Consider the four cases that left&right could be concatenated
                if let (Some(ls), Some(rs)) = (left_compat.single_line, right_compat.single_line) {
                    compat.single_line = Some(ls + rs);
                }
                if let (Some(ls), Some(rm)) = (left_compat.single_line, right_compat.multi_line) {
                    let mut option = rm;
                    option.first_length += ls;
                    multi_line_options.push(option);
                }
                if let (Some(lm), Some(rs)) = (left_compat.multi_line, right_compat.single_line) {
                    let mut option = lm;
                    option.last_length += rs;
                    multi_line_options.push(option);
                }
                if let (Some(lm), Some(rm)) = (left_compat.multi_line, right_compat.multi_line) {
                    multi_line_options.push(MultiLine {
                        first_length: lm.first_length,
                        last_length: rm.last_length,
                        choosy_first: lm.choosy_first,
                        choosy_last: rm.choosy_last,
                    });
                    *choosy = match (lm.choosy_last, rm.choosy_first) {
                        (true, true) => return Err(()),
                        (true, false) => ChoosyChild::Left,
                        (false, true) => ChoosyChild::Right,
                        (false, false) => ChoosyChild::Neither,
                    };
                }
                // TODO: check if minimizing by first_length == minimizing by last_length
                compat.multi_line = multi_line_options
                    .into_iter()
                    .min_by_key(|ml| ml.first_length);
                Ok(compat)
            }
            Notation::Nest(left, right) => {
                let left_compat = left.finalize_helper()?;
                let mut right_compat = right.finalize_helper()?;
                let indent = left_compat.single_line.ok_or(())?;
                if let Some(flat_width) = right_compat.single_line.as_mut() {
                    *flat_width += indent
                }
                if let Some(ml) = right_compat.multi_line.as_mut() {
                    ml.first_length += indent;
                }
                Ok(right_compat)
            }
            Notation::Choice((note1, _), (note2, _)) => {
                let compat1 = note1.finalize_helper()?;
                let compat2 = note2.finalize_helper()?;
                match (compat1.into_requirement(), compat2.into_requirement()) {
                    (Some(req1), Some(req2)) => Ok(compat1.min(&compat2)),
                    (Some(req1), None) => {
                        let dummy = Notation::Newline;
                        *self = mem::replace(note1, dummy);
                        Ok(compat1)
                    }
                    (None, Some(req2)) => {
                        let dummy = Notation::Newline;
                        *self = mem::replace(note2, dummy);
                        Ok(compat2)
                    }
                    (None, None) => Err(()),
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
}

impl Compatibility {
    fn new() -> Compatibility {
        Compatibility {
            single_line: None,
            multi_line: None,
        }
    }
    fn is_possible(&self) -> bool {
        self.single_line.is_some() || self.multi_line.is_some()
    }

    fn into_requirement(&self) -> Option<Requirement> {
        if !self.is_possible() {
            None
        } else {
            Some(Requirement {
                single_line: self.single_line,
                multi_line: self.multi_line.map(|ml| (ml.first_length, ml.last_length)),
            })
        }
    }

    fn min(&self, other: &Compatibility) -> Compatibility {
        let single_line = match (self.single_line, other.single_line) {
            (Some(x), Some(y)) => Some(x.min(y)),
            (Some(x), None) | (None, Some(x)) => Some(x),
            (None, None) => None,
        };
        let multi_line = match (self.multi_line, other.multi_line) {
            (Some(x), Some(y)) if x.first_length <= y.first_length => Some(x),
            (Some(_), Some(y)) => Some(y),
            (Some(x), None) | (None, Some(x)) => Some(x),
            (None, None) => None,
        };
        Compatibility {
            single_line,
            multi_line,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal() {
        let mut lit = Notation::Literal("foobar".into());
        assert_eq!(
            lit.finalize_helper().unwrap(),
            Compatibility {
                single_line: Some(6),
                multi_line: None,
            }
        );
        let mut newline = Notation::Newline;
        assert_eq!(
            newline.finalize_helper().unwrap(),
            Compatibility {
                single_line: None,
                multi_line: Some(MultiLine {
                    first_length: 0,
                    last_length: 0,
                    choosy_first: false,
                    choosy_last: false
                }),
            }
        );
    }

    #[test]
    fn test_concat_literals() {
        let mut note = Notation::concat(Notation::literal("foo"), Notation::literal("bar"));
        assert_eq!(
            note.finalize_helper().unwrap(),
            Compatibility {
                single_line: Some(6),
                multi_line: None,
            }
        );
        match note {
            Notation::Concat(_, _, choosy) => assert_eq!(choosy, ChoosyChild::Neither),
            _ => panic!("Expected Notation::Concat variant"),
        }
    }
}
