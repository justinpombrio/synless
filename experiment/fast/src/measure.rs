use super::notation::Notation;
use super::validate::ValidNotation;

#[derive(Clone, Debug)]
pub enum MeasuredNotation {
    Literal(String),
    Newline,
    Indent(usize, Box<MeasuredNotation>),
    Flat(Box<MeasuredNotation>),
    Align(Box<MeasuredNotation>),
    /// Requirement is for second MesuredNotation
    Concat(Box<MeasuredNotation>, Box<MeasuredNotation>, Requirement),
    Choice(
        (Box<MeasuredNotation>, Requirement),
        (Box<MeasuredNotation>, Requirement),
    ),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Requirement {
    pub single_line: Option<usize>,
    pub multi_line: Option<MultiLine>,
    pub aligned: Option<AlignedMultiLine>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MultiLine {
    pub first: usize,
    pub middle: usize,
    pub last: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AlignedMultiLine {
    // includes first line
    pub middle: usize,
    pub last: usize,
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
                req.multi_line = None;
                req.aligned = None;
                let note = MeasuredNotation::Flat(Box::new(note));
                (note, req)
            }
            Notation::Align(note) => {
                let (note, mut req) = note.measure_rec();
                let multi_line = req.multi_line.take();
                let aligned = multi_line.map(|ml| AlignedMultiLine {
                    middle: ml.first.max(ml.middle),
                    last: ml.last,
                });
                let req = req.or_aligned(aligned);
                let note = MeasuredNotation::Align(Box::new(note));
                (note, req)
            }
            Notation::Concat(left, right) => {
                let (left_note, left_req) = left.measure_rec();
                let (right_note, right_req) = right.measure_rec();
                let req = left_req.concat(right_req);
                let note =
                    MeasuredNotation::Concat(Box::new(left_note), Box::new(right_note), right_req);
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
    pub fn new_single_line(len: usize) -> Requirement {
        Requirement {
            single_line: Some(len),
            multi_line: None,
            aligned: None,
        }
    }

    pub fn fits(&self, width: usize) -> bool {
        if let Some(sl) = self.single_line {
            if sl <= width {
                return true;
            }
        }
        if let Some(ml) = self.multi_line {
            if ml.first <= width && ml.middle <= width && ml.last <= width {
                return true;
            }
        }
        if let Some(al) = self.aligned {
            if al.middle <= width && al.last <= width {
                return true;
            }
        }
        false
    }

    pub fn has_single_line(&self) -> bool {
        self.single_line.is_some()
    }

    pub fn indent(mut self, indent: usize) -> Self {
        if let Some(multi_line) = self.multi_line.as_mut() {
            multi_line.middle += indent;
            multi_line.last += indent;
        }
        self
    }

    pub fn concat(self, other: Requirement) -> Self {
        let mut req = Requirement {
            single_line: None,
            multi_line: None,
            aligned: None,
        };
        let mut multi_line_options = vec![];
        let mut aligned_options = vec![];
        if let (Some(ls), Some(rs)) = (self.single_line, other.single_line) {
            req.single_line = Some(ls + rs);
        }
        if let (Some(ls), Some(rm)) = (self.single_line, other.multi_line) {
            multi_line_options.push(MultiLine {
                first: ls + rm.first,
                middle: rm.middle,
                last: rm.last,
            });
        }
        if let (Some(lm), Some(rs)) = (self.multi_line, other.single_line) {
            multi_line_options.push(MultiLine {
                first: lm.first,
                middle: lm.middle,
                last: lm.last + rs,
            });
        }
        if let (Some(lm), Some(rm)) = (self.multi_line, other.multi_line) {
            multi_line_options.push(MultiLine {
                first: lm.first,
                middle: lm.middle.max(lm.last + rm.first).max(rm.middle),
                last: rm.last,
            });
        }
        if let (Some(ls), Some(ra)) = (self.single_line, other.aligned) {
            aligned_options.push(AlignedMultiLine {
                middle: ls + ra.middle,
                last: ls + ra.last,
            });
        }
        if let (Some(la), Some(rs)) = (self.aligned, other.single_line) {
            aligned_options.push(AlignedMultiLine {
                middle: la.middle,
                last: la.last + rs,
            });
        }
        if let (Some(la), Some(ra)) = (self.aligned, other.aligned) {
            aligned_options.push(AlignedMultiLine {
                middle: la.middle.max(la.last + ra.middle),
                last: la.last + ra.last,
            });
        }
        if let (Some(lm), Some(ra)) = (self.multi_line, other.aligned) {
            multi_line_options.push(MultiLine {
                first: lm.first,
                middle: lm.middle.max(lm.last + ra.middle),
                last: lm.last + ra.last,
            });
        }
        if let (Some(la), Some(rm)) = (self.aligned, other.multi_line) {
            multi_line_options.push(MultiLine {
                first: la.middle.max(la.last + rm.first),
                middle: rm.middle,
                last: rm.last,
            });
        }
        // This only works well if the options left and right obey Justin's Property:
        //   (left.first < right.first) -> (left.last <= right.last).
        req.multi_line = multi_line_options.into_iter().min_by_key(|ml| ml.first);
        req.aligned = aligned_options.into_iter().min_by_key(|al| al.middle);
        req
    }

    fn or_single_line(mut self, single_line: Option<usize>) -> Self {
        if let Some(new) = single_line {
            self.single_line = match self.single_line {
                Some(old) => Some(old.min(new)),
                None => Some(new),
            };
        }
        self
    }

    fn or_multi_line(mut self, multi_line: Option<MultiLine>) -> Self {
        if let Some(new) = multi_line {
            self.multi_line = match self.multi_line {
                Some(old) => {
                    if new.first < old.first {
                        Some(new)
                    } else {
                        Some(old)
                    }
                }
                None => Some(new),
            }
        }
        self
    }

    fn or_aligned(mut self, aligned: Option<AlignedMultiLine>) -> Self {
        if let Some(new) = aligned {
            self.aligned = match self.aligned {
                Some(old) => {
                    if new.middle < old.middle {
                        Some(new)
                    } else {
                        Some(old)
                    }
                }
                None => Some(new),
            }
        }
        self
    }

    /// Combine the best (smallest) parts of both Requirements.
    fn best(self, other: Self) -> Self {
        self.or_single_line(other.single_line)
            .or_multi_line(other.multi_line)
            .or_aligned(other.aligned)
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
