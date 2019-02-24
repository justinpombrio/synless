// TODO: replace this random grammar with a more useful Json notation.
use pretty::{
    child, empty, if_empty_text, literal, repeat, star, text, Color, Notation, Repeat, Style,
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
    let note = repeat(Repeat {
        empty: empty(),
        lone: star(),
        first: star() + punct(", "),
        middle: star() + punct(", "),
        last: star(),
    }) | repeat(Repeat {
        empty: empty(),
        lone: star(),
        first: star() + punct(",") ^ empty(),
        middle: star() + punct(",") ^ empty(),
        last: star(),
    });
    map.insert("args".to_string(), note);

    let note = repeat(Repeat {
        empty: punct("[]"),
        lone: punct("[") + star() + punct("]"),
        first: punct("[") + star() + punct(", "),
        middle: star() + punct(", "),
        last: star() + punct("]"),
    }) | repeat(Repeat {
        empty: punct("[]"),
        lone: punct("[") + star() + punct("]"),
        first: star() + punct(",") ^ empty(),
        middle: star() + punct(",") ^ empty(),
        last: star() + punct("]"),
    }) | repeat(Repeat {
        empty: punct("[]"),
        lone: punct("[") + star() + punct("]"),
        first: punct("[") + (star() + punct(", ") | star() + punct(",") ^ empty()),
        middle: star() + punct(", ") | star() + punct(",") ^ empty(),
        last: star() + punct("]"),
    });
    map.insert("list".to_string(), note);

    let note =
        word("func ") + child(0) + punct("(") + child(1) + punct(") { ") + child(2) + punct(" }")
            | word("func ") + child(0) + punct("(") + child(1) + punct(") {")
                ^ empty() + word("  ") + child(2)
                ^ empty() + punct("}")
            | word("func ") + child(0) + punct("(")
                ^ empty() + word("  ") + child(1) + punct(")")
                ^ empty() + punct("{")
                ^ empty() + word("  ") + child(2)
                ^ empty() + punct("}");
    map.insert("function".to_string(), note);

    let note = child(0) + punct(" + ") + child(1)
        | child(0) ^ punct("+ ") + child(1)
        | child(0) ^ punct("+") ^ child(1);
    map.insert("add".to_string(), note);

    let note = if_empty_text(txt() + punct("·"), txt());
    map.insert("id".to_string(), note);

    let note = punct("'") + txt() + punct("'");
    map.insert("string".to_string(), note);

    map
}
