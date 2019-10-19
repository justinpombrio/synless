use std::collections::HashMap;
use std::iter;
use termion::event::Key;

use crate::error::Error;
use crate::prog::{Prog, Word};
use editor::AstRef;
use language::{Arity, Language, LanguageName, Sort};

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

pub struct KmapFactory<'l>(Vec<(Key, KmapFilter, Prog<'l>)>);

pub struct FilteredKmap<'l>(HashMap<Key, Prog<'l>>);

impl<'l> KmapFactory<'l> {
    pub fn filter<'a>(&'a self, ast: AstRef<'a, 'l>, required_sort: &Sort) -> FilteredKmap<'l> {
        FilteredKmap(
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

    pub fn make_node_map(lang: &Language) -> Self {
        KmapFactory(
            lang.keymap()
                .iter()
                .map(|(&ch, construct_name)| {
                    (
                        Key::Char(ch),
                        KmapFilter::Sort(lang.lookup_construct(construct_name).sort.clone()),
                        Prog::named(
                            construct_name,
                            &[
                                // Push the new node onto the stack, and then
                                // apply the quoted command (eg. InsertAfter)
                                // that was already on the top of the stack.
                                Word::LangConstruct(lang.name().into(), construct_name.to_owned()),
                                Word::NodeByName,
                                Word::Swap,
                                Word::Apply,
                                Word::PopMap,
                            ],
                        ),
                    )
                })
                .chain(iter::once((
                    Key::Esc,
                    KmapFilter::Always,
                    Prog::named(
                        "Cancel",
                        &[
                            Word::Pop, // get rid of the quoted command on the stack
                            Word::PopMap,
                        ],
                    ),
                )))
                .collect(),
        )
    }

    pub fn make_tree_map() -> Self {
        KmapFactory(vec![
            (Key::Char('d'), KmapFilter::Always, Prog::single(Word::Cut)),
            (Key::Char('y'), KmapFilter::Always, Prog::single(Word::Copy)),
            (
                Key::Char('P'),
                KmapFilter::Always,
                Prog::single(Word::PasteSwap),
            ),
            (
                Key::Char('p'),
                KmapFilter::Always,
                Prog::named("PasteReplace", &[Word::PasteSwap, Word::PopClipboard]),
            ),
            (
                Key::Char('a'),
                KmapFilter::Always,
                Prog::named("TypeA", &[Word::Char('a'), Word::InsertChar]),
            ),
            (Key::Char('u'), KmapFilter::Always, Prog::single(Word::Undo)),
            (Key::Ctrl('r'), KmapFilter::Always, Prog::single(Word::Redo)),
            (Key::Right, KmapFilter::Always, Prog::single(Word::Right)),
            (Key::Left, KmapFilter::Always, Prog::single(Word::Left)),
            (Key::Up, KmapFilter::Always, Prog::single(Word::Parent)),
            (
                Key::Backspace,
                KmapFilter::ParentArity(vec![ArityType::Flexible, ArityType::Mixed]),
                Prog::single(Word::Remove),
            ),
            (
                Key::Char('x'),
                KmapFilter::Always,
                Prog::single(Word::Clear),
            ),
            (
                Key::Down,
                KmapFilter::Always,
                Prog::named("Child", &[Word::Usize(0), Word::Child]),
            ),
            (
                Key::Char('i'),
                KmapFilter::ParentArity(vec![ArityType::Flexible, ArityType::Mixed]),
                Prog::single(Word::InsertHoleAfter),
            ),
            (
                Key::Char('I'),
                KmapFilter::ParentArity(vec![ArityType::Flexible, ArityType::Mixed]),
                Prog::single(Word::InsertHoleBefore),
            ),
            (
                Key::Char('o'),
                KmapFilter::SelfArity(vec![ArityType::Flexible, ArityType::Mixed]),
                Prog::single(Word::InsertHolePostpend),
            ),
            (
                Key::Char('O'),
                KmapFilter::SelfArity(vec![ArityType::Flexible, ArityType::Mixed]),
                Prog::single(Word::InsertHolePrepend),
            ),
            (
                Key::Char('r'),
                KmapFilter::Always,
                Prog::named(
                    "Replace",
                    &[
                        Word::Replace.quote(),
                        Word::MapName("node".into()),
                        Word::SelfSort,
                        Word::PushMap,
                    ],
                ),
            ),
            (
                Key::Char(' '),
                KmapFilter::ParentArity(vec![ArityType::Flexible, ArityType::Mixed]),
                Prog::named(
                    "SpeedBoolMode",
                    &[
                        Word::MapName("speed_bool".into()),
                        Word::AnySort,
                        Word::PushMap,
                    ],
                ),
            ),
        ])
    }

    pub fn make_speed_bool_map() -> Self {
        let lang: LanguageName = "json".into();
        KmapFactory(vec![
            (
                Key::Char('t'),
                KmapFilter::ParentArity(vec![ArityType::Flexible, ArityType::Mixed]),
                Prog::named(
                    "True",
                    &[
                        Word::LangConstruct(lang.clone(), "true".into()),
                        Word::InsertHoleAfter,
                        Word::NodeByName,
                        Word::Replace,
                    ],
                ),
            ),
            (
                Key::Char('f'),
                KmapFilter::ParentArity(vec![ArityType::Flexible, ArityType::Mixed]),
                Prog::named(
                    "False",
                    &[
                        Word::LangConstruct(lang, "false".into()),
                        Word::InsertHoleAfter,
                        Word::NodeByName,
                        Word::Replace,
                    ],
                ),
            ),
            (
                Key::Esc,
                KmapFilter::Always,
                Prog::named("Exit", &[Word::PopMap]),
            ),
        ])
    }
}

impl<'l> FilteredKmap<'l> {
    pub fn lookup(&self, key: Key) -> Result<Prog<'l>, Error> {
        self.0.get(&key).cloned().ok_or(Error::UnknownKey(key))
    }

    pub fn hints(&self) -> Vec<(String, String)> {
        let mut v: Vec<_> = self
            .0
            .iter()
            .map(|(key, prog)| (self.format_key(key), self.format_prog(prog)))
            .collect();
        v.sort_unstable();
        v
    }

    pub fn format_prog(&self, prog: &Prog) -> String {
        if let Some(ref name) = prog.name {
            name.to_string()
        } else if prog.words.len() == 1 {
            format!("{:?}", prog.words[0])
        } else {
            "...".into()
        }
    }

    pub fn format_key(&self, key: &Key) -> String {
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
