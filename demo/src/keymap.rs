use std::collections::HashMap;
use termion::event::Key;

use crate::error::ShellError;
use crate::prog::{Prog, Value, Word};
use editor::AstRef;
use language::{Arity, Sort};

/// Rules for when a particular item should be included in a keymap
#[derive(Clone, Debug)]
pub enum KmapFilter {
    Always,
    Sort(Sort),
    ParentArity(Vec<ArityType>),
    SelfArity(Vec<ArityType>),
}

/// Like `Arity`, but without any data in the variants.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum ArityType {
    Text,
    Fixed,
    Flexible,
    Mixed,
}

pub struct TreeKmapFactory<'l>(pub Vec<(Key, KmapFilter, Prog<'l>)>);

#[derive(Clone)]
pub enum Kmap<'l> {
    Tree(HashMap<Key, Prog<'l>>),
    Text(HashMap<Key, Prog<'l>>),
}

impl<'l> TreeKmapFactory<'l> {
    pub fn filter<'a>(&'a self, ast: AstRef<'a, 'l>, required_sort: &Sort) -> Kmap<'l> {
        Kmap::Tree(
            self.0
                .iter()
                .filter_map(|(key, filter, prog)| match filter {
                    KmapFilter::Always => Some((key, prog)),
                    KmapFilter::Sort(sort) => {
                        if required_sort.accepts(sort) {
                            Some((key, prog))
                        } else {
                            None
                        }
                    }
                    KmapFilter::ParentArity(arity_types) => {
                        let (parent, _) = ast.parent()?;
                        for arity_type in arity_types {
                            if arity_type.is_type_of(parent.arity()) {
                                return Some((key, prog));
                            }
                        }
                        None
                    }
                    KmapFilter::SelfArity(arity_types) => {
                        for arity_type in arity_types {
                            if arity_type.is_type_of(ast.arity()) {
                                return Some((key, prog));
                            }
                        }
                        None
                    }
                })
                .map(|(key, prog)| (key.to_owned(), prog.to_owned()))
                .collect(),
        )
    }
}

impl<'l> Kmap<'l> {
    pub fn lookup(&self, key: Key) -> Result<Prog<'l>, ShellError> {
        match self {
            Kmap::Tree(map) => map.get(&key).cloned().ok_or(ShellError::UnknownKey(key)),
            Kmap::Text(map) => {
                if let Some(binding) = map.get(&key) {
                    Ok(binding.to_owned())
                } else if let Key::Char(c) = key {
                    Ok(Prog::named(
                        c,
                        &[Word::Literal(Value::Char(c)), Word::InsertChar],
                    ))
                } else {
                    Err(ShellError::UnknownKey(key))
                }
            }
        }
    }

    pub fn hints(&self) -> Vec<(String, String)> {
        match self {
            Kmap::Tree(map) | Kmap::Text(map) => {
                let mut v: Vec<_> = map
                    .iter()
                    .map(|(key, prog)| (format_key(key), prog.name().unwrap_or("...").to_owned()))
                    .collect();
                v.sort_unstable();
                v
            }
        }
    }
}

impl ArityType {
    fn is_type_of(self, arity: Arity) -> bool {
        match (self, arity) {
            (ArityType::Flexible, Arity::Flexible(..)) => true,
            (ArityType::Fixed, Arity::Fixed(..)) => true,
            (ArityType::Text, Arity::Text) => true,
            (ArityType::Mixed, Arity::Mixed(..)) => true,
            _ => false,
        }
    }
}

pub fn format_key(key: &Key) -> String {
    match key {
        Key::Backspace => "Bksp".to_string(),
        Key::Left => "←".to_string(),
        Key::Right => "→".to_string(),
        Key::Up => "↑".to_string(),
        Key::Down => "↓".to_string(),
        Key::Home => "Home".to_string(),
        Key::End => "End".to_string(),
        Key::PageUp => "PgUp".to_string(),
        Key::PageDown => "PgDn".to_string(),
        Key::Delete => "Del".to_string(),
        Key::Insert => "Ins".to_string(),
        Key::F(num) => format!("F{}", num),
        Key::Char(' ') => "Spc".to_string(),
        Key::Char(c) => c.to_string(),
        Key::Alt(' ') => "A-Spc".to_string(),
        Key::Alt(c) => format!("A-{}", c),
        Key::Ctrl(' ') => "C-Spc".to_string(),
        Key::Ctrl(c) => format!("C-{}", c),
        Key::Null => "Null".to_string(),
        Key::Esc => "Esc".to_string(),
        _ => "(unknown)".to_string(),
    }
}
