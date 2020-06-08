use super::measure::{LineLength, MeasuredNotation};

/// Display the notation, using at most `width` columns if possible.
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
            Empty(_) => (),
            Literal(_, text) => {
                self.lines.last_mut().unwrap().1.push_str(text);
            }
            Newline(_) => {
                let indent = indent.expect("Newline in Flat");
                self.lines.push((indent, String::new()));
            }
            Flat(_, note) => {
                self.pp(note, None, prefix_len, suffix_len);
            }
            Align(_, note) => {
                let new_indent = indent.and(prefix_len);
                self.pp(note, new_indent, prefix_len, suffix_len)
            }
            Indent(_, j, note) => {
                self.pp(note, indent.map(|i| i + j), prefix_len, suffix_len);
            }
            Concat(_, left, right, known_line_lens) => {
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
            Choice(_, choice) => {
                let notation = choice.choose(indent, prefix_len, suffix_len, self.width);
                self.pp(notation, indent, prefix_len, suffix_len);
            }
        }
    }
}
