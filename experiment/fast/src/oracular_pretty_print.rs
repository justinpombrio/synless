//! Slow (exp time), but definitely correct, pretty printer.

use super::Notation;
use Notation::*;

pub fn oracular_pretty_print(notation: &Notation, width: usize) -> Vec<(usize, String)> {
    let prefix = vec![(0, "".to_string())];
    let options = pp(0, prefix, &notation);
    let fallback = options.last().unwrap().clone();
    for option in options {
        if fits(width, &option) {
            return option;
        }
    }
    fallback
}

// INVARIANT: prefix is never empty. It should at the very least contain 1 empty string.
fn pp(
    indent: usize,
    mut lines: Vec<(usize, String)>,
    notation: &Notation,
) -> Vec<Vec<(usize, String)>> {
    match notation {
        Literal(text) => {
            lines.last_mut().unwrap().1.push_str(text);
            vec![lines]
        }
        Newline => {
            lines.push((indent, "".to_string()));
            vec![lines]
        }
        Indent(i, notation) => pp(indent + i, lines, notation),
        Flat(notation) => {
            let len = lines.len();
            pp(indent, lines, notation)
                .into_iter()
                .filter(|ls| ls.len() == len)
                .collect()
        }
        Concat(left, right) => {
            let mut options = vec![];
            for left_option in pp(indent, lines.clone(), left) {
                for option in pp(indent, left_option.clone(), right) {
                    options.push(option);
                }
            }
            options
        }
        Align(note) => {
            let (i, s) = lines.last().unwrap();
            let indent = i + s.chars().count();
            pp(indent, lines, note)
        }
        Choice(left, right) => {
            let mut options = pp(indent, lines.clone(), left);
            options.append(&mut pp(indent, lines, right));
            options
        }
    }
}

fn fits(width: usize, lines: &[(usize, String)]) -> bool {
    for (i, s) in lines {
        if i + s.chars().count() > width {
            return false;
        }
    }
    true
}
