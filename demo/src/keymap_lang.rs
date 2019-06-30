use crate::NotationSet;
use language::{Arity, Construct, Language};
use pretty::{child, literal, no_wrap, repeat, text, Color, Notation, Repeat, Style};

pub fn make_keymap_lang() -> (Language, NotationSet) {
    let notations = vec![
        ("key".into(), text(Style::color(Color::Base0D))),
        ("prog".into(), text(Style::color(Color::Base04))),
        ("entry".into(), entry()),
        ("dict".into(), dict()),
    ];
    let constructs = vec![
        Construct::new("key", "Key", Arity::Text, Some('k')),
        Construct::new("prog", "Value", Arity::Text, Some('p')),
        Construct::new(
            "dict",
            "Dict",
            Arity::Flexible("Entry".to_string()),
            Some('d'),
        ),
        Construct::new(
            "entry",
            "Entry",
            Arity::Fixed(vec!["Key".to_string(), "Value".to_string()]),
            Some('e'),
        ),
    ];
    // TODO: some of this boilerplate should get abstracted out
    let mut lang = Language::new("keymap");
    for construct in constructs {
        lang.add(construct);
    }
    let note_set = NotationSet::new(&lang, notations);
    (lang, note_set)
}

/// Try putting the key and value on the same line.
/// If they don't fit, wrap after the colon, and indent the value.
fn entry() -> Notation {
    no_wrap(child(0) + punct(":") + child(1)) | (child(0) + punct(":") ^ indent() + child(1))
}

/// Wrap entries tightly.
fn dict() -> Notation {
    repeat(Repeat {
        empty: punct("(empty keymap)"),
        lone: child(0),
        join: child(0) + punct(", ") + no_wrap(child(1))
            | child(0) + punct(",") ^ no_wrap(child(1)),
        surround: child(0),
    })
}

fn punct(text: &str) -> Notation {
    literal(text, Style::color(Color::Base03))
}

fn indent() -> Notation {
    literal("  ", Style::plain())
}
