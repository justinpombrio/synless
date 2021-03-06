use crate::NotationSet;
use language::{Arity, Construct, Language};
use pretty::notation_constructors::{child, literal, repeat, text};
use pretty::{Color, Notation, RepeatInner, Style};

pub fn make_message_lang() -> (Language, NotationSet) {
    let notations = vec![
        ("message".into(), text(Style::color(Color::Base08))),
        ("list".into(), list()),
    ];
    let constructs = vec![
        Construct::new("message", "Message", Arity::Text, None),
        Construct::new("list", "List", Arity::Flexible("Message".into()), None),
    ];
    // TODO: some of this boilerplate should get abstracted out
    let mut lang = Language::new("message".into());
    for construct in constructs {
        lang.add(construct);
    }
    let note_set = NotationSet::new(&lang, notations);
    (lang, note_set)
}

/// Put all messages on separate lines
fn list() -> Notation {
    repeat(RepeatInner {
        empty: literal("", Style::plain()),
        lone: child(0),
        join: Notation::Left ^ Notation::Right,
        surround: Notation::Surrounded,
    })
}
