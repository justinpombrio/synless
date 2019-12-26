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
        // This only works well if the options left and right obey the No-Tradeoff rule.
        //   (left.first < right.first) -> (left.last <= right.last).
        self.first
    }
    fn y(&self) -> usize {
        self.middle
    }
}

impl Stair for Aligned {
    fn x(&self) -> usize {
        self.middle
    }
    fn y(&self) -> usize {
        self.last
    }
}

impl Requirements {
    pub fn new() -> Requirements {
        Requirements {
            single_line: None,
            multi_line: Staircase::new(),
            aligned: Staircase::new(),
        }
    }

    pub fn new_single_line(len: usize) -> Requirements {
        Requirements {
            single_line: Some(len),
            multi_line: Staircase::new(),
            aligned: Staircase::new(),
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
        // TODO iterate over mutable refs? is the order guaranteed to be the same?
        let multi_lines = self.multi_line.into_iter();
        self.multi_line = Staircase::new();
        for mut ml in multi_lines {
            ml.middle += indent;
            ml.last += indent;
            self.multi_line.unchecked_insert(ml);
        }
        self
    }

    pub fn concat(&self, other: &Requirements) -> Self {
        let mut req = Requirements::new();

        if let (Some(ls), Some(rs)) = (self.single_line, other.single_line) {
            req.single_line = Some(ls + rs);
        }

        if let Some(ls) = self.single_line {
            for rm in &other.multi_line {
                req.multi_line.insert(MultiLine {
                    first: ls + rm.first,
                    middle: rm.middle,
                    last: rm.last,
                });
            }
        }

        if let Some(rs) = other.single_line {
            for lm in &self.multi_line {
                req.multi_line.insert(MultiLine {
                    first: lm.first,
                    middle: lm.middle,
                    last: lm.last + rs,
                });
            }
        }

        for lm in &self.multi_line {
            for rm in &other.multi_line {
                req.multi_line.insert(MultiLine {
                    first: lm.first,
                    middle: lm.middle.max(lm.last + rm.first).max(rm.middle),
                    last: rm.last,
                });
            }
        }

        if let Some(ls) = self.single_line {
            for ra in &other.aligned {
                req.aligned.insert(Aligned {
                    middle: ls + ra.middle,
                    last: ls + ra.last,
                });
            }
        }

        if let Some(rs) = other.single_line {
            for la in &self.aligned {
                req.aligned.insert(Aligned {
                    middle: la.middle,
                    last: la.last + rs,
                });
            }
        }

        for la in &self.aligned {
            for ra in &other.aligned {
                req.aligned.insert(Aligned {
                    middle: la.middle.max(la.last + ra.middle),
                    last: la.last + ra.last,
                });
            }
        }

        for lm in &self.multi_line {
            for ra in &other.aligned {
                req.multi_line.insert(MultiLine {
                    first: lm.first,
                    middle: lm.middle.max(lm.last + ra.middle),
                    last: lm.last + ra.last,
                });
            }
        }

        for la in &self.aligned {
            for rm in &other.multi_line {
                req.multi_line.insert(MultiLine {
                    first: la.middle.max(la.last + rm.first),
                    middle: rm.middle,
                    last: rm.last,
                });
            }
        }
        req
    }

    /// Combine the best (smallest) options from both Requirements.
    pub fn best(mut self, other: Self) -> Self {
        self.single_line = match (self.single_line, other.single_line) {
            (None, None) => None,
            (Some(sl), None) | (None, Some(sl)) => Some(sl),
            (Some(self_sl), Some(other_sl)) => Some(self_sl.min(other_sl)),
        };
        for ml in other.multi_line {
            self.multi_line.insert(ml);
        }
        for al in other.aligned {
            self.aligned.insert(al);
        }
        self
    }

    pub fn min_first_line_len(&self) -> Option<usize> {
        let mut min_len: Option<usize> = None;
        let mut add_option = |x: usize| min_len = Some(min_len.map_or(x, |y| x.min(y)));
        if let Some(len) = self.single_line {
            add_option(len);
        }
        if let Some(ml) = self.multi_line.iter().next() {
            add_option(ml.first);
        }
        if let Some(al) = &self.aligned.iter().next() {
            add_option(al.middle);
        }
        min_len
    }
}
