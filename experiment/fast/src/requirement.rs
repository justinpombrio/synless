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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NonChoosyFirstLineLen {
    Single(usize),
    Multi(usize),
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

    #[cfg(test)]
    pub fn with_single_line(mut self, single_line: usize) -> Requirements {
        if let Some(sl) = self.single_line {
            self.single_line = Some(sl.min(single_line))
        } else {
            self.single_line = Some(single_line)
        }
        self
    }

    #[cfg(test)]
    pub fn with_multi_line(mut self, multi_line: MultiLine) -> Requirements {
        self.multi_line.insert(multi_line);
        self
    }

    #[cfg(test)]
    pub fn with_aligned(mut self, aligned: Aligned) -> Requirements {
        self.aligned.insert(aligned);
        self
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
            ml.first += indent;
            ml.middle += indent;
            ml.last += indent;
            // TODO can we use unchecked insert, or iterate over mutable refs?
            // is the order guaranteed to be the same?
            self.multi_line.insert(ml);
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

    pub fn nest(&self, indent: usize, other: Requirements) -> Self {
        let other = other.indent(indent);

        let mut req = Requirements::new();

        if let (Some(ls), Some(rs)) = (self.single_line, other.single_line) {
            req.multi_line.insert(MultiLine {
                first: ls,
                middle: 0,
                last: rs,
            });
        }

        if let Some(ls) = self.single_line {
            for rm in &other.multi_line {
                req.multi_line.insert(MultiLine {
                    first: ls,
                    middle: rm.first.max(rm.middle),
                    last: rm.last,
                });
            }
        }

        if let Some(rs) = other.single_line {
            for lm in &self.multi_line {
                req.multi_line.insert(MultiLine {
                    first: lm.first,
                    middle: lm.middle.max(lm.last),
                    last: rs,
                });
            }
        }

        for lm in &self.multi_line {
            for rm in &other.multi_line {
                req.multi_line.insert(MultiLine {
                    first: lm.first,
                    middle: lm.middle.max(lm.last).max(rm.first).max(rm.middle),
                    last: rm.last,
                });
            }
        }

        if let Some(ls) = self.single_line {
            for ra in &other.aligned {
                req.multi_line.insert(MultiLine {
                    first: ls,
                    middle: ra.middle,
                    last: ra.last,
                });
            }
        }

        if let Some(rs) = other.single_line {
            for la in &self.aligned {
                req.multi_line.insert(MultiLine {
                    first: la.middle.max(la.last),
                    middle: 0, // TODO 0? the smaller of la.middle and la.last? does it matter?
                    last: rs,
                });
            }
        }

        for la in &self.aligned {
            for ra in &other.aligned {
                req.multi_line.insert(MultiLine {
                    first: la.middle.max(la.last),
                    middle: ra.middle,
                    last: ra.last,
                });
            }
        }

        for lm in &self.multi_line {
            for ra in &other.aligned {
                req.multi_line.insert(MultiLine {
                    first: lm.first,
                    middle: lm.middle.max(lm.last).max(ra.middle),
                    last: ra.last,
                });
            }
        }

        for la in &self.aligned {
            for rm in &other.multi_line {
                req.multi_line.insert(MultiLine {
                    first: la.middle.max(la.last),
                    middle: rm.first.max(rm.middle),
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
        // Make sure this matches the x and y chosen when implementing the `Stair` trait!
        if let Some(ml) = self.multi_line.min_by_x() {
            add_option(ml.first);
        }
        if let Some(al) = self.aligned.min_by_x() {
            add_option(al.middle);
        }
        min_len
    }

    /// Panics if there's more than one possible first line
    pub fn non_choosy_first_line_len(&self) -> NonChoosyFirstLineLen {
        if !self.aligned.is_empty() {
            panic!("non_choosy_first_line_len: found an aligned possibility");
        }
        match self.multi_line.len() {
            0 => NonChoosyFirstLineLen::Single(
                self.single_line
                    .expect("non_choosy_first_line_len: impossible requirements"),
            ),
            1 => {
                assert!(
                    self.single_line.is_none(),
                    "non_choosy_first_line_len: found both multi and single line possibilities"
                );
                NonChoosyFirstLineLen::Multi(self.multi_line.iter().next().unwrap().first)
            }
            _ => panic!("non_choosy_first_line_len: found multiple multi-line possibilities"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_possible() {
        assert!(!Requirements::new().is_possible());
        assert!(Requirements::new_single_line(4).is_possible());
        assert!(Requirements::new()
            .with_multi_line(MultiLine {
                first: 1,
                middle: 2,
                last: 3
            })
            .is_possible());
        assert!(Requirements::new()
            .with_aligned(Aligned { middle: 2, last: 3 })
            .is_possible());
    }

    fn example_req() -> Requirements {
        Requirements::new_single_line(10)
            .with_multi_line(MultiLine {
                first: 2,
                middle: 6,
                last: 3,
            })
            .with_multi_line(MultiLine {
                first: 3,
                middle: 5,
                last: 4,
            })
            .with_aligned(Aligned { middle: 3, last: 4 })
            .with_aligned(Aligned { middle: 4, last: 3 })
    }

    #[test]
    fn test_indent() {
        let req = example_req();
        let expected = Requirements::new_single_line(10)
            .with_multi_line(MultiLine {
                first: 102,
                middle: 106,
                last: 103,
            })
            .with_multi_line(MultiLine {
                first: 103,
                middle: 105,
                last: 104,
            })
            .with_aligned(Aligned { middle: 3, last: 4 })
            .with_aligned(Aligned { middle: 4, last: 3 });

        assert_eq!(req.indent(100), expected);
    }

    fn assert_fits_exactly(width: usize, req: Requirements) {
        assert!(req.fits(width + 1));
        assert!(req.fits(width));
        assert!(!req.fits(width - 1));
    }

    #[test]
    fn test_fits() {
        assert_fits_exactly(6, Requirements::new_single_line(6));

        assert_fits_exactly(
            6,
            Requirements::new().with_multi_line(MultiLine {
                first: 5,
                middle: 6,
                last: 5,
            }),
        );

        assert_fits_exactly(
            6,
            Requirements::new().with_multi_line(MultiLine {
                first: 6,
                middle: 5,
                last: 5,
            }),
        );

        assert_fits_exactly(
            6,
            Requirements::new().with_multi_line(MultiLine {
                first: 5,
                middle: 5,
                last: 6,
            }),
        );

        assert_fits_exactly(
            6,
            Requirements::new()
                .with_multi_line(MultiLine {
                    first: 5,
                    middle: 5,
                    last: 6,
                })
                .with_multi_line(MultiLine {
                    first: 10,
                    middle: 1,
                    last: 10,
                }),
        );

        assert_fits_exactly(
            6,
            Requirements::new().with_aligned(Aligned { middle: 6, last: 5 }),
        );

        assert_fits_exactly(
            6,
            Requirements::new().with_aligned(Aligned { middle: 5, last: 6 }),
        );

        assert_fits_exactly(
            6,
            Requirements::new()
                .with_aligned(Aligned {
                    middle: 10,
                    last: 1,
                })
                .with_aligned(Aligned { middle: 5, last: 6 }),
        );
    }

    #[test]
    fn test_min_first_line() {
        assert_eq!(Requirements::new().min_first_line_len(), None);
        assert_eq!(
            Requirements::new_single_line(5).min_first_line_len(),
            Some(5)
        );

        assert_eq!(
            Requirements::new_single_line(5)
                .with_multi_line(MultiLine {
                    first: 4,
                    middle: 0,
                    last: 0,
                })
                .min_first_line_len(),
            Some(4)
        );

        assert_eq!(
            Requirements::new_single_line(5)
                .with_multi_line(MultiLine {
                    first: 6,
                    middle: 0,
                    last: 0,
                })
                .min_first_line_len(),
            Some(5)
        );

        assert_eq!(
            Requirements::new_single_line(5)
                .with_multi_line(MultiLine {
                    first: 6,
                    middle: 2,
                    last: 0,
                })
                .with_multi_line(MultiLine {
                    first: 4,
                    middle: 1,
                    last: 0,
                })
                .min_first_line_len(),
            Some(4)
        );

        assert_eq!(
            Requirements::new_single_line(5)
                .with_multi_line(MultiLine {
                    first: 6,
                    middle: 1,
                    last: 0,
                })
                .with_multi_line(MultiLine {
                    first: 4,
                    middle: 2,
                    last: 0,
                })
                .min_first_line_len(),
            Some(4)
        );

        assert_eq!(
            Requirements::new_single_line(5)
                .with_aligned(Aligned { middle: 4, last: 0 })
                .min_first_line_len(),
            Some(4)
        );
        assert_eq!(
            Requirements::new_single_line(5)
                .with_aligned(Aligned { middle: 4, last: 1 })
                .with_aligned(Aligned { middle: 6, last: 2 })
                .min_first_line_len(),
            Some(4)
        );
        assert_eq!(
            Requirements::new_single_line(5)
                .with_aligned(Aligned { middle: 4, last: 2 })
                .with_aligned(Aligned { middle: 6, last: 1 })
                .min_first_line_len(),
            Some(4)
        );

        assert_eq!(
            Requirements::new_single_line(5)
                .with_aligned(Aligned { middle: 3, last: 2 })
                .with_multi_line(MultiLine {
                    first: 4,
                    middle: 2,
                    last: 0,
                })
                .min_first_line_len(),
            Some(3)
        );

        assert_eq!(
            Requirements::new_single_line(5)
                .with_aligned(Aligned { middle: 4, last: 2 })
                .with_multi_line(MultiLine {
                    first: 3,
                    middle: 2,
                    last: 0,
                })
                .min_first_line_len(),
            Some(3)
        );

        assert_eq!(
            Requirements::new_single_line(3)
                .with_aligned(Aligned { middle: 4, last: 2 })
                .with_multi_line(MultiLine {
                    first: 5,
                    middle: 2,
                    last: 0,
                })
                .min_first_line_len(),
            Some(3)
        );
    }
}
