use super::measure::MeasuredNotation;

use MeasuredNotation::*;

pub fn pretty_print(notation: &MeasuredNotation, width: usize) -> Vec<String> {
    let prefix = vec!["".to_string()];
    pp(width, 0, prefix, 0, notation)
}

// INVARIANT: prefix is never empty. It should at the very least contain 1 empty string.
fn pp(
    width: usize,
    indent: usize,
    mut prefix: Vec<String>,
    suffix_len: usize,
    notation: &MeasuredNotation,
) -> Vec<String> {
    match notation {
        Literal(text) => {
            let line = format!("{}{}", prefix.pop().unwrap(), text);
            let mut answer = prefix;
            answer.push(line);
            answer
        }
        Newline => {
            // TODO better way to print spaces?
            let line = format!("{:indent$}", "", indent = indent);
            let mut answer = prefix;
            answer.push(line);
            answer
        }
        Indent(i, notation) => pp(width, indent + i, prefix, suffix_len, notation),
        Flat(notation) => {
            let text = pp_flat(notation);
            let line = format!("{}{}", prefix.pop().unwrap(), text);
            let mut answer = prefix;
            answer.push(line);
            answer
        }
        Concat(left, right, right_req) => {
            let single = right_req.single_line.unwrap_or(0) + suffix_len;
            let first = right_req.multi_line.map(|ml| ml.0).unwrap_or(0);
            let prefix = pp(width, indent, prefix, single.min(first), left);
            pp(width, indent, prefix, suffix_len, right)
        }
        Nest(left, right) => {
            let text = pp_flat(left);
            let indent = indent + text.chars().count();
            prefix.last_mut().unwrap().push_str(&text);
            pp(width, indent, prefix, suffix_len, right)
        }
        Choice((left, left_req), (right, _)) => {
            let prefix_len = prefix.last().unwrap().chars().count() as isize;
            let single_line_len = (width as isize) - prefix_len - (suffix_len as isize);
            let first_line_len = (width as isize) - prefix_len;
            let last_line_len = (width as isize) - (suffix_len as isize);

            let left_fits = left_req.fits_single_line(single_line_len)
                || left_req.fits_multi_line(first_line_len, last_line_len);
            if left_fits {
                pp(width, indent, prefix, suffix_len, left)
            } else {
                pp(width, indent, prefix, suffix_len, right)
            }
        }
    }
}

fn pp_flat(notation: &MeasuredNotation) -> String {
    match notation {
        Literal(text) => text.to_string(),
        Newline => panic!("pp_flat found a newline!"),
        Indent(_, notation) => pp_flat(notation),
        Flat(notation) => pp_flat(notation),
        Concat(left, right, _) => format!("{}{}", pp_flat(left), pp_flat(right)),
        Nest(left, right) => format!("{}{}", pp_flat(left), pp_flat(right)),
        Choice((left, left_req), (right, _)) => {
            if left_req.has_single_line() {
                pp_flat(left)
            } else {
                pp_flat(right)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::notation::Notation;
    use super::*;

    fn flat(notation: Notation) -> Notation {
        Notation::Flat(Box::new(notation))
    }

    fn lit(s: &str) -> Notation {
        Notation::literal(s)
    }

    fn nest(left: Notation, right: Notation) -> Notation {
        Notation::Nest(Box::new(left), Box::new(right))
    }

    fn line() -> Notation {
        Notation::Newline
    }

    fn hello() -> Notation {
        lit("Hello")
    }

    fn goodbye() -> Notation {
        lit("Good") + Notation::Newline + lit("Bye")
    }

    fn list_one(element: Notation) -> Notation {
        let option1 = lit("[") + element.clone() + lit("]");
        let option2 = nest(lit("["), line() + element) + line() + lit("]");
        option1 | option2
    }

    fn list_tight(elements: Vec<Notation>) -> Notation {
        let empty = lit("[]");
        let lone = |elem| lit("[") + elem + lit("]");
        let first = |first: Notation| first;
        let middle = |note: Notation| {
            let single = lit(", ") + flat(note.clone());
            let multi = lit(",") + line() + note;
            single | multi
        };
        let surround = |accum: Notation| {
            let single = flat(lit("[") + accum.clone() + lit("]"));
            let multi = nest(lit("["), line() + accum) + line() + lit("]");
            single | multi
        };
        Notation::repeat(elements, empty, lone, first, middle, middle, surround)
    }

    fn assert_pp(notation: Notation, width: usize, expected_lines: &[&str]) {
        let notation = notation.validate().unwrap();
        let notation = notation.measure();
        let lines = pretty_print(&notation, width);
        assert_eq!(lines, expected_lines);
    }

    #[test]
    fn test_pp_hello() {
        let n = Notation::indent(
            4,
            Notation::concat(
                Notation::concat(lit("Hello"), Notation::Newline),
                lit("world!"),
            ),
        );
        assert_pp(n, 80, &["Hello", "    world!"])
    }

    #[test]
    fn test_pp_choice() {
        let n = (hello() | goodbye()) + lit(" world");
        assert_pp(n, 80, &["Hello world"]);

        let n = (goodbye() | hello()) + lit(" world");
        assert_pp(n, 80, &["Good", "Bye world"]);

        let n = (goodbye() | goodbye()) + lit(" world");
        assert_pp(n, 80, &["Good", "Bye world"]);

        let n = (flat(goodbye()) | hello()) + lit(" world");
        assert_pp(n, 80, &["Hello world"]);

        let n = (hello() | goodbye()) + lit(" world");
        assert_pp(n, 3, &["Good", "Bye world"]);
    }

    #[test]
    fn test_pp_list_one() {
        let n = list_one(hello());
        assert_pp(n, 80, &["[Hello]"]);

        let n = list_one(goodbye());
        assert_pp(n, 80, &["[Good", "Bye]"]);
        // TODO test nest case
    }

    #[test]
    fn test_pp_list() {
        let n = list_tight(vec![]);
        assert_pp(n, 80, &["[]"]);

        let n = list_tight(vec![hello()]);
        assert_pp(n, 80, &["[Hello]"]);

        let n = list_tight(vec![hello(), hello()]);
        assert_pp(n, 80, &["[Hello, Hello]"]);

        let n = list_tight(vec![hello(), hello()]);
        assert_pp(n, 10, &["[", " Hello,", " Hello", "]"]);

        let n = list_tight(vec![goodbye()]);
        assert_pp(n, 80, &["[Good", "Bye]"]);

        let n = list_tight(vec![hello(), hello(), hello(), hello()]);
        assert_pp(n, 15, &["[", " Hello, Hello,", " Hello, Hello", "]"]);

        let n = list_tight(vec![goodbye(), hello(), hello()]);
        assert_pp(n, 80, &["[", " Good", " Bye, Hello, Hello", "]"]);
    }

    #[test]
    fn test_pp_simple_choice() {
        let ab = lit("ab") | (lit("a") + line() + lit("b"));
        let cd = lit("cd") | (lit("c") + line() + lit("d"));
        let ef = lit("ef") | (lit("e") + line() + lit("f"));
        let abcd = ab.clone() + cd.clone();
        assert_pp(abcd.clone(), 5, &["abcd"]);
        assert_pp(abcd.clone(), 4, &["abcd"]);
        assert_pp(abcd.clone(), 3, &["abc", "d"]);
        assert_pp(abcd, 2, &["a", "bc", "d"]);

        let abcdef = ab + cd + ef;
        assert_pp(abcdef.clone(), 7, &["abcdef"]);
        assert_pp(abcdef.clone(), 6, &["abcdef"]);
        assert_pp(abcdef.clone(), 5, &["abcde", "f"]);
        assert_pp(abcdef.clone(), 4, &["abc", "def"]);
        assert_pp(abcdef.clone(), 3, &["abc", "def"]);
        assert_pp(abcdef, 2, &["a", "bc", "de", "f"]);
    }
}
