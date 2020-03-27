use super::measure::{LineLength, MeasuredNotation, Shapes};

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
            Empty => (),
            Literal(text) => {
                self.lines.last_mut().unwrap().1.push_str(text);
            }
            Flat(note) => {
                self.pp(note, None, prefix_len, suffix_len);
            }
            Align(note) => {
                let new_indent = indent.and(prefix_len);
                self.pp(note, new_indent, prefix_len, suffix_len)
            }
            Indent(j, note) => {
                self.pp(note, indent.map(|i| i + j), prefix_len, suffix_len);
            }
            Vert(left, right) => {
                let indent = indent.expect("Vert in Flat");
                self.pp(left, Some(indent), prefix_len, Some(0));
                self.lines.push((indent, String::new()));
                self.pp(right, Some(indent), Some(indent), suffix_len);
            }
            Concat(left, right, known_line_lens) => {
                let middle_suffix_len = match known_line_lens.right_first_line {
                    None => None,
                    Some(LineLength::Single(len)) => suffix_len.map(|s| len + s),
                    Some(LineLength::Multi(len)) => Some(len),
                };
                let middle_prefix_len = match known_line_lens.left_last_line {
                    None => None,
                    Some(LineLength::Single(len)) => prefix_len.map(|p| p + len),
                    Some(LineLength::Multi(len)) => indent.map(|i| i + len),
                };
                self.pp(left, indent, prefix_len, middle_suffix_len);
                self.pp(right, indent, middle_prefix_len, suffix_len);
            }
            Choice((left, left_shapes), (right, right_shapes)) => {
                // TODO: avoid clone
                let mut left_shapes = left_shapes.to_owned();
                let mut right_shapes = right_shapes.to_owned();
                if indent == None {
                    left_shapes = left_shapes.flat();
                    right_shapes = right_shapes.flat();
                }
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
                    .concat(left_shapes)
                    .concat(suffix_shape)
                    .indent(indent.unwrap_or(0));

                if full_left_shapes.fits(self.width) {
                    self.pp(left, indent, prefix_len, suffix_len);
                } else {
                    self.pp(right, indent, prefix_len, suffix_len);
                }
            }
        }
    }
}
