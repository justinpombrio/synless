use super::notation::Notation;
use super::validate::ValidNotation;

#[derive(Clone, Debug)]
pub enum MeasuredNotation {
    Literal(String),
    Newline,
    Indent(usize, Box<MeasuredNotation>),
    Flat(Box<MeasuredNotation>),
    /// Requirement is for second MesuredNotation
    Concat(Box<MeasuredNotation>, Box<MeasuredNotation>, Requirement),
    Nest(Box<MeasuredNotation>, Box<MeasuredNotation>),
    /// Requirement is for first MeasuredNotation
    Choice(
        (Box<MeasuredNotation>, Requirement),
        (Box<MeasuredNotation>, Requirement),
    ),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Requirement {
    pub single_line: Option<usize>,
    // first line length, last line length
    pub multi_line: Option<(usize, usize)>,
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
                };
                (note, req)
            }
            Notation::Newline => {
                let note = MeasuredNotation::Newline;
                let req = Requirement {
                    single_line: None,
                    multi_line: Some((0, 0)),
                };
                (note, req)
            }
            Notation::Indent(indent, note) => {
                let (note, mut req) = note.measure_rec();
                if let Some((_, last_len)) = req.multi_line.as_mut() {
                    *last_len += *indent;
                }
                let note = MeasuredNotation::Indent(*indent, Box::new(note));
                (note, req)
            }
            Notation::Flat(note) => {
                let (note, mut req) = note.measure_rec();
                req.multi_line = None;
                let note = MeasuredNotation::Flat(Box::new(note));
                (note, req)
            }
            Notation::Concat(left, right) => {
                let (left_note, left_req) = left.measure_rec();
                let (right_note, right_req) = right.measure_rec();

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

                // Create notation
                let note =
                    MeasuredNotation::Concat(Box::new(left_note), Box::new(right_note), right_req);
                (note, req)
            }
            Notation::Nest(left, right) => {
                let (left_note, left_req) = left.measure_rec();
                let (right_note, right_req) = right.measure_rec();
                let indent = left_req.single_line.expect("Left Nest not flat");
                let req = Requirement {
                    single_line: right_req.single_line.map(|w| indent + w),
                    multi_line: right_req.multi_line.map(|(f, l)| (indent + f, indent + l)),
                };
                let note = MeasuredNotation::Nest(Box::new(left_note), Box::new(right_note));
                (note, req)
            }
            Notation::Choice(left, right) => {
                let (left_note, left_req) = left.measure_rec();
                let (right_note, right_req) = right.measure_rec();
                let req = left_req.best(right_req);
                let note = MeasuredNotation::Choice(
                    (Box::new(left_note), left_req),
                    (Box::new(right_note), right_req),
                );
                (note, req)
            }
        }
    }
}

impl Requirement {
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

    pub fn suffix_len(&self, suffix_len: usize) -> usize {
        match (self.single_line, self.multi_line) {
            (Some(sl), Some(ml)) => ml.0.min(sl + suffix_len),
            (Some(sl), None) => sl + suffix_len,
            (None, Some(ml)) => ml.0,
            (None, None) => panic!("Impossible notation"),
        }
    }

    /// Combine the best (smallest) parts of both Requirements.
    fn best(self, other: Self) -> Self {
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
                multi_line: Some((0, 0)),
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
            }
        );
        match note {
            MeasuredNotation::Concat(_, _, reserved) => assert_eq!(
                reserved,
                Requirement {
                    single_line: Some(3),
                    multi_line: None
                }
            ),
            _ => panic!("Expected MeasuredNotation::Concat variant"),
        }
    }
}
