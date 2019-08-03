use crate::NotationSet;
use language::{Arity, Construct, Language};
use pretty::{child, literal, no_wrap, repeat, text, Notation, Repeat, Style};

pub fn make_keymap_lang() -> (Language, NotationSet) {
    let notations = vec![
        ("key".into(), key()),
        ("prog".into(), prog()),
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

fn key() -> Notation {
    let style = Style::plain();
    text(style)
}

fn prog() -> Notation {
    let style = Style::plain();
    text(style)
}

/// Try putting the key and value on the same line.
/// If they don't fit, wrap after the colon, and indent the value.
fn entry() -> Notation {
    no_wrap(child(0) + punct(": ") + child(1)) | (child(0) + punct(":") ^ indent() + child(1))
}

/// Put all entries on separate lines.
/// If there is more than one entry, put the opening and closing delimiters on separate lines too.
fn dict() -> Notation {
    repeat(Repeat {
        empty: punct("{}"),
        lone: punct("{") + child(0) + punct("}"),
        join: child(0) + punct(",") ^ child(1),
        surround: punct("{") ^ indent() + child(0) ^ punct("}"),
    })
}

fn punct(text: &str) -> Notation {
    literal(text, Style::plain())
}

fn indent() -> Notation {
    literal("  ", Style::plain())
}
