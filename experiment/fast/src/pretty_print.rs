use super::measure::{MeasuredNotation, Requirement};
use MeasuredNotation::*;

pub fn pretty_print(notation: &MeasuredNotation, width: usize) -> Vec<String> {
    let prefix = vec!["".to_string()];
    let suffix_req = Requirement::new_single_line(0);
    pp(width, 0, prefix, suffix_req, notation)
}

// INVARIANT: prefix is never empty. It should at the very least contain 1 empty string.
fn pp(
    width: usize,
    indent: usize,
    mut prefix: Vec<String>,
    suffix_req: Requirement,
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
            // TODO better way to print spaces? E.g. return Vec<(indent, line)>.
            let line = format!("{:indent$}", "", indent = indent);
            let mut answer = prefix;
            answer.push(line);
            answer
        }
        Indent(i, notation) => pp(width, indent + i, prefix, suffix_req, notation),
        Flat(notation) => {
            let text = pp_flat(notation);
            let mut answer = prefix;
            let line = format!("{}{}", answer.pop().unwrap(), text);
            answer.push(line);
            answer
        }
        Concat(left, right, right_req) => {
            let new_suffix_req = right_req.concat(suffix_req);
            let prefix = pp(width, indent, prefix, new_suffix_req, left);
            pp(width, indent, prefix, suffix_req, right)
        }
        Align(note) => {
            let indent = prefix.last().unwrap().chars().count();
            pp(width, indent, prefix, suffix_req, note)
        }
        Choice((left, left_req), (right, _)) => {
            let prefix_len = prefix.last().unwrap().chars().count();
            let req = Requirement::new_single_line(prefix_len)
                .concat(*left_req)
                .concat(suffix_req);
            if req.fits(width) {
                pp(width, indent, prefix, suffix_req, left)
            } else {
                pp(width, indent, prefix, suffix_req, right)
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
        Align(notation) => pp_flat(notation),
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

    fn indent(i: usize, notation: Notation) -> Notation {
        Notation::Indent(i, Box::new(notation))
    }

    fn align(notation: Notation) -> Notation {
        Notation::Align(Box::new(notation))
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
        let option2 = lit("[") + align(line() + element) + line() + lit("]");
        option1 | option2
    }

    fn list_tight(elements: Vec<Notation>) -> Notation {
        let empty = lit("[]");
        let lone = |elem| lit("[") + elem + lit("]");
        let first = |first: Notation| first;
        let middle = |note: Notation| {
            let single = lit(", ") + note.clone();
            let multi = lit(",") + line() + note;
            single | multi
        };
        let surround = |accum: Notation| {
            let single = flat(lit("[") + accum.clone() + lit("]"));
            let multi = align(lit("[") + indent(1, line() + accum) + line() + lit("]"));
            single | multi
        };
        Notation::repeat(elements, empty, lone, first, middle, middle, surround)
    }

    fn json_string(s: &str) -> Notation {
        // Using single quote instead of double quote to avoid inconvenient
        // escaping
        lit("'") + lit(s) + lit("'")
    }

    fn json_key(s: &str) -> Notation {
        // Using single quote instead of double quote to avoid inconvenient
        // escaping
        lit("'") + lit(s) + lit("'")
    }

    fn json_entry(key: &str, value: Notation) -> Notation {
        // json_key(key) + lit(":") + (lit(" ") | line()) + value
        json_key(key) + lit(": ") + value
    }

    fn json_dict(entries: Vec<Notation>) -> Notation {
        let tab = 4;
        let empty = lit("{}");
        let lone = |elem: Notation| {
            (lit("{") + flat(elem.clone()) + lit("}"))
                | (lit("{") + indent(tab, line() + elem) + line() + lit("}"))
        };
        let first = |first: Notation| first;
        let middle = |note: Notation| lit(",") + line() + note;
        let surround = |accum: Notation| {
            let single = flat(lit("{") + accum.clone() + lit("}"));
            let multi = lit("{") + indent(tab, line() + accum) + line() + lit("}");
            single | multi
        };
        Notation::repeat(entries, empty, lone, first, middle, middle, surround)
    }

    fn assert_pp(notation: Notation, width: usize, expected_lines: &[&str]) {
        let notation = notation.validate().unwrap();
        let notation = notation.measure();
        let lines = pretty_print(&notation, width);
        if lines != expected_lines {
            eprintln!(
                "EXPECTED:\n{}\n\nACTUAL:\n{}\n",
                expected_lines.join("\n"),
                lines.join("\n"),
            );
            assert_eq!(lines, expected_lines);
        }
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

        let n = list_tight(vec![goodbye(), hello(), hello(), goodbye()]);
        assert_pp(
            n,
            80,
            &["[", " Good", " Bye, Hello, Hello, Good", " Bye", "]"],
        );
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

    #[test]
    fn test_pp_dict() {
        let e1 = json_entry("Name", json_string("Alice"));
        let e2 = json_entry("Age", lit("42"));
        let e3 = json_entry(
            "Favorites",
            list_tight(vec![
                json_string("chocolate"),
                json_string("lemon"),
                json_string("almond"),
            ]),
        );

        let n = json_dict(vec![e1.clone()]);
        assert_pp(n, 80, &["{'Name': 'Alice'}"]);

        let n = json_dict(vec![e1.clone(), e2.clone()]);
        assert_pp(
            n,
            80,
            &[
                // force rustfmt
                "{",
                "    'Name': 'Alice',",
                "    'Age': 42",
                "}",
            ],
        );

        let n = json_dict(vec![e1, e2, e3]);
        assert_pp(
            n,
            38,
            &[
                "{",
                "    'Name': 'Alice',",
                "    'Age': 42,",
                "    'Favorites': [",
                "                  'chocolate',",
                "                  'lemon', 'almond'",
                "                 ]",
                "}",
            ],
        );
    }

    #[test]
    fn test_pp_align() {
        let n = lit("four") + list_tight(vec![hello(), hello()]);
        assert_pp(
            n,
            10,
            &[
                // make rustfmt split lines
                "four[",
                "     Hello,",
                "     Hello",
                "    ]",
            ],
        );
    }

}
