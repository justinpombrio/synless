use super::notation::Notation;
use super::requirement::{Aligned, MultiLine, Requirements};
use super::staircase::Staircase;
use super::validate::ValidNotation;

#[derive(Clone, Debug)]
pub enum MeasuredNotation {
    Literal(String),
    Newline,
    Indent(usize, Box<MeasuredNotation>),
    Flat(Box<MeasuredNotation>),
    Align(Box<MeasuredNotation>),
    /// Requirements is for second MeasuredNotation
    Concat(Box<MeasuredNotation>, Box<MeasuredNotation>, Requirements),
    Choice(
        (Box<MeasuredNotation>, Requirements),
        (Box<MeasuredNotation>, Requirements),
    ),
}

impl ValidNotation {
    pub fn measure(&self) -> MeasuredNotation {
        self.0.measure_rec().0
    }
}

impl Notation {
    fn measure_rec(&self) -> (MeasuredNotation, Requirements) {
        match self {
            Notation::Literal(lit) => {
                let note = MeasuredNotation::Literal(lit.clone());
                let req = Requirements::new_single_line(lit.chars().count());
                (note, req)
            }
            Notation::Newline => {
                let note = MeasuredNotation::Newline;
                let mut req = Requirements::new();
                req.multi_line.insert(MultiLine {
                    first: 0,
                    middle: 0,
                    last: 0,
                });
                (note, req)
            }
            Notation::Indent(indent, note) => {
                let (note, mut req) = note.measure_rec();
                req = req.indent(*indent);
                let note = MeasuredNotation::Indent(*indent, Box::new(note));
                (note, req)
            }
            Notation::Flat(note) => {
                let (note, mut req) = note.measure_rec();
                let note = MeasuredNotation::Flat(Box::new(note));
                req.multi_line = Staircase::new();
                req.aligned = Staircase::new();
                (note, req)
            }
            Notation::Align(note) => {
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
            Notation::Concat(left, right) => {
                let (left_note, left_req) = left.measure_rec();
                let (right_note, right_req) = right.measure_rec();
                let req = left_req.concat(&right_req);
                let note =
                    MeasuredNotation::Concat(Box::new(left_note), Box::new(right_note), right_req);
                (note, req)
            }
            Notation::Choice(left, right) => {
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

    #[test]
    fn test_literal() {
        let lit = Notation::Literal("foobar".into());
        let req = lit.measure_rec().1;
        assert_eq!(req, Requirements::new_single_line(6));
    }

    #[test]
    fn test_newline() {
        let newline = Notation::Newline;
        let req = newline.measure_rec().1;
        let mut expected_req = Requirements::new();
        expected_req.multi_line.insert(MultiLine {
            first: 0,
            middle: 0,
            last: 0,
        });
        assert_eq!(req, expected_req);
    }

    #[test]
    fn test_concat_literals() {
        let note = Notation::concat(Notation::literal("foo"), Notation::literal("bar"));
        let (note, req) = note.measure_rec();

        assert_eq!(req, Requirements::new_single_line(6));
        match note {
            MeasuredNotation::Concat(_, _, reserved) => {
                assert_eq!(reserved, Requirements::new_single_line(3))
            }
            _ => panic!("Expected MeasuredNotation::Concat variant"),
        }
    }
}
