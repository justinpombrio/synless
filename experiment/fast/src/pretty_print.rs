use super::notation::{ChoosyChild, Notation};

use Notation::*;

pub fn pretty_print(notation: &Notation, width: usize) -> Vec<String> {
    let prefix = vec!["".to_string()];
    let suffix = vec!["".to_string()];
    pp(width, 0, prefix, suffix, notation)
}

// INVARIANT: prefix & suffix are never empty.
fn pp(
    width: usize,
    indent: usize,
    mut prefix: Vec<String>,
    mut suffix: Vec<String>,
    notation: &Notation,
) -> Vec<String> {
    match notation {
        Literal(text) => {
            let middle_line = format!("{}{}{}", prefix.pop().unwrap(), text, suffix.pop().unwrap());
            let mut answer = prefix;
            answer.push(middle_line);
            answer.extend(suffix);
            answer
        }
        Newline => {
            let middle_line = format!("{:indent$}", suffix.pop().unwrap(), indent = indent);
            let mut answer = prefix;
            answer.push(middle_line);
            answer.extend(suffix);
            answer
        }
        Indent(i, notation) => pp(width, indent + i, prefix, suffix, notation),
        NoWrap(notation) => {
            let text = pp_nowrap(notation);
            let middle_line = format!("{}{}{}", prefix.pop().unwrap(), text, suffix.pop().unwrap());
            let mut answer = prefix;
            answer.push(middle_line);
            answer.extend(suffix);
            answer
        }
        Concat(_, _, ChoosyChild::Uninitialized) => {
            panic!("Concat's `choosy` field was never initialized");
        }
        Concat(left, right, ChoosyChild::Left) => {
            let suffix = pp(width, indent, vec!["".to_string()], suffix, right);
            pp(width, indent, prefix, suffix, left)
        }
        Concat(left, right, _) => {
            let prefix = pp(width, indent, prefix, vec!["".to_string()], left);
            pp(width, indent, prefix, suffix, right)
        }
        Nest(left, right) => {
            let text = pp_nowrap(left);
            let indent = indent + text.chars().count();
            prefix.last_mut().unwrap().push_str(&text);
            pp(width, indent, prefix, suffix, right)
        }
        Choice((left, left_req), (right, _right_req)) => {
            let prefix_len = prefix.last().unwrap().chars().count();
            let suffix_len = suffix.last().unwrap().chars().count();

            let single_line_len = width - prefix_len - suffix_len;
            let first_line_len = width - prefix_len;
            let last_line_len = width - suffix_len;

            let left_fits = left_req.fits_single_line(single_line_len)
                || left_req.fits_multi_line(first_line_len, last_line_len);
            if left_fits {
                pp(width, indent, prefix, suffix, left)
            } else {
                pp(width, indent, prefix, suffix, right)
            }
        }
    }
}

fn pp_nowrap(notation: &Notation) -> String {
    match notation {
        Literal(text) => text.to_string(),
        Newline => panic!("pp_nowrap found a newline!"),
        Indent(_, notation) => pp_nowrap(notation),
        NoWrap(notation) => pp_nowrap(notation),
        Concat(left, right, _) => format!("{}{}", pp_nowrap(left), pp_nowrap(right)),
        Nest(left, right) => format!("{}{}", pp_nowrap(left), pp_nowrap(right)),
        Choice((left, left_req), (right, right_req)) => {
            if left_req.has_single_line() {
                pp_nowrap(left)
            } else if right_req.has_single_line() {
                pp_nowrap(right)
            } else {
                panic!("pp_nowrap found a choice with no single line options!");
            }
        }
    }
}

#[test]
fn test_pp() {
    let mut n = Notation::indent(
        4,
        Notation::concat(
            Notation::concat(Notation::literal("Hello"), Notation::Newline),
            Notation::literal("world!"),
        ),
    );
    n.finalize().unwrap();
    assert_eq!(pretty_print(&n, 80), vec!["Hello", "    world!"]);
}
