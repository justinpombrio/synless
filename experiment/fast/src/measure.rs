use super::requirement::{Aligned, NonChoosyFirstLineLen, Requirements};
use super::staircase::Staircase;
use super::validate::{ChoosyChild, ValidNotation};

#[derive(Clone, Debug)]
pub enum MeasuredNotation {
    Literal(String),
    Nest(Box<MeasuredNotation>, usize, Box<MeasuredNotation>),
    Flat(Box<MeasuredNotation>),
    Align(Box<MeasuredNotation>),
    Concat(
        Box<MeasuredNotation>,
        Box<MeasuredNotation>,
        ChoosyChild,
        /// If the left requirement is choosy, store the first line length of
        /// the right requirement. Otherwise, None.
        Option<NonChoosyFirstLineLen>,
    ),
    Choice(
        (Box<MeasuredNotation>, Requirements),
        (Box<MeasuredNotation>, Requirements),
    ),
}

impl ValidNotation {
    pub fn measure(&self) -> MeasuredNotation {
        self.measure_rec().0
    }

    fn measure_rec(&self) -> (MeasuredNotation, Requirements) {
        match self {
            ValidNotation::Literal(lit) => {
                let note = MeasuredNotation::Literal(lit.clone());
                let req = Requirements::new_single_line(lit.chars().count());
                (note, req)
            }
            ValidNotation::Nest(left, indent, right) => {
                let (left_note, left_req) = left.measure_rec();
                let (right_note, right_req) = right.measure_rec();
                let note =
                    MeasuredNotation::Nest(Box::new(left_note), *indent, Box::new(right_note));
                let req = left_req.nest(*indent, right_req);
                (note, req)
            }
            ValidNotation::Flat(note) => {
                let (note, mut req) = note.measure_rec();
                let note = MeasuredNotation::Flat(Box::new(note));
                req.multi_line = Staircase::new();
                req.aligned = Staircase::new();
                (note, req)
            }
            ValidNotation::Align(note) => {
                let (note, mut req) = note.measure_rec();
                let note = MeasuredNotation::Align(Box::new(note));
                for ml in req.multi_line.drain() {
                    req.aligned.insert(Aligned {
                        middle: ml.first.max(ml.middle),
                        last: ml.last,
                    })
                }
                (note, req)
            }
            ValidNotation::Concat(left, right, choosy_child) => {
                let (left_note, left_req) = left.measure_rec();
                let (right_note, right_req) = right.measure_rec();
                let non_choosy_right_first_line_len = match *choosy_child {
                    ChoosyChild::Left => Some(right_req.non_choosy_first_line_len()),
                    _ => None,
                };
                let note = MeasuredNotation::Concat(
                    Box::new(left_note),
                    Box::new(right_note),
                    *choosy_child,
                    non_choosy_right_first_line_len,
                );
                let req = left_req.concat(&right_req);
                (note, req)
            }
            ValidNotation::Choice(left, right) => {
                let (left_note, left_req) = left.measure_rec();
                let (right_note, right_req) = right.measure_rec();
                // TODO avoid cloning?
                let note = MeasuredNotation::Choice(
                    (Box::new(left_note), left_req.clone()),
                    (Box::new(right_note), right_req.clone()),
                );
                let req = left_req.best(right_req);
                (note, req)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notation::Notation;

    #[test]
    fn test_literal() {
        let lit = Notation::Literal("foobar".into()).validate().unwrap();
        let req = lit.measure_rec().1;
        assert_eq!(req, Requirements::new_single_line(6));
    }

    #[test]
    fn test_concat_literals() {
        let note = Notation::concat(Notation::literal("foo"), Notation::literal("bar"))
            .validate()
            .unwrap();
        let (note, req) = note.measure_rec();

        assert_eq!(req, Requirements::new_single_line(6));
        match note {
            MeasuredNotation::Concat(_, _, choosy_child, len) => {
                assert_eq!(choosy_child, ChoosyChild::Neither);
                assert_eq!(len, None);
            }
            _ => panic!("Expected MeasuredNotation::Concat variant"),
        }
    }
}
