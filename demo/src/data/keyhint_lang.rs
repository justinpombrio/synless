use crate::NotationSet;
use language::{Arity, Construct, Language};
use pretty::notation_constructors::{child, literal, no_wrap, repeat, text};
use pretty::{Color, Notation, RepeatInner, Style};

pub fn make_keyhint_lang() -> (Language, NotationSet) {
    let notations = vec![
        ("key".into(), text(Style::color(Color::Base0D))),
        ("prog".into(), text(Style::color(Color::Base04))),
        ("binding".into(), binding()),
        ("keymap".into(), keymap()),
    ];
    let constructs = vec![
        Construct::new("key", "Key", Arity::Text, Some('k')),
        Construct::new("prog", "Value", Arity::Text, Some('p')),
        Construct::new(
            "keymap",
            "Keymap",
            Arity::Flexible("Binding".into()),
            Some('d'),
        ),
        Construct::new(
            "binding",
            "Binding",
            Arity::Fixed(vec!["Key".into(), "Value".into()]),
            Some('e'),
        ),
    ];
    // TODO: some of this boilerplate should get abstracted out
    let mut lang = Language::new("keyhint".into());
    for construct in constructs {
        lang.add(construct);
    }
    let note_set = NotationSet::new(&lang, notations);
    (lang, note_set)
}

/// Try putting the key and value on the same line.
/// If they don't fit, wrap after the colon, and indent the value.
fn binding() -> Notation {
    no_wrap(child(0) + punct(":") + child(1)) | ((child(0) + punct(":")) ^ (indent() + child(1)))
}

/// Wrap entries tightly.
fn keymap() -> Notation {
    repeat(RepeatInner {
        empty: punct("(empty keyhints)"),
        lone: child(0),
        join: (Notation::Left + punct(", ") + no_wrap(Notation::Right))
            | (Notation::Left + punct(",")) ^ no_wrap(Notation::Right),
        surround: Notation::Surrounded,
    })
}

fn punct(text: &str) -> Notation {
    literal(text, Style::color(Color::Base03))
}

fn indent() -> Notation {
    literal("  ", Style::plain())
}
