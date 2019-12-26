use crate::staircase::{Stair, Staircase};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Requirements {
    pub single_line: Option<usize>,
    pub multi_line: Staircase<MultiLine>,
    pub aligned: Staircase<Aligned>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MultiLine {
    pub first: usize,
    pub middle: usize,
    pub last: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Aligned {
    // includes first line
    pub middle: usize,
    pub last: usize,
}

impl Stair for MultiLine {
    fn x(&self) -> usize {
        self.first
    }
    fn y(&self) -> usize {
        self.middle
    }
}

impl Stair for AlignedMultiLine {
    fn x(&self) -> usize {
        self.middle
    }
    fn y(&self) -> usize {
        self.last
    }
}

impl Requirements {
    pub fn new_single_line(len: usize) -> Requirements {
        Requirements {
            single_line: Some(len),
            multi_line: None,
            aligned: None,
        }
    }

    /// Do _any_ of the requirements fit within this width?
    pub fn fits(&self, width: usize) -> bool {
        if let Some(sl) = self.single_line {
            if sl <= width {
                return true;
            }
        }
        // TODO: could be more efficient
        for ml in &self.multi_line {
            if ml.first <= width && ml.middle <= width && ml.last <= width {
                return true;
            }
        }
        for al in &self.aligned {
            if al.middle <= width && al.last <= width {
                return true;
            }
        }
        false
    }

    pub fn is_possible(&self) -> bool {
        self.single_line.is_some() || !self.multi_line.is_empty() || !self.aligned.is_empty()
    }

    pub fn indent(mut self, indent: usize) -> Self {
        let multi_lines = self.multi_line.into_iter();
        self.multi_line = Staircase::new();
        for mut ml in multi_lines {
            ml.middle += indent;
            ml.last += indent;
            self.multi_line.unchecked_insert(ml);
        }
        self
    }

    // TODO: rewrite
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
        // This only works well if the options left and right obey the No-Tradeoff rule.
        //   (left.first < right.first) -> (left.last <= right.last).
        req.multi_line = multi_line_options.into_iter().min_by_key(|ml| ml.first);
        req.aligned = aligned_options.into_iter().min_by_key(|al| al.middle);
        req
    }

    // TODO: rewrite
    pub fn or_single_line(mut self, single_line: Option<usize>) -> Self {
        if let Some(new) = single_line {
            self.single_line = match self.single_line {
                Some(old) => Some(old.min(new)),
                None => Some(new),
            };
        }
        self
    }

    // TODO: rewrite
    pub fn or_multi_line(mut self, multi_line: Option<MultiLine>) -> Self {
        if let Some(new) = multi_line {
            self.multi_line = match self.multi_line {
                Some(old) => {
                    // TODO: Need to keep multiple options here
                    if new.first < old.first {
                        Some(new)
                    } else {
                        Some(old)
                    }
                }
                None => Some(new),
            };
        }
        self
    }

    // TODO: rewrite
    pub fn or_aligned(mut self, aligned: Option<AlignedMultiLine>) -> Self {
        if let Some(new) = aligned {
            self.aligned = match self.aligned {
                Some(old) => {
                    // TODO: Need to keep multiple options here
                    if new.middle < old.middle {
                        Some(new)
                    } else {
                        Some(old)
                    }
                }
                None => Some(new),
            };
        }
        self
    }

    // TODO: rewrite
    /// Combine the best (smallest) options from both Requirements.
    pub fn best(self, other: Self) -> Self {
        self.or_single_line(other.single_line)
            .or_multi_line(other.multi_line)
            .or_aligned(other.aligned)
    }

    pub fn min_first_line_len(&self) -> usize {
        let mut min_len = None;
        let add_option = |x: usize| min_len = min_len.map_or(x, |y| x.min(y));
        if let Some(len) = self.single_line {
            add_option(len);
        }
        if let Some(ml) = self.multi_line.iter().first() {
            add_option(ml.first);
        }
        if let Some(al) = &self.aligned.iter().first() {
            add_option(al.middle);
        }
        min_len.expect("min_first_line_len: no options")
    }
}
