use super::measure::{KnownLineLengths, LineLength, MeasuredNotation, Shapes};

/// Display the notation, using at most `width` columns if at all possible.
/// Returns a list of `(indent, line)` pairs, where `indent` is the number of
/// spaces that should precede `line`.
pub fn pretty_print(notation: &MeasuredNotation, width: usize) -> Vec<(usize, String)> {
    let mut printer = PrettyPrinter {
        lines: vec![(0, "".to_string())],
        width,
    };
    printer.pp(notation, Some(0), Some(0), Some(0));
    printer.lines
}

struct PrettyPrinter {
    /// INVARIANT: Never empty
    lines: Vec<(usize, String)>,
    width: usize,
}

impl PrettyPrinter {
    /// - `indent` is `None` iff we're inside a Flat.
    /// - `prefix_len` and `suffix_len` are `None` when they are unknown because
    ///   there's a choosy notation there.
    fn pp(
        &mut self,
        notation: &MeasuredNotation,
        indent: Option<usize>,
        prefix_len: Option<usize>,
        suffix_len: Option<usize>,
    ) {
        use MeasuredNotation::*;

        match notation {
            Literal(text) => {
                self.lines.last_mut().unwrap().1.push_str(text);
            }
            Flat(note) => {
                self.pp(note, None, prefix_len, suffix_len);
            }
            Align(note) => {
                let indent = indent.map(|i| i + prefix_len.expect("Too choosy! (Align)"));
                self.pp(note, indent, prefix_len, suffix_len)
            }
            Nest(j, note) => {
                let new_indent = indent.expect("Nest in Flat") + j;
                self.lines.push((new_indent, String::new()));
                self.pp(note, Some(new_indent), Some(new_indent), suffix_len);
            }
            Concat(left, right, known_line_lens) => match known_line_lens {
                KnownLineLengths::Left { left_last_line } => {
                    self.pp(left, indent, prefix_len, None);
                    let middle_prefix_len = match left_last_line {
                        LineLength::Single(len) => prefix_len.expect("Too choosy! Concat") + len,
                        LineLength::Multi(len) => indent.expect("Multi in Flat") + len,
                    };
                    self.pp(right, indent, Some(middle_prefix_len), suffix_len);
                }
                KnownLineLengths::Right { right_first_line } => {
                    let middle_suffix_len = match right_first_line {
                        LineLength::Single(len) => *len + suffix_len.expect("Too choosy! Concat"),
                        LineLength::Multi(len) => *len,
                    };
                    self.pp(left, indent, prefix_len, Some(middle_suffix_len));
                    self.pp(right, indent, None, suffix_len);
                }
                KnownLineLengths::Both { .. } => {
                    self.pp(left, indent, prefix_len, None);
                    self.pp(right, indent, None, suffix_len);
                }
            },
            Choice((left, left_shapes), (right, right_shapes)) => {
                // TODO: avoid clone
                let (left_shapes, right_shapes) = if indent == None {
                    (left_shapes.clone().flat(), right_shapes.clone().flat())
                } else {
                    (left_shapes.clone(), right_shapes.clone())
                };
                if !left_shapes.is_possible() {
                    self.pp(right, indent, prefix_len, suffix_len);
                    return;
                }
                if !right_shapes.is_possible() {
                    self.pp(left, indent, prefix_len, suffix_len);
                    return;
                }

                let prefix_shape = Shapes::new_single_line(prefix_len.expect("Too choosy! Choice"));
                let suffix_shape = Shapes::new_single_line(suffix_len.expect("Too choosy! Choice"));

                let full_left_shapes = prefix_shape
                    .concat(left_shapes.clone().indent(indent.unwrap_or(0)))
                    .concat(suffix_shape);

                if full_left_shapes.fits(self.width) {
                    self.pp(left, indent, prefix_len, suffix_len);
                } else {
                    self.pp(right, indent, prefix_len, suffix_len);
                }
            }
        }
    }
}
