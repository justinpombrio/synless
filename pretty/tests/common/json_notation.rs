use pretty::{child, literal, no_wrap, repeat, text, Notation, Repeat, Style};
use std::collections::HashMap;

pub fn make_json_notation() -> HashMap<String, Notation> {
    let mut map = HashMap::new();
    map.insert("string".into(), json_string());
    map.insert("number".into(), json_number());
    map.insert("true".into(), json_boolean(true));
    map.insert("false".into(), json_boolean(false));
    map.insert("null".into(), json_null());
    map.insert("list".into(), json_list());
    map.insert("dict_entry".into(), json_dict_entry());
    map.insert("dict".into(), json_dict());
    map
}

fn json_string() -> Notation {
    let style = Style::plain();
    literal("\"", style) + text(style) + literal("\"", style)
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
        join: (child(0) + punct(", ") + no_wrap(child(1)))
            | (child(0) + punct(",")) ^ no_wrap(child(1)),
        surround: punct("[") + child(0) + punct("]"),
    }) | repeat(Repeat {
        empty,
        lone,
        join: (child(0) + punct(",")) ^ child(1),
        surround: punct("[") + child(0) + punct("]"),
    })
}

/// Try putting the key and value on the same line.
/// If they don't fit, wrap after the colon, and indent the value.
fn json_dict_entry() -> Notation {
    no_wrap(child(0) + punct(": ") + child(1)) | ((child(0) + punct(":")) ^ (indent() + child(1)))
}

/// Put all entries on separate lines.
/// If there is more than one entry, put the opening and closing delimiters on separate lines too.
fn json_dict() -> Notation {
    repeat(Repeat {
        empty: punct("{}"),
        lone: punct("{") + child(0) + punct("}"),
        join: (child(0) + punct(",")) ^ child(1),
        surround: punct("{") ^ (indent() + child(0)) ^ punct("}"),
    })
}

fn punct(text: &str) -> Notation {
    literal(text, Style::plain())
}

fn indent() -> Notation {
    literal("  ", Style::plain())
}
