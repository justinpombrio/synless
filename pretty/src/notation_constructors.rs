use crate::{Notation, RepeatInner, Style};

pub fn child(index: usize) -> Notation {
    Notation::Child(index)
}

pub fn no_wrap(inner: Notation) -> Notation {
    Notation::NoWrap(Box::new(inner))
}

pub fn text(style: Style) -> Notation {
    Notation::Text(style)
}

pub fn literal(string: &str, style: Style) -> Notation {
    Notation::Literal(string.to_owned(), style)
}

pub fn repeat(inner: RepeatInner) -> Notation {
    Notation::Repeat(Box::new(inner))
}
