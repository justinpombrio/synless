use crate::NotationSet;
use language::{Arity, Construct, Language};
use pretty::{child, literal, no_wrap, repeat, text, Notation, Repeat, Style};

pub fn make_json_lang() -> (Language, NotationSet) {
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
    let constructs = vec![
        Construct::new("string", "Value", Arity::Text, Some('s')),
        Construct::new("number", "Value", Arity::Text, Some('n')),
        Construct::new("true", "Value", Arity::Fixed(Vec::new()), Some('t')),
        Construct::new("false", "Value", Arity::Fixed(Vec::new()), Some('f')),
        Construct::new("null", "Value", Arity::Fixed(Vec::new()), Some('x')),
        Construct::new(
            "list",
            "Value",
            Arity::Flexible("Value".to_string()),
            Some('l'),
        ),
        Construct::new(
            "dict",
            "Value",
            Arity::Flexible("Entry".to_string()),
            Some('d'),
        ),
        Construct::new("key", "Key", Arity::Text, Some('k')),
        Construct::new(
            "entry",
            "Entry",
            Arity::Fixed(vec!["Key".to_string(), "Value".to_string()]),
            Some('e'),
        ),
    ];
    // TODO: some of this boilerplate should get abstracted out
    let mut lang = Language::new("json");
    for construct in constructs {
        lang.add(construct);
    }
    let note_set = NotationSet::new(&lang, notations);
    (lang, note_set)
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
