use super::Notation;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValidationError {
    Impossible,
    TooChoosy,
}

/// There may be several ways to display a Notation. This struct captures
/// whether it can be displayed on a single line, and whether it can be
/// displayed over multiple lines, and what lines might contain choosy nodes in
/// the _worst case_.
#[derive(Clone, Copy, Debug)]
struct Possibilities {
    /// - `None` if the notation cannot be displayed on a single line.
    /// - `Some(false)` if it can be displayed on a single line, and that line
    ///   is guaranteed to not be choosy.
    /// - `Some(true)` if it can be displayed on a single line, and that line
    ///   might be choosy.
    single_line: Option<bool>,
    /// `None` if the notation cannot be displayed across multiple lines.
    multi_line: Option<ChoosyLines>,
}

/// For a notation that can be displayed across multiple lines, could the first
/// or last line be choosy?
#[derive(Clone, Copy, Debug)]
struct ChoosyLines {
    first: bool,
    last: bool,
}

impl Possibilities {
    fn new_impossible() -> Self {
        Self {
            single_line: None,
            multi_line: None,
        }
    }

    fn new_single(single_choosy: bool) -> Self {
        Self {
            single_line: Some(single_choosy),
            multi_line: None,
        }
    }

    fn new_multi(first_choosy: bool, last_choosy: bool) -> Self {
        Self {
            single_line: None,
            multi_line: Some(ChoosyLines {
                first: first_choosy,
                last: last_choosy,
            }),
        }
    }

    fn is_possible(self) -> bool {
        self.single_line.is_some() || self.multi_line.is_some()
    }

    /// _Could_ the first line be choosy? `None` if not possible.
    fn choosy_first(self) -> Option<bool> {
        match (self.single_line, self.multi_line) {
            (Some(_), Some(_)) => Some(true),
            (Some(choosy), None) => Some(choosy),
            (None, Some(ml)) => Some(ml.first),
            (None, None) => None,
        }
    }

    /// _Could_ the last line be choosy? `None` if not possible.
    fn choosy_last(self) -> Option<bool> {
        match (self.single_line, self.multi_line) {
            (Some(_), Some(_)) => Some(true),
            (Some(choosy), None) => Some(choosy),
            (None, Some(ml)) => Some(ml.last),
            (None, None) => None,
        }
    }

    /// Determine the Possibilities of `Choice(A, B)` given the Possibilities of
    /// `A` and the Possibilities of `B`.
    fn choice(self, other: Possibilities) -> Possibilities {
        Possibilities {
            single_line: union_options(self.single_line, other.single_line, |_, _| true),
            multi_line: union_options(self.multi_line, other.multi_line, |_, _| ChoosyLines {
                first: true,
                last: true,
            }),
        }
    }
}

impl Notation {
    /// Validate a notation. This consists of:
    ///
    /// - Ensuring there is at least one layout option for displaying it.
    /// - Ensuring that no two _choosy_ nodes share a line.
    ///
    /// A node is _choosy_ iff:
    ///
    /// 1. It is a `Choice`, and both of its options are possible. (A notation
    ///    can only be _impossible_ if it contains `Flat` of a `Vert`.)
    /// 2. Or it is an `Align`, and contains at least one multi-line layout option.
    ///    (If an `Align` can only be laid out on a single line, then it has no
    ///     effect and can be ignored.)
    pub fn validate(&self) -> Result<(), ValidationError> {
        let poss = self.validate_rec()?;
        if poss.is_possible() {
            Ok(())
        } else {
            Err(ValidationError::Impossible)
        }
    }

    fn validate_rec(&self) -> Result<Possibilities, ValidationError> {
        use Notation::*;

        match self {
            Empty | Literal(_) => Ok(Possibilities::new_single(false)),
            Newline => Ok(Possibilities::new_multi(false, false)),
            Flat(note) => {
                let mut poss = note.validate_rec()?;
                poss.multi_line = None;
                Ok(poss)
            }
            Align(note) => {
                let mut poss = note.validate_rec()?;
                if let Some(multi) = poss.multi_line.as_mut() {
                    multi.first = true;
                    multi.last = true;
                }
                Ok(poss)
            }
            Indent(_indent, note) => note.validate_rec(),
            Concat(left, right) => {
                let left_poss = left.validate_rec()?;
                let right_poss = right.validate_rec()?;

                if left_poss.choosy_last() == Some(true) && right_poss.choosy_first() == Some(true)
                {
                    return Err(ValidationError::TooChoosy);
                }

                let mut poss = Possibilities::new_impossible();
                if let (Some(ls), Some(rs)) = (left_poss.single_line, right_poss.single_line) {
                    poss = poss.choice(Possibilities::new_single(ls || rs));
                }
                if let (Some(ls), Some(rm)) = (left_poss.single_line, right_poss.multi_line) {
                    poss = poss.choice(Possibilities::new_multi(ls || rm.first, rm.last));
                }
                if let (Some(lm), Some(rs)) = (left_poss.multi_line, right_poss.single_line) {
                    poss = poss.choice(Possibilities::new_multi(lm.first, lm.last || rs));
                }
                if let (Some(lm), Some(rm)) = (left_poss.multi_line, right_poss.multi_line) {
                    poss = poss.choice(Possibilities::new_multi(lm.first, rm.last));
                }

                Ok(poss)
            }
            Choice(left, right) => {
                let left_poss = left.validate_rec()?;
                let right_poss = right.validate_rec()?;
                Ok(left_poss.choice(right_poss))
            }
        }
    }
}

fn union_options<T, F>(opt_a: Option<T>, opt_b: Option<T>, combine: F) -> Option<T>
where
    F: Fn(T, T) -> T,
{
    match (opt_a, opt_b) {
        (None, None) => None,
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (Some(a), Some(b)) => Some(combine(a, b)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lit(s: &str) -> Notation {
        Notation::Literal(s.into())
    }

    fn newline() -> Notation {
        Notation::Newline
    }

    fn flat(note: Notation) -> Notation {
        Notation::Flat(Box::new(note))
    }

    fn nest(indent: usize, note: Notation) -> Notation {
        Notation::Indent(
            indent,
            Box::new(Notation::Concat(
                Box::new(Notation::Newline),
                Box::new(note),
            )),
        )
    }

    #[test]
    fn test_impossible_flat() {
        let note = lit("foo") + lit("bar");
        note.validate().unwrap();

        let note = lit("foo") + nest(4, lit("bar"));
        note.validate().unwrap();

        let note = flat(lit("foo") + nest(4, lit("bar")));
        assert_eq!(note.validate(), Err(ValidationError::Impossible));
    }

    #[test]
    fn test_choosy() {
        let note = lit("foo") | lit("bar");
        note.validate().unwrap();

        let note = (lit("foo") | lit("bar")) + lit("red");
        note.validate().unwrap();

        let note = lit("foo") + (lit("red") | lit("blue"));
        note.validate().unwrap();

        let note = (lit("foo") | lit("bar")) + (lit("red") | lit("blue"));
        assert_eq!(note.validate(), Err(ValidationError::TooChoosy));

        let note = flat(lit("foo") + newline() + lit("bar") | lit("baz"));
        note.validate().unwrap();

        let note = flat(lit("first") + newline() + lit("second") | lit("1") + newline() + lit("2"));
        assert_eq!(note.validate(), Err(ValidationError::Impossible));
    }
}
