use super::measure::MeasuredNotation;
use super::requirement::{NonChoosyFirstLineLen, Requirements};
use MeasuredNotation::*;

pub fn pretty_print(notation: &MeasuredNotation, width: usize) -> Vec<(usize, String)> {
    let mut printer = PrettyPrinter {
        lines: vec![(0, "".to_string())],
        width,
    };
    printer.pp(Some(0), 0, notation);
    printer.lines
}

// TODO:
// - make pp a method on `prefix`
// - helper method: self.prefix_len()

struct PrettyPrinter {
    /// INVARIANT: Never empty
    lines: Vec<(usize, String)>,
    width: usize,
}

impl PrettyPrinter {
    fn prefix_len(&self) -> usize {
        let (i, s) = self.lines.last().unwrap();
        i + s.chars().count()
    }

    // If indent is None, we're inside a Flat
    fn pp(&mut self, indent: Option<usize>, suffix_len: usize, notation: &MeasuredNotation) {
        match notation {
            Literal(text) => {
                self.lines.last_mut().unwrap().1.push_str(text);
            }
            Flat(note) => {
                self.pp(None, suffix_len, note);
            }
            Align(note) => {
                let indent = indent.map(|i| i + self.prefix_len());
                self.pp(indent, suffix_len, note)
            }
            Nest(left, i, right) => {
                if indent.is_none() {
                    unreachable!();
                }
                self.pp(indent, 0, left);
                let new_indent = indent.map(|j| j + i);
                self.lines.push((new_indent.unwrap(), String::new()));
                self.pp(new_indent, suffix_len, right);
            }
            Concat(left, right, _, non_choosy_right_first_line_len) => {
                let new_suffix_len = match non_choosy_right_first_line_len {
                    None => 666, // this shouldn't actually get used, because left isn't choosy
                    Some(NonChoosyFirstLineLen::Multi(len)) => *len,
                    Some(NonChoosyFirstLineLen::Single(len)) => *len + suffix_len,
                };
                self.pp(indent, new_suffix_len, left);
                self.pp(indent, suffix_len, right);
            }
            Choice((left, left_req), (right, right_req)) => {
                let prefix_req = Requirements::new_single_line(self.prefix_len());
                let suffix_req = Requirements::new_single_line(suffix_len);

                let full_left_req = match indent {
                    None => prefix_req.concat(left_req).concat(&suffix_req),
                    Some(i) => prefix_req
                        .concat(&left_req.clone().indent(i))
                        .concat(&suffix_req),
                };

                let left_fits = full_left_req.fits(self.width);
                let right_impossible = !right_req.is_possible();
                if left_fits || right_impossible {
                    self.pp(indent, suffix_len, left)
                } else {
                    self.pp(indent, suffix_len, right)
                }
            }
        }
    }
}
