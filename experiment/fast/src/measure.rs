use super::notation::Notation;
use crate::staircase::{Stair, Staircase};

#[derive(Clone, Debug)]
pub enum MeasuredNotation {
    Empty,
    Literal(String),
    Indent(usize, Box<MeasuredNotation>),
    Vert(Box<MeasuredNotation>, Box<MeasuredNotation>),
    Flat(Box<MeasuredNotation>),
    Align(Box<MeasuredNotation>),
    Concat(
        Box<MeasuredNotation>,
        Box<MeasuredNotation>,
        KnownLineLengths,
    ),
    Choice(
        (Box<MeasuredNotation>, Shapes),
        (Box<MeasuredNotation>, Shapes),
    ),
}

/// A set of shapes that a Notation fits inside. Since a Notation may be
/// embedded inside other Notations, and does not know beforehand what those may
/// be, it must keep track of a few categories of shapes: those on a single
/// line, those that use multiple lines, and those that are aligned.
///
/// For efficiency, within each category, only the smallest shapes are stored:
/// if one shape fits inside another, only the former is stored.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shapes {
    /// The minimum number of characters worth of space required for displaying
    /// the Notation on a single line. `None` if it cannot be displayed on a
    /// single line.
    pub single_line: Option<usize>,
    /// The set of minimal amounts of space required for displaying the
    /// Notation, when it is displayed across multiple lines and is not aligned.
    pub multi_line: Staircase<MultiLineShape>,
    /// The set of minimal amounts of space required for displaying the
    /// Notation, when it is displayed across multiple lines and is aligned.
    pub aligned: Staircase<AlignedShape>,
}

/// The space required for one way of laying out a Notation, across multiple
/// lines, while not being aligned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MultiLineShape {
    /// The number of characters required for the first line.
    pub first: usize,
    /// The number of characters required for the last line.
    pub last: usize,
}

/// The space required for one way of laying out a Notation, across multiple
/// lines, while being aligned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AlignedShape {
    /// The maximum number of characters required for any line except the last.
    pub non_last: usize,
    /// The number of characters required for the last line.
    pub last: usize,
}

/// The known line lengths of the two children of a Concat node. A line length
/// can only be known if that line is not choosy. If it is choosy the length is
/// ambiguous because making the choice in different ways could result in
/// different lengths.
///
/// `left_last_line` and `right_first_line` cannot both be `None`, because that
/// would only happen if _both_ children were choosy, which is illegal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownLineLengths {
    pub left_last_line: Option<LineLength>,
    pub right_first_line: Option<LineLength>,
}

/// The length of a line, and whether it is a single line, or part of a
/// multi-line layout.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LineLength {
    Single(usize),
    Multi(usize),
}

// TODO: Update prose
/// MultiLineShapes have three dimensions of variation, but it's only practical to
/// store tradeoffs between two of them. We therefore store the tradeoff between
/// `first` and `middle`, and require that notation authors follow the "No
/// Tradeoff" rule, which says that for all Multilines `x` and `y` of a Notation:
///
/// > cmp(x.first, y.first) == cmp(x.last, y.last)
impl Stair for MultiLineShape {
    fn x(&self) -> usize {
        self.first
    }
    fn y(&self) -> usize {
        self.last
    }
}

/// AlignedShapes have two dimensions of variation; we store the tradeoff between the
/// two in a Staircase.
impl Stair for AlignedShape {
    fn x(&self) -> usize {
        self.non_last
    }
    fn y(&self) -> usize {
        self.last
    }
}

impl MultiLineShape {
    /// Insert `indent` spaces to the left of all of the lines but the first.
    pub fn indent(mut self, indent: usize) -> Self {
        self.last += indent;
        self
    }
}

impl AlignedShape {
    /// Insert `indent` spaces to the left of all of the lines but the first.
    pub fn indent(self, _indent: usize) -> Self {
        // Everything is aligned based on the _first_ line.
        // First line doesn't move -> nothing moves.
        self
    }
}

impl Notation {
    /// "Measure" a Notation, computing information such as the set of smallest
    /// shapes it fits inside and some of its line lengths. This information is
    /// stored in the [`MeasuredNotation`](MeasuredNotation) for later use by
    /// pretty printing.
    pub fn measure(&self) -> MeasuredNotation {
        self.measure_rec().0
    }

    fn measure_rec(&self) -> (MeasuredNotation, Shapes) {
        match self {
            Notation::Empty => (MeasuredNotation::Empty, Shapes::new_single_line(0)),
            Notation::Literal(lit) => {
                let note = MeasuredNotation::Literal(lit.clone());
                let shapes = Shapes::new_single_line(lit.chars().count());
                (note, shapes)
            }
            Notation::Flat(note) => {
                let (note, shapes) = note.measure_rec();
                let note = MeasuredNotation::Flat(Box::new(note));
                let shapes = shapes.flat();
                (note, shapes)
            }
            Notation::Align(note) => {
                let (note, mut shapes) = note.measure_rec();
                let note = MeasuredNotation::Align(Box::new(note));
                for ml in shapes.multi_line.drain() {
                    shapes.aligned.insert(AlignedShape {
                        non_last: ml.first,
                        last: ml.last,
                    })
                }
                (note, shapes)
            }
            Notation::Indent(indent, note) => {
                let (note, shapes) = note.measure_rec();
                (
                    MeasuredNotation::Indent(*indent, Box::new(note)),
                    shapes.indent(*indent),
                )
            }
            Notation::Vert(left, right) => {
                let (left_note, left_shapes) = left.measure_rec();
                let (right_note, right_shapes) = right.measure_rec();
                (
                    MeasuredNotation::Vert(Box::new(left_note), Box::new(right_note)),
                    left_shapes.vert(right_shapes),
                )
            }
            Notation::Concat(left, right) => {
                let (left_note, left_shapes) = left.measure_rec();
                let (right_note, right_shapes) = right.measure_rec();
                let known_line_lens = KnownLineLengths {
                    left_last_line: left_shapes.known_last_line_len(),
                    right_first_line: right_shapes.known_first_line_len(),
                };
                let note = MeasuredNotation::Concat(
                    Box::new(left_note),
                    Box::new(right_note),
                    known_line_lens,
                );
                let shapes = left_shapes.concat(right_shapes);
                (note, shapes)
            }
            Notation::Choice(left, right) => {
                let (left_note, left_shapes) = left.measure_rec();
                let (right_note, right_shapes) = right.measure_rec();
                // TODO avoid cloning?
                let note = MeasuredNotation::Choice(
                    (Box::new(left_note), left_shapes.clone()),
                    (Box::new(right_note), right_shapes.clone()),
                );
                let shapes = left_shapes.union(right_shapes);
                (note, shapes)
            }
        }
    }
}

impl Shapes {
    /// Construct an "empty" set of shapes.
    /// (This is the `Shapes` of the impossible notation.)
    pub fn new() -> Shapes {
        Shapes {
            single_line: None,
            multi_line: Staircase::new(),
            aligned: Staircase::new(),
        }
    }

    /// The shape of a Notation that can only be laid out on a single
    /// line with the given number of characters.
    pub fn new_single_line(len: usize) -> Shapes {
        Shapes {
            single_line: Some(len),
            multi_line: Staircase::new(),
            aligned: Staircase::new(),
        }
    }

    /// The shape of a newline.
    /// (This is the `Shapes` of `Vert(Literal(""), Literal(""))`.)
    pub fn new_newline() -> Shapes {
        let mut multi_line = Staircase::new();
        multi_line.insert(MultiLineShape { first: 0, last: 0 });
        Shapes {
            single_line: None,
            multi_line,
            aligned: Staircase::new(),
        }
    }

    #[cfg(test)]
    pub fn with_single_line(mut self, single_line: usize) -> Shapes {
        if let Some(sl) = self.single_line {
            self.single_line = Some(sl.min(single_line))
        } else {
            self.single_line = Some(single_line)
        }
        self
    }

    #[cfg(test)]
    pub fn with_multi_line(mut self, multi_line: MultiLineShape) -> Shapes {
        self.multi_line.insert(multi_line);
        self
    }

    #[cfg(test)]
    pub fn with_aligned(mut self, aligned: AlignedShape) -> Shapes {
        self.aligned.insert(aligned);
        self
    }

    /// Do _any_ of the shapes fit within this width?
    pub fn fits(&self, width: usize) -> bool {
        if let Some(sl) = self.single_line {
            if sl <= width {
                return true;
            }
        }
        // TODO: could be more efficient
        for ml in &self.multi_line {
            if ml.first <= width && ml.last <= width {
                return true;
            }
        }
        for al in &self.aligned {
            if al.non_last <= width && al.last <= width {
                return true;
            }
        }
        false
    }

    /// Does the Notation have at least one possible layout?
    pub fn is_possible(&self) -> bool {
        self.single_line.is_some() || !self.multi_line.is_empty() || !self.aligned.is_empty()
    }

    /// Returns the Shapes for `Flat(n)`,
    /// where `self` is the Shapes for a Notation `n`.
    pub fn flat(mut self) -> Self {
        self.multi_line = Staircase::new();
        self.aligned = Staircase::new();
        self
    }

    /// Insert `indent` spaces to the left of all of the lines but the first.
    /// This is the Shapes for `Indent(usize, n)`, where `self` is the Shapes for Notation `n`.
    pub fn indent(mut self, indent: usize) -> Self {
        let multi_lines = self.multi_line.into_iter();
        self.multi_line = Staircase::new();
        for ml in multi_lines {
            // TODO: unchecked insert?
            self.multi_line.insert(ml.indent(indent));
        }

        let aligneds = self.aligned.into_iter();
        self.aligned = Staircase::new();
        for al in aligneds {
            // TODO: unchecked insert?
            self.aligned.insert(al.indent(indent));
        }

        self
    }

    /// Returns the Shapes for `Vert(n, m)`,
    /// where `self` and `other` are the Shapes for Notations `n` and `m` respectively.
    pub fn vert(self, other: Shapes) -> Self {
        self.concat(Shapes::new_newline()).concat(other)
    }

    /// Returns the Shapes for `Concat(n, m)`,
    /// where `self` and `other` are the Shapes for Notations `n` and `m` respectively.
    pub fn concat(self, other: Shapes) -> Self {
        let mut shapes = Shapes::new();

        if let (Some(ls), Some(rs)) = (self.single_line, other.single_line) {
            shapes.single_line = Some(ls + rs);
        }

        if let Some(ls) = self.single_line {
            for rm in &other.multi_line {
                shapes.multi_line.insert(MultiLineShape {
                    first: ls + rm.first,
                    last: rm.last,
                });
            }
        }

        if let Some(rs) = other.single_line {
            for lm in &self.multi_line {
                shapes.multi_line.insert(MultiLineShape {
                    first: lm.first,
                    last: lm.last + rs,
                });
            }
        }

        for lm in &self.multi_line {
            for rm in &other.multi_line {
                shapes.multi_line.insert(MultiLineShape {
                    first: lm.first,
                    last: rm.last,
                });
            }
        }

        if let Some(ls) = self.single_line {
            for ra in &other.aligned {
                shapes.aligned.insert(AlignedShape {
                    non_last: ls + ra.non_last,
                    last: ls + ra.last,
                });
            }
        }

        if let Some(rs) = other.single_line {
            for la in &self.aligned {
                shapes.aligned.insert(AlignedShape {
                    non_last: la.non_last,
                    last: la.last + rs,
                });
            }
        }

        for la in &self.aligned {
            for ra in &other.aligned {
                shapes.aligned.insert(AlignedShape {
                    non_last: la.non_last.max(la.last + ra.non_last),
                    last: la.last + ra.last,
                });
            }
        }

        for lm in &self.multi_line {
            for ra in &other.aligned {
                shapes.multi_line.insert(MultiLineShape {
                    first: lm.first,
                    last: lm.last + ra.last,
                });
            }
        }

        for la in &self.aligned {
            for rm in &other.multi_line {
                shapes.multi_line.insert(MultiLineShape {
                    // Eh, this is _basically_ true
                    first: la.non_last,
                    last: rm.last,
                });
            }
        }
        shapes
    }

    /// Take the union of two sets of shapes.
    ///
    /// (For efficiency, any shape that is strictly larger than another in the
    /// same category is tossed out.)
    pub fn union(mut self, other: Self) -> Self {
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

    /// The length of the first line, if it is not choosy.
    fn known_first_line_len(&self) -> Option<LineLength> {
        if !self.aligned.is_empty() {
            return None;
        }
        if let Some(sl) = self.single_line {
            if !self.multi_line.is_empty() {
                None
            } else {
                Some(LineLength::Single(sl))
            }
        } else {
            self.sole_multiline().map(|ml| LineLength::Multi(ml.first))
        }
    }

    /// The length of the last line, if it is not choosy.
    fn known_last_line_len(&self) -> Option<LineLength> {
        if !self.aligned.is_empty() {
            return None;
        }
        if let Some(sl) = self.single_line {
            if !self.multi_line.is_empty() {
                None
            } else {
                Some(LineLength::Single(sl))
            }
        } else {
            self.sole_multiline().map(|ml| LineLength::Multi(ml.last))
        }
    }

    fn sole_multiline(&self) -> Option<MultiLineShape> {
        if self.multi_line.len() == 1 {
            Some(*self.multi_line.iter().next().unwrap())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notation::Notation;

    #[test]
    fn test_possible() {
        assert!(!Shapes::new().is_possible());
        assert!(Shapes::new_single_line(4).is_possible());
        assert!(Shapes::new()
            .with_multi_line(MultiLineShape { first: 1, last: 3 })
            .is_possible());
        assert!(Shapes::new()
            .with_aligned(AlignedShape {
                non_last: 2,
                last: 3
            })
            .is_possible());
    }

    fn example_shapes() -> Shapes {
        Shapes::new_single_line(10)
            .with_multi_line(MultiLineShape { first: 2, last: 3 })
            .with_multi_line(MultiLineShape { first: 3, last: 4 })
            .with_aligned(AlignedShape {
                non_last: 3,
                last: 4,
            })
            .with_aligned(AlignedShape {
                non_last: 4,
                last: 3,
            })
    }

    #[test]
    fn test_indent() {
        let shapes = example_shapes();
        let expected = Shapes::new_single_line(10)
            .with_multi_line(MultiLineShape {
                first: 2,
                last: 103,
            })
            .with_multi_line(MultiLineShape {
                first: 3,
                last: 104,
            })
            .with_aligned(AlignedShape {
                non_last: 3,
                last: 4,
            })
            .with_aligned(AlignedShape {
                non_last: 4,
                last: 3,
            });

        assert_eq!(shapes.indent(100), expected);
    }

    fn assert_fits_exactly(width: usize, shapes: Shapes) {
        assert!(shapes.fits(width + 1));
        assert!(shapes.fits(width));
        assert!(!shapes.fits(width - 1));
    }

    #[test]
    fn test_fits() {
        assert_fits_exactly(6, Shapes::new_single_line(6));

        assert_fits_exactly(
            5,
            Shapes::new().with_multi_line(MultiLineShape { first: 5, last: 5 }),
        );

        assert_fits_exactly(
            6,
            Shapes::new().with_multi_line(MultiLineShape { first: 6, last: 5 }),
        );

        assert_fits_exactly(
            6,
            Shapes::new().with_multi_line(MultiLineShape { first: 5, last: 6 }),
        );

        assert_fits_exactly(
            6,
            Shapes::new()
                .with_multi_line(MultiLineShape { first: 5, last: 6 })
                .with_multi_line(MultiLineShape {
                    first: 10,
                    last: 10,
                }),
        );

        assert_fits_exactly(
            6,
            Shapes::new().with_aligned(AlignedShape {
                non_last: 6,
                last: 5,
            }),
        );

        assert_fits_exactly(
            6,
            Shapes::new().with_aligned(AlignedShape {
                non_last: 5,
                last: 6,
            }),
        );

        assert_fits_exactly(
            6,
            Shapes::new()
                .with_aligned(AlignedShape {
                    non_last: 10,
                    last: 1,
                })
                .with_aligned(AlignedShape {
                    non_last: 5,
                    last: 6,
                }),
        );
    }

    #[test]
    fn test_literal() {
        let lit = Notation::Literal("foobar".into());
        lit.validate().unwrap();
        let shapes = lit.measure_rec().1;
        assert_eq!(shapes, Shapes::new_single_line(6));
    }

    #[test]
    fn test_concat_literals() {
        let note = Notation::Concat(
            Box::new(Notation::Literal("fooo".to_string())),
            Box::new(Notation::Literal("bar".to_string())),
        );
        note.validate().unwrap();
        let (note, shapes) = note.measure_rec();

        assert_eq!(shapes, Shapes::new_single_line(7));
        match note {
            MeasuredNotation::Concat(_, _, known_line_lens) => match known_line_lens {
                KnownLineLengths {
                    left_last_line,
                    right_first_line,
                } => {
                    assert_eq!(left_last_line, Some(LineLength::Single(4)));
                    assert_eq!(right_first_line, Some(LineLength::Single(3)));
                }
            },
            _ => panic!("Expected MeasuredNotation::Concat variant"),
        }
    }
}
