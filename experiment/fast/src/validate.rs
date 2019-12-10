//! Validate that there is at least _one_ way to lay out a notation.

use super::Notation;
use std::mem;
use Notation::*;

pub struct ValidNotation(pub(crate) Notation);

struct Possibilities {
    single_line: bool,
    multi_line: bool,
}

impl Possibilities {
    fn is_possible(&self) -> bool {
        self.single_line || self.multi_line
    }
}

impl Notation {
    pub fn validate(mut self) -> Result<ValidNotation, ()> {
        let possibilities = self.validate_rec();
        if possibilities.is_possible() {
            Ok(ValidNotation(self))
        } else {
            Err(())
        }
    }

    fn validate_rec(&mut self) -> Possibilities {
        match self {
            Literal(_) => Possibilities {
                single_line: true,
                multi_line: false,
            },
            Newline => Possibilities {
                single_line: false,
                multi_line: true,
            },
            Indent(_indent, note) => note.validate_rec(),
            Flat(note) => {
                let mut possibilities = note.validate_rec();
                possibilities.multi_line = false;
                possibilities
            }
            Align(note) => note.validate_rec(),
            Concat(left, right) => {
                let left_poss = left.validate_rec();
                let right_poss = right.validate_rec();
                Possibilities {
                    single_line: left_poss.single_line && right_poss.single_line,
                    multi_line: left_poss.single_line && right_poss.multi_line
                        || left_poss.multi_line && right_poss.single_line
                        || left_poss.multi_line && right_poss.multi_line,
                }
            }
            Choice(left, right) => {
                let left_poss = left.validate_rec();
                let right_poss = right.validate_rec();
                if !left_poss.is_possible() {
                    let dummy = Notation::Newline;
                    let right = mem::replace(right.as_mut(), dummy);
                    *self = right;
                    right_poss
                } else if !right_poss.is_possible() {
                    let dummy = Notation::Newline;
                    let left = mem::replace(left.as_mut(), dummy);
                    *self = left;
                    left_poss
                } else {
                    Possibilities {
                        single_line: left_poss.single_line || right_poss.single_line,
                        multi_line: left_poss.multi_line || right_poss.multi_line,
                    }
                }
            }
        }
    }
}
