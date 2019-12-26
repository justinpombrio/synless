use super::measure::MeasuredNotation;
use super::requirement::Requirement;
use MeasuredNotation::*;

pub fn pretty_print(notation: &MeasuredNotation, width: usize) -> Vec<(usize, String)> {
    let prefix = vec![(0, "".to_string())];
    let suffix_req = Requirement::new_single_line(0);
    pp(width, 0, prefix, suffix_req, notation)
}

// TODO:
// - make pp a method on `prefix`
// - helper method: self.prefix_len()

// INVARIANT: prefix is never empty. It should at the very least contain 1 empty string.
fn pp(
    width: usize,
    indent: usize,
    mut prefix: Vec<(usize, String)>,
    suffix_req: Requirement,
    notation: &MeasuredNotation,
) -> Vec<(usize, String)> {
    match notation {
        Literal(text) => {
            prefix.last_mut().unwrap().1.push_str(text);
            prefix
        }
        Newline => {
            prefix.push((indent, "".to_string()));
            prefix
        }
        Indent(i, notation) => pp(width, indent + i, prefix, suffix_req, notation),
        Flat(notation) => {
            let (i, s) = prefix.last().unwrap();
            let prefix_len = i + s.chars().count();
            let min_suffix_len = suffix_req.min_first_line_len();
            let flat = pp_flat(width, prefix_len, min_suffix_len, notation);
            prefix.last_mut().unwrap().1.push_str(&flat);
            prefix
        }
        Concat(left, right, right_req) => {
            let new_suffix_req = right_req.indent(indent).concat(suffix_req);
            let prefix = pp(width, indent, prefix, new_suffix_req, left);
            pp(width, indent, prefix, suffix_req, right)
        }
        Align(note) => {
            let (i, s) = prefix.last().unwrap();
            let indent = i + s.chars().count();
            pp(width, indent, prefix, suffix_req, note)
        }
        Choice((left, left_req), (right, right_req)) => {
            let (i, s) = prefix.last().unwrap();
            let prefix_len = i + s.chars().count();
            let full_left_req = Requirement::new_single_line(prefix_len)
                .concat(left_req.indent(indent))
                .concat(suffix_req);
            let full_right_req = Requirement::new_single_line(prefix_len)
                .concat(right_req.indent(indent))
                .concat(suffix_req);
            let left_fits = full_left_req.fits(width);
            let right_impossible = !full_right_req.is_possible();
            if left_fits || right_impossible {
                pp(width, indent, prefix, suffix_req, left)
            } else {
                pp(width, indent, prefix, suffix_req, right)
            }
        }
    }
}

fn pp_flat(
    width: usize,
    prefix_len: usize,
    suffix_len: usize,
    notation: &MeasuredNotation,
) -> String {
    match notation {
        Literal(text) => text.to_string(),
        Newline => panic!("pp_flat found a newline!"),
        Indent(_, notation) => pp_flat(width, prefix_len, suffix_len, notation),
        Flat(notation) => pp_flat(width, prefix_len, suffix_len, notation),
        Align(notation) => pp_flat(width, prefix_len, suffix_len, notation),
        Concat(left, right, right_req) => {
            let min_right_len = right_req
                .single_line
                .expect("pp_flat found a non-flat right");
            let left_str = pp_flat(width, prefix_len, suffix_len + min_right_len, left);
            let actual_left_len = left_str.chars().count();
            let right_str = pp_flat(width, prefix_len + actual_left_len, suffix_len, right);
            format!("{}{}", left_str, right_str)
        }
        Choice((left, left_req), (right, right_req)) => {
            let left_fits = left_req
                .single_line
                .map_or(false, |l| prefix_len + l + suffix_len <= width);
            let right_impossible = right_req.single_line.is_none();
            if left_fits || right_impossible {
                pp_flat(width, prefix_len, suffix_len, left)
            } else {
                pp_flat(width, prefix_len, suffix_len, right)
            }
        }
    }
}
