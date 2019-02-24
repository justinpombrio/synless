// TODO: replace this random grammar with a more useful Json notation.
use pretty::{
    child, empty, if_empty_text, literal, repeat, text, Color, Notation, Repeat, Style,
};
use std::collections::HashMap;


pub fn example_notation() -> HashMap<String, Notation> {
    fn punct(text: &str) -> Notation {
        literal(text, Style::color(Color::Base0A))
    }
    fn word(text: &str) -> Notation {
        literal(text, Style::color(Color::Base0D))
    }
    fn txt() -> Notation {
        text(Style::plain())
    }

    let mut map = HashMap::new();
    let note = repeat(Repeat{
        empty:    empty(),
        lone:     child(0),
        join:     child(0) + punct(", ") + child(1),
        surround: child(0)
    }) | repeat(Repeat{
        empty:    empty(),
        lone:     child(0),
        join:     child(0) + punct(",") ^ child(1),
        surround: child(0)
    });
    map.insert("args".to_string(), note);

    let note = repeat(Repeat{
        empty:    punct("[]"),
        lone:     punct("[") + child(0) + punct("]"),
        join:     child(0) + punct(", ") + child(1),
        surround: punct("[") + child(0) + punct("]")
    })| repeat(Repeat{
        empty:    punct("[]"),
        lone:     punct("[") + child(0) + punct("]"),
        join:     child(0) + punct(",") ^ child(1),
        surround: punct("[") + child(0) + punct("]")
    })| repeat(Repeat{
        empty:    punct("[]"),
        lone:     punct("[") + child(0) + punct("]"),
        join:     child(0) + punct(", ") + child(1)
                | child(0) + punct(",") ^ child(1),
        surround: punct("[") + child(0) + punct("]")
    });
    map.insert("list".to_string(), note);

    let note =
        word("func ") + child(0)
        + punct("(") + child(1) + punct(") { ") + child(2) + punct(" }")
        | word("func ") + child(0) + punct("(") + child(1) + punct(") {") ^ empty()
        + word("  ") + child(2) ^ empty()
        + punct("}")
        | word("func ") + child(0) + punct("(") ^ empty()
        + word("  ") + child(1) + punct(")") ^ empty()
        + punct("{") ^ empty()
        + word("  ") + child(2) ^ empty()
        + punct("}");
    map.insert("function".to_string(), note);

    let note =
        child(0) + punct(" + ") + child(1)
        | child(0) ^ punct("+ ") + child(1)
        | child(0) ^ punct("+") ^ child(1);
    map.insert("add".to_string(), note);

    let note = if_empty_text(txt() + punct("·"), txt());
    map.insert("id".to_string(), note);

    let note = punct("'") + txt() + punct("'");
    map.insert("string".to_string(), note);

    map
}
