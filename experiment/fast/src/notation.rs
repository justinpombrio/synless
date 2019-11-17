#[derive(Clone, Debug)]
pub enum Notation {
    Literal(String),
    Newline,
    Indent(usize, Box<Notation>),
    NoWrap(Box<Notation>),
    Concat {
        left: Box<Notation>,
        right: Box<Notation>,
        choosy: ChoosyChild,
    },
    Nest(Box<Notation>, Box<Notation>),
    Choice(Vec<(Notation, Requirement)>),
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
            Notation::Concat {
                left,
                right,
                choosy,
            } => {
                let left_compat = left.finalize_helper()?;
                let right_compat = right.finalize_helper()?;

                let multi_options = match (
                    left_compat.single_line,
                    right_compat.single_line,
                    &left_compat.multi_line,
                    &right_compat.multi_line,
                ) {
                    (Some(ls), Some(rs), Some(lm), Some(rm)) => vec![
                        lm.concat_multi(rm)?,
                        lm.concat_single_right(rs),
                        rm.concat_single_left(ls),
                    ],
                    (None, Some(rs), Some(lm), Some(rm)) => {
                        vec![lm.concat_multi(rm)?, lm.concat_single_right(rs)]
                    }
                    (Some(ls), None, Some(lm), Some(rm)) => {
                        vec![lm.concat_multi(rm)?, rm.concat_single_left(ls)]
                    }
                    (None, None, Some(lm), Some(rm)) => vec![lm.concat_multi(rm)?],
                    (Some(ls), _, None, Some(rm)) => vec![rm.concat_single_left(ls)],
                    (_, Some(rs), Some(lm), None) => vec![lm.concat_single_right(rs)],
                    _ => Vec::new(),
                };

                *choosy = ChoosyChild::Neither;
                if let (Some(lm), Some(rm)) = (left_compat.multi_line, right_compat.multi_line) {
                    *choosy = match (lm.choosy_last, rm.choosy_first) {
                        (true, true) => return Err(()),
                        (true, false) => ChoosyChild::Left,
                        (false, true) => ChoosyChild::Right,
                        (false, false) => ChoosyChild::Neither,
                    };
                }

                let multi_line = multi_options.into_iter().min_by_key(|ml| ml.first_length);

                let single_line = if let (Some(x), Some(y)) =
                    (left_compat.single_line, right_compat.single_line)
                {
                    Some(x + y)
                } else {
                    None
                };

                Ok(Compatibility {
                    single_line,
                    multi_line,
                })
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
            Notation::Choice(options) => {
                let mut valid_options = Vec::new();
                let mut total_compat = Compatibility::new();
                for (mut note, _) in options.drain(..) {
                    let compat = note.finalize_helper()?;
                    if let Some(req) = compat.into_requirement() {
                        valid_options.push((note, req));
                        total_compat = total_compat.min(&compat);
                    }
                }
                if valid_options.is_empty() {
                    return Err(());
                }
                *options = valid_options;
                Ok(total_compat)
            }
        }
    }

    fn literal(lit: &str) -> Self {
        Notation::Literal(lit.to_owned())
    }

    fn concat(left: Notation, right: Notation) -> Self {
        Notation::Concat {
            left: Box::new(left),
            right: Box::new(right),
            choosy: ChoosyChild::Uninitialized,
        }
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

impl MultiLine {
    fn concat_single_right(&self, right: usize) -> MultiLine {
        MultiLine {
            first_length: self.first_length,
            last_length: self.last_length + right,
            choosy_first: self.choosy_first,
            choosy_last: self.choosy_last,
        }
    }

    fn concat_single_left(&self, left: usize) -> MultiLine {
        MultiLine {
            first_length: left + self.first_length,
            last_length: self.last_length,
            choosy_first: self.choosy_first,
            choosy_last: self.choosy_last,
        }
    }

    fn concat_multi(&self, right: &MultiLine) -> Result<MultiLine, ()> {
        if self.choosy_last && right.choosy_first {
            Err(())
        } else {
            Ok(MultiLine {
                first_length: self.first_length,
                last_length: right.last_length,
                choosy_first: self.choosy_first,
                choosy_last: right.choosy_last,
            })
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
            Notation::Concat { choosy, .. } => assert_eq!(choosy, ChoosyChild::Neither),
            _ => panic!("Expected Notation::Concat variant"),
        }
    }
}
