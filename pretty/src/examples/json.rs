use std::collections::HashMap;

use crate::{Notation, Repeat, Style, Color, text, literal, child, repeat, no_wrap};

// TODO: links from notation constructor functions to enum variants.
// TODO: NoWrap - say that it's recursive.


fn brackets(open: &str, delim: &str, close: &str) -> Notation {
    let style = Style::color(Color::Base08);

    let open = || literal(open, style);
    let close = || literal(close, style);
    let delim_space = || literal(&format!("{} ", delim), style);
    let delim = || literal(delim, style);

    
    repeat(Repeat { // Newline between elements
        empty:    open() + close(),
        lone:     open() + child(0) + close(),
        join:     child(0) + delim() ^ child(1),
        surround: open() + child(0) + close()
    }) | repeat(Repeat { // All one one line
        empty:    open() + close(),
        lone:     open() + child(0) + close(),
        join:     child(0) + delim_space() + child(1),
        surround: no_wrap(open() + child(0) + close())
    })
}

fn object_member() -> Notation {
    let style = Style::color(Color::Base08);
    child(0) + literal(": ", style) + child(1)
        | child(0) + literal(":", style) ^ child(1)
}

fn string() -> Notation {
    let style = Style::color(Color::Base0B);
    literal("\"", style) + text(style) + literal("\"", style)
}

fn number() -> Notation {
    let style = Style::color(Color::Base0D);
    text(style)
}

fn constant(c: &str) -> Notation {
    let style = Style::color(Color::Base09);
    literal(c, style)
}

pub fn json_notation() -> HashMap<String, Notation> {
    let mut map: HashMap<String, Notation> = HashMap::new();
    map.insert("object".to_string(), brackets("{", ",", "}"));
    map.insert("member".to_string(), object_member());
    map.insert("array".to_string(),  brackets("[", ",", "]"));
    map.insert("string".to_string(), string());
    map.insert("number".to_string(), number());
    map.insert("true".to_string(), constant("true"));
    map.insert("false".to_string(), constant("false"));
    map.insert("null".to_string(), constant("null"));
    map
}


#[cfg(test)]
mod json_tests {
    use std::collections::HashMap;
    use lazy_static::lazy_static;

    use super::json_notation;
    use crate::pretty::testing::TestTree;
    use crate::Notation;
    
    lazy_static! {
        pub static ref notations: HashMap<String, Notation> = json_notation();
    }

    fn leaf(construct: &str, contents: &str) -> TestTree {
        let note = notations.get(construct).unwrap().clone();
        TestTree::new_leaf(note, contents)
    }

    fn branch(construct: &str, children: Vec<TestTree>) -> TestTree {
        let note = notations.get(construct).unwrap().clone();
        TestTree::new_branch(note, children)
    }

    #[test]
    fn test_json_1() {
        let json = branch("array", vec!(
            leaf("number", "317"),
            leaf("string", "json says \"hello, world\""),
            branch("array", vec!(
                branch("true", vec!()),
                branch("false", vec!()))),
            branch("null", vec!())));
        assert_eq!(json.write(80),
                   "[317, \"json says \"hello, world\"\", [true, false], null]");
        assert_eq!(json.write(54),
                   "[317, \"json says \"hello, world\"\", [true, false], null]");
        assert_eq!(json.write(53),
                   "[317,
 \"json says \"hello, world\"\",
 [true, false],
 null]");
    }
}
