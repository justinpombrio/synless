use super::notation::Notation;
use super::requirement::{AlignedMultiLine, MultiLine, Requirement};
use super::validate::ValidNotation;

#[derive(Clone, Debug)]
pub enum MeasuredNotation {
    Literal(String),
    Newline,
    Indent(usize, Box<MeasuredNotation>),
    Flat(Box<MeasuredNotation>),
    Align(Box<MeasuredNotation>),
    /// Requirement is for second MeasuredNotation
    Concat(Box<MeasuredNotation>, Box<MeasuredNotation>, Requirement),
    Choice(
        (Box<MeasuredNotation>, Requirement),
        (Box<MeasuredNotation>, Requirement),
    ),
}

impl ValidNotation {
    pub fn measure(&self) -> MeasuredNotation {
        self.0.measure_rec().0
    }
}

impl Notation {
    fn measure_rec(&self) -> (MeasuredNotation, Requirement) {
        match self {
            Notation::Literal(lit) => {
                let note = MeasuredNotation::Literal(lit.clone());
                let req = Requirement {
                    single_line: Some(lit.chars().count()),
                    multi_line: None,
                    aligned: None,
                };
                (note, req)
            }
            Notation::Newline => {
                let note = MeasuredNotation::Newline;
                let req = Requirement {
                    single_line: None,
                    multi_line: Some(MultiLine {
                        first: 0,
                        middle: 0,
                        last: 0,
                    }),
                    aligned: None,
                };
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
                req.multi_line = None;
                req.aligned = None;
                (note, req)
            }
            Notation::Align(note) => {
                let (note, mut req) = note.measure_rec();
                let note = MeasuredNotation::Align(Box::new(note));
                let multi_line = req.multi_line.take();
                let aligned = multi_line.map(|ml| AlignedMultiLine {
                    middle: ml.first.max(ml.middle),
                    last: ml.last,
                });
                let req = req.or_aligned(aligned);
                (note, req)
            }
            Notation::Concat(left, right) => {
                let (left_note, left_req) = left.measure_rec();
                let (right_note, right_req) = right.measure_rec();
                let note =
                    MeasuredNotation::Concat(Box::new(left_note), Box::new(right_note), right_req);
                let req = left_req.concat(right_req);
                (note, req)
            }
            Notation::Choice(left, right) => {
                let (left_note, left_req) = left.measure_rec();
                let (right_note, right_req) = right.measure_rec();
                let note = MeasuredNotation::Choice(
                    (Box::new(left_note), left_req),
                    (Box::new(right_note), right_req),
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
        assert_eq!(
            req,
            Requirement {
                single_line: Some(6),
                multi_line: None,
                aligned: None,
            }
        );
    }

    #[test]
    fn test_newline() {
        let newline = Notation::Newline;
        let req = newline.measure_rec().1;
        assert_eq!(
            req,
            Requirement {
                single_line: None,
                multi_line: Some(MultiLine {
                    first: 0,
                    middle: 0,
                    last: 0
                }),
                aligned: None,
            }
        );
    }

    #[test]
    fn test_concat_literals() {
        let note = Notation::concat(Notation::literal("foo"), Notation::literal("bar"));
        let (note, req) = note.measure_rec();
        assert_eq!(
            req,
            Requirement {
                single_line: Some(6),
                multi_line: None,
                aligned: None
            }
        );
        match note {
            MeasuredNotation::Concat(_, _, reserved) => assert_eq!(
                reserved,
                Requirement {
                    single_line: Some(3),
                    multi_line: None,
                    aligned: None
                }
            ),
            _ => panic!("Expected MeasuredNotation::Concat variant"),
        }
    }
}
