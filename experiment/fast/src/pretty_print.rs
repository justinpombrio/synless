use super::notation::Notation;

use Notation::*;

pub fn pretty_print(notation: &Notation, width: usize) -> Vec<String> {
    let prefix = vec!["".to_string()];
    pp(width, 0, prefix, 0, notation)
}

// INVARIANT: prefix is never empty. It should at the very least contain 1 empty string.
fn pp(
    width: usize,
    indent: usize,
    mut prefix: Vec<String>,
    reserved: usize,
    notation: &Notation,
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
        Indent(i, notation) => pp(width, indent + i, prefix, reserved, notation),
        NoWrap(notation) => {
            let text = pp_nowrap(notation);
            let line = format!("{}{}", prefix.pop().unwrap(), text);
            let mut answer = prefix;
            answer.push(line);
            answer
        }
        Concat(left, right, right_req) => {
            let single = right_req.single_line.unwrap_or(0) + reserved;
            let first = right_req.multi_line.map(|ml| ml.0).unwrap_or(0);

            let prefix = pp(width, indent, prefix, single.min(first), left);
            pp(width, indent, prefix, reserved, right)
        }
        Nest(left, right) => {
            let text = pp_nowrap(left);
            let indent = indent + text.chars().count();
            prefix.last_mut().unwrap().push_str(&text);
            pp(width, indent, prefix, reserved, right)
        }
        Choice((left, left_req), (right, _right_req)) => {
            let prefix_len = prefix.last().unwrap().chars().count() as isize;
            let suffix_len = reserved as isize;
            let single_line_len = (width as isize) - prefix_len - suffix_len;
            let first_line_len = (width as isize) - prefix_len;
            let last_line_len = (width as isize) - suffix_len;

            let left_fits = left_req.fits_single_line(single_line_len)
                || left_req.fits_multi_line(first_line_len, last_line_len);
            if left_fits {
                pp(width, indent, prefix, reserved, left)
            } else {
                pp(width, indent, prefix, reserved, right)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn no_wrap(notation: Notation) -> Notation {
        Notation::NoWrap(Box::new(notation))
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
            let single = lit(", ") + no_wrap(note.clone());
            let multi = lit(",") + line() + note;
            single | multi
        };
        let surround = |accum: Notation| {
            let single = no_wrap(lit("[") + accum.clone() + lit("]"));
            let multi = nest(lit("["), line() + accum) + line() + lit("]");
            single | multi
        };
        Notation::repeat(elements, empty, lone, first, middle, middle, surround)
    }

    fn assert_pp(notation: &mut Notation, width: usize, expected_lines: &[&str]) {
        notation.finalize().unwrap();
        let lines = pretty_print(&notation, width);
        assert_eq!(lines, expected_lines);
    }

    #[test]
    fn test_pp_hello() {
        let mut n = Notation::indent(
            4,
            Notation::concat(
                Notation::concat(lit("Hello"), Notation::Newline),
                lit("world!"),
            ),
        );
        assert_pp(&mut n, 80, &["Hello", "    world!"])
    }

    #[test]
    fn test_pp_choice() {
        let mut n = (hello() | goodbye()) + lit(" world");
        assert_pp(&mut n, 80, &["Hello world"]);

        let mut n = (goodbye() | hello()) + lit(" world");
        assert_pp(&mut n, 80, &["Good", "Bye world"]);

        let mut n = (goodbye() | goodbye()) + lit(" world");
        assert_pp(&mut n, 80, &["Good", "Bye world"]);

        let mut n = (no_wrap(goodbye()) | hello()) + lit(" world");
        assert_pp(&mut n, 80, &["Hello world"]);

        let mut n = (hello() | goodbye()) + lit(" world");
        assert_pp(&mut n, 3, &["Good", "Bye world"]);
    }

    #[test]
    fn test_pp_list_one() {
        let mut n = list_one(hello());
        assert_pp(&mut n, 80, &["[Hello]"]);

        let mut n = list_one(goodbye());
        assert_pp(&mut n, 80, &["[Good", "Bye]"]);
        // TODO test nest case
    }

    #[test]
    fn test_pp_list() {
        let mut n = list_tight(vec![]);
        assert_pp(&mut n, 80, &["[]"]);

        let mut n = list_tight(vec![hello()]);
        assert_pp(&mut n, 80, &["[Hello]"]);

        let mut n = list_tight(vec![hello(), hello()]);
        assert_pp(&mut n, 80, &["[Hello, Hello]"]);

        let mut n = list_tight(vec![hello(), hello()]);
        assert_pp(&mut n, 10, &["[", " Hello,", " Hello", "]"]);

        let mut n = list_tight(vec![goodbye()]);
        assert_pp(&mut n, 80, &["[Good", "Bye]"]);

        let mut n = list_tight(vec![hello(), hello(), hello(), hello()]);
        assert_pp(&mut n, 15, &["[", " Hello, Hello,", " Hello, Hello", "]"]);

        let mut n = list_tight(vec![goodbye(), hello(), hello()]);
        assert_pp(&mut n, 80, &["[", " Good", " Bye, Hello, Hello", "]"]);
    }

    #[test]
    fn test_pp_simple_choice() {
        let ab = lit("ab") | (lit("a") + line() + lit("b"));
        let cd = lit("cd") | (lit("c") + line() + lit("d"));
        let ef = lit("ef") | (lit("e") + line() + lit("f"));
        let mut abcd = ab.clone() + cd.clone();
        assert_pp(&mut abcd, 5, &["abcd"]);
        assert_pp(&mut abcd, 4, &["abcd"]);
        assert_pp(&mut abcd, 3, &["abc", "d"]);
        assert_pp(&mut abcd, 2, &["a", "bc", "d"]);

        let mut abcdef = ab + cd + ef;
        assert_pp(&mut abcdef, 7, &["abcdef"]);
        assert_pp(&mut abcdef, 6, &["abcdef"]);
        assert_pp(&mut abcdef, 5, &["abcde", "f"]);
        assert_pp(&mut abcdef, 4, &["abc", "def"]);
        assert_pp(&mut abcdef, 3, &["abc", "def"]);
        assert_pp(&mut abcdef, 2, &["a", "bc", "de", "f"]);
    }
}
