use editor::NotationSet;
use language::{Arity, Construct, Language, LanguageSet};
use pretty::{child, literal, no_wrap, repeat, text, Notation, Repeat, Style};
// use std::collections::HashMap;

pub fn make_json_lang() -> (LanguageSet, NotationSet) {
    let notations = vec![
        ("string".into(), json_string()),
        ("number".into(), json_number()),
        ("true".into(), json_boolean(true)),
        ("false".into(), json_boolean(false)),
        ("null".into(), json_null()),
        ("list".into(), json_list()),
        ("dict".into(), json_dict()),
        ("key".into(), json_key()),
        ("entry".into(), json_dict_entry()),
    ];
    let value = "ValueSort";
    let entry = "EntrySort";
    let key = "KeySort";
    // let dict = "DictSort";
    let constructs = vec![
        // TODO no key for root?
        Construct::new("string", value, Arity::Text, 's'),
        Construct::new("number", value, Arity::Text, 'n'),
        Construct::new("true", value, Arity::Fixed(Vec::new()), 't'),
        Construct::new("false", value, Arity::Fixed(Vec::new()), 'f'),
        Construct::new("null", value, Arity::Fixed(Vec::new()), 'x'),
        Construct::new("list", value, Arity::Flexible(value.into()), 'l'),
        Construct::new("dict", value, Arity::Flexible(entry.into()), 'd'),
        Construct::new("key", key, Arity::Text, 'k'),
        Construct::new(
            "entry",
            entry,
            Arity::Fixed(vec![key.into(), value.into()]),
            'e',
        ),
    ];
    let mut lang = Language::new("json");
    for construct in constructs {
        lang.add(construct);
    }
    let note_set = NotationSet::new(&lang, notations);
    let lang_set = LanguageSet::new();
    lang_set.insert(lang.name().to_owned(), lang);
    (lang_set, note_set)
}

fn json_string() -> Notation {
    let style = Style::plain();
    literal("\"", style) + text(style) + literal("\"", style)
}

fn json_key() -> Notation {
    let style = Style::plain();
    literal("'", style) + text(style) + literal("'", style)
}

fn json_number() -> Notation {
    let style = Style::plain();
    text(style)
}

fn json_boolean(value: bool) -> Notation {
    let style = Style::plain();
    let name = if value { "true" } else { "false" };
    literal(name, style)
}

fn json_null() -> Notation {
    let style = Style::plain();
    literal("null", style)
}

/// If there is any multiline element, all elements must go on separate lines.
/// Otherwise, they can be grouped together on the same lines.
/// Put the opening and closing delimiters on the same lines as the first and last elements, respectively.
fn json_list() -> Notation {
    let empty = punct("[]");
    let lone = punct("[") + child(0) + punct("]");
    repeat(Repeat {
        empty: empty.clone(),
        lone: lone.clone(),
        join: child(0) + punct(", ") + no_wrap(child(1))
            | child(0) + punct(",") ^ no_wrap(child(1)),
        surround: punct("[") + child(0) + punct("]"),
    }) | repeat(Repeat {
        empty,
        lone,
        join: child(0) + punct(",") ^ child(1),
        surround: punct("[") + child(0) + punct("]"),
    })
}

/// Try putting the key and value on the same line.
/// If they don't fit, wrap after the colon, and indent the value.
fn json_dict_entry() -> Notation {
    no_wrap(child(0) + punct(": ") + child(1)) | (child(0) + punct(":") ^ indent() + child(1))
}

/// Put all entries on separate lines.
/// If there is more than one entry, put the opening and closing delimiters on separate lines too.
fn json_dict() -> Notation {
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
