//! Slow (exp time), but definitely correct, pretty printer.

use super::validate::ValidNotation;
use super::Notation;
use Notation::*;

pub fn oracular_pretty_print(notation: &ValidNotation, width: usize) -> Vec<(usize, String)> {
    let prefix = vec![(0, "".to_string())];
    let options = pp(0, prefix, &notation.0);
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
            let text = pp_flat(notation).expect("Oracle found Flat containing newline!");
            lines.last_mut().unwrap().1.push_str(&text);
            vec![lines]
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

fn pp_flat(notation: &Notation) -> Option<String> {
    match notation {
        Literal(text) => Some(text.to_string()),
        Newline => panic!("pp_flat found a newline!"),
        Indent(_, notation) => pp_flat(notation),
        Flat(notation) => pp_flat(notation),
        Concat(left, right) => Some(format!("{}{}", pp_flat(left)?, pp_flat(right)?)),
        Align(notation) => pp_flat(notation),
        Choice(left, right) => pp_flat(left).or_else(|| pp_flat(right)),
    }
}
