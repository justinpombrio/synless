use std::collections::HashMap;
use std::iter;
use termion::event::Key;

use crate::prog::{Prog, Word};
use language::{Language, LanguageName};

pub struct Keymap<'l>(pub HashMap<Key, Prog<'l>>);

impl<'l> Keymap<'l> {
    pub fn node(lang: &Language) -> Self {
        Keymap(
            lang.keymap()
                .iter()
                .map(|(&ch, construct_name)| {
                    (
                        Key::Char(ch),
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

    pub fn tree() -> Self {
        let map = vec![
            (Key::Char('d'), Prog::single(Word::Cut)),
            (Key::Char('y'), Prog::single(Word::Copy)),
            (Key::Char('p'), Prog::single(Word::PasteAfter)),
            (Key::Char('P'), Prog::single(Word::PasteBefore)),
            (Key::Ctrl('p'), Prog::single(Word::PastePrepend)),
            (Key::Alt('p'), Prog::single(Word::PastePostpend)),
            (Key::Char('R'), Prog::single(Word::PasteReplace)),
            (
                Key::Char('a'),
                Prog::named("TypeA", &[Word::Char('a'), Word::InsertChar]),
            ),
            (Key::Char('u'), Prog::single(Word::Undo)),
            (Key::Ctrl('r'), Prog::single(Word::Redo)),
            (Key::Right, Prog::single(Word::Right)),
            (Key::Left, Prog::single(Word::Left)),
            (Key::Up, Prog::single(Word::Parent)),
            (Key::Backspace, Prog::single(Word::Remove)),
            (
                Key::Down,
                Prog::named("Child", &[Word::Usize(0), Word::Child]),
            ),
            (
                Key::Char('i'),
                Prog::named(
                    "InsertAfter",
                    &[
                        Word::InsertAfter.quote(),
                        Word::MapName("node".into()),
                        Word::PushMap,
                    ],
                ),
            ),
            (
                Key::Char('I'),
                Prog::named(
                    "InsertBefore",
                    &[
                        Word::InsertBefore.quote(),
                        Word::MapName("node".into()),
                        Word::PushMap,
                    ],
                ),
            ),
            (
                Key::Char('o'),
                Prog::named(
                    "InsertPostpend",
                    &[
                        Word::InsertPostpend.quote(),
                        Word::MapName("node".into()),
                        Word::PushMap,
                    ],
                ),
            ),
            (
                Key::Char('O'),
                Prog::named(
                    "InsertPrepend",
                    &[
                        Word::InsertPrepend.quote(),
                        Word::MapName("node".into()),
                        Word::PushMap,
                    ],
                ),
            ),
            (
                Key::Char('r'),
                Prog::named(
                    "Replace",
                    &[
                        Word::Replace.quote(),
                        Word::MapName("node".into()),
                        Word::PushMap,
                    ],
                ),
            ),
            (
                Key::Char(' '),
                Prog::named(
                    "SpeedBoolMode",
                    &[Word::MapName("speed_bool".into()), Word::PushMap],
                ),
            ),
        ]
        .into_iter()
        .collect();

        Keymap(map)
    }

    pub fn speed_bool() -> Self {
        let lang: LanguageName = "json".into();
        let map = vec![
            (
                Key::Char('t'),
                Prog::named(
                    "True",
                    &[
                        Word::LangConstruct(lang.clone(), "true".into()),
                        Word::NodeByName,
                        Word::InsertAfter,
                    ],
                ),
            ),
            (
                Key::Char('f'),
                Prog::named(
                    "False",
                    &[
                        Word::LangConstruct(lang, "false".into()),
                        Word::NodeByName,
                        Word::InsertAfter,
                    ],
                ),
            ),
            (Key::Esc, Prog::named("Exit", &[Word::PopMap])),
        ]
        .into_iter()
        .collect();

        Keymap(map)
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
