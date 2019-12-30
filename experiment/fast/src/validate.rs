//! Validate that there is at least _one_ way to lay out a notation.

use super::Notation;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidNotation {
    Literal(String),
    Flat(Box<ValidNotation>),
    Align(Box<ValidNotation>),
    Concat(Box<ValidNotation>, Box<ValidNotation>, ChoosyChild),
    Nest(Box<ValidNotation>, usize, Box<ValidNotation>),
    Choice(Box<ValidNotation>, Box<ValidNotation>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValidationError {
    Impossible,
    TooChoosy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChoosyChild {
    Left,
    Right,
    Neither,
}

#[derive(Clone, Copy, Debug)]
struct Possibilities {
    /// `Some` if there's at least one single line possibility, and `true` if there's more than one.
    single_line: Option<bool>,
    multi_line: Option<ChoosyLines>,
}

// In the worst case, might this position be choosy (contain a choice or an aligned)
#[derive(Clone, Copy, Debug)]
struct ChoosyLines {
    first: bool,
    last: bool,
}

impl Possibilities {
    fn new_impossible() -> Self {
        Self {
            single_line: None,
            multi_line: None,
        }
    }

    fn new_single(single_choosy: bool) -> Self {
        Self {
            single_line: Some(single_choosy),
            multi_line: None,
        }
    }

    fn new_multi(first_choosy: bool, last_choosy: bool) -> Self {
        Self {
            single_line: None,
            multi_line: Some(ChoosyLines {
                first: first_choosy,
                last: last_choosy,
            }),
        }
    }

    fn is_possible(self) -> bool {
        self.single_line.is_some() || self.multi_line.is_some()
    }

    fn choosy_first(self) -> Option<bool> {
        match (self.single_line, self.multi_line) {
            (Some(_), Some(_)) => Some(true),
            (Some(choosy), None) => Some(choosy),
            (None, Some(ml)) => Some(ml.first),
            (None, None) => None,
        }
    }

    fn choosy_last(self) -> Option<bool> {
        match (self.single_line, self.multi_line) {
            (Some(_), Some(_)) => Some(true),
            (Some(choosy), None) => Some(choosy),
            (None, Some(ml)) => Some(ml.last),
            (None, None) => None,
        }
    }

    fn or(self, other: Possibilities) -> Possibilities {
        Possibilities {
            single_line: or_map(self.single_line, other.single_line, |_, _| true),
            multi_line: or_map(self.multi_line, other.multi_line, |_, _| ChoosyLines {
                first: true,
                last: true,
            }),
        }
    }
}

impl Notation {
    pub fn validate(self) -> Result<ValidNotation, ValidationError> {
        let (valid_notation, poss) = self.validate_rec()?;
        if poss.is_possible() {
            Ok(valid_notation)
        } else {
            Err(ValidationError::Impossible)
        }
    }

    fn validate_rec(&self) -> Result<(ValidNotation, Possibilities), ValidationError> {
        Ok(match self {
            Notation::Literal(l) => (
                ValidNotation::Literal(l.into()),
                Possibilities {
                    single_line: Some(false),
                    multi_line: None,
                },
            ),
            Notation::Flat(note) => {
                let (valid_note, mut poss) = note.validate_rec()?;
                poss.multi_line = None;
                (ValidNotation::Flat(Box::new(valid_note)), poss)
            }
            Notation::Align(note) => {
                let (valid_note, mut poss) = note.validate_rec()?;
                if let Some(multi) = poss.multi_line.as_mut() {
                    multi.first = true;
                    multi.last = true;
                }
                (ValidNotation::Align(Box::new(valid_note)), poss)
            }
            Notation::Nest(left, indent, right) => {
                let (valid_left, left_poss) = left.validate_rec()?;
                let (valid_right, right_poss) = right.validate_rec()?;

                let multi_line = match (left_poss.choosy_first(), right_poss.choosy_last()) {
                    (Some(first), Some(last)) => Some(ChoosyLines { first, last }),
                    _ => None,
                };
                (
                    ValidNotation::Nest(Box::new(valid_left), *indent, Box::new(valid_right)),
                    Possibilities {
                        single_line: None,
                        multi_line,
                    },
                )
            }
            Notation::Concat(left, right) => {
                let (valid_left, left_poss) = left.validate_rec()?;
                let (valid_right, right_poss) = right.validate_rec()?;

                let choosy_child = match (
                    left_poss.choosy_last().unwrap_or(false),
                    right_poss.choosy_first().unwrap_or(false),
                ) {
                    (false, false) => ChoosyChild::Neither,
                    (true, false) => ChoosyChild::Left,
                    (false, true) => ChoosyChild::Right,
                    (true, true) => return Err(ValidationError::TooChoosy),
                };

                let mut poss = Possibilities::new_impossible();
                if let (Some(ls), Some(rs)) = (left_poss.single_line, right_poss.single_line) {
                    poss = poss.or(Possibilities::new_single(ls || rs));
                }
                if let (Some(ls), Some(rm)) = (left_poss.single_line, right_poss.multi_line) {
                    poss = poss.or(Possibilities::new_multi(ls | rm.first, rm.last));
                }
                if let (Some(lm), Some(rs)) = (left_poss.multi_line, right_poss.single_line) {
                    poss = poss.or(Possibilities::new_multi(lm.first, lm.last || rs));
                }
                if let (Some(lm), Some(rm)) = (left_poss.multi_line, right_poss.multi_line) {
                    poss = poss.or(Possibilities::new_multi(lm.first, rm.last));
                }

                (
                    ValidNotation::Concat(
                        Box::new(valid_left),
                        Box::new(valid_right),
                        choosy_child,
                    ),
                    poss,
                )
            }
            Notation::Choice(left, right) => {
                let (valid_left, left_poss) = left.validate_rec()?;
                let (valid_right, right_poss) = right.validate_rec()?;

                (
                    ValidNotation::Choice(Box::new(valid_left), Box::new(valid_right)),
                    left_poss.or(right_poss),
                )
            }
        })
    }
}

fn or_map<T, F>(opta: Option<T>, optb: Option<T>, combine: F) -> Option<T>
where
    F: Fn(T, T) -> T,
{
    match (opta, optb) {
        (None, None) => None,
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (Some(a), Some(b)) => Some(combine(a, b)),
    }
}

#[allow(dead_code)]
fn and_map<T, F>(opta: Option<T>, optb: Option<T>, combine: F) -> Option<T>
where
    F: Fn(T, T) -> T,
{
    match (opta, optb) {
        (None, None) | (Some(..), None) | (None, Some(..)) => None,
        (Some(a), Some(b)) => Some(combine(a, b)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lit(s: &str) -> Notation {
        Notation::Literal(s.into())
    }

    fn flat(note: Notation) -> Notation {
        Notation::Flat(Box::new(note))
    }

    fn nest(left: Notation, indent: usize, right: Notation) -> Notation {
        Notation::Nest(Box::new(left), indent, Box::new(right))
    }

    #[test]
    fn test_impossible_flat() {
        let note = lit("foo") + lit("bar");
        note.validate().unwrap();

        let note = nest(lit("foo"), 4, lit("bar"));
        note.validate().unwrap();

        let note = flat(nest(lit("foo"), 4, lit("bar")));
        assert_eq!(note.validate(), Err(ValidationError::Impossible));
    }

    #[test]
    fn test_choosy() {
        let note = lit("foo") | lit("bar");
        note.validate().unwrap();

        let note = (lit("foo") | lit("bar")) + lit("red");
        note.validate().unwrap();

        let note = lit("foo") + (lit("red") | lit("blue"));
        note.validate().unwrap();

        let note = (lit("foo") | lit("bar")) + (lit("red") | lit("blue"));
        assert_eq!(note.validate(), Err(ValidationError::TooChoosy));
        // TODO add test with multiline choice inside of flat
    }
}
