use std::collections::HashMap;
use termion::event::Key;

use crate::prog::{Prog, Word};

pub struct Keymap<'l>(pub HashMap<Key, Prog<'l>>);

impl<'l> Keymap<'l> {
    pub fn normal() -> Self {
        let map = vec![
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
                Prog::named("InsertAfter", &[Word::SelectNode, Word::InsertAfter]),
            ),
            (
                Key::Char('o'),
                Prog::named("InsertPostpend", &[Word::SelectNode, Word::InsertPostpend]),
            ),
            (
                Key::Char('r'),
                Prog::named("Replace", &[Word::SelectNode, Word::Replace]),
            ),
            (
                Key::Char(' '),
                Prog::named(
                    "SpeedMode",
                    &[
                        Word::Message("entering speed-bool mode!".into()),
                        Word::Echo,
                        Word::MapName("space".into()),
                        Word::PushMap,
                    ],
                ),
            ),
        ]
        .into_iter()
        .collect();

        Keymap(map)
    }

    pub fn space() -> Self {
        let map = vec![
            (
                Key::Char('t'),
                Prog::named(
                    "True",
                    &[
                        Word::NodeName("true".into()),
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
                        Word::NodeName("false".into()),
                        Word::NodeByName,
                        Word::InsertAfter,
                    ],
                ),
            ),
            (
                Key::Char(' '),
                Prog::named(
                    "Exit",
                    &[
                        Word::Message("leaving speed-bool mode!".into()),
                        Word::Echo,
                        Word::PopMap,
                    ],
                ),
            ),
        ]
        .into_iter()
        .collect();

        Keymap(map)
    }

    pub fn summary(&self) -> String {
        let mut s = String::new();
        for (key, prog) in &self.0 {
            let prog_name = if let Some(ref name) = prog.name {
                name.to_string()
            } else if prog.words.len() == 1 {
                format!("{}", prog.words[0])
            } else {
                "...".into()
            };
            s += &format!("{}:{}, ", self.format_key(key), prog_name);
        }
        s
    }

    fn format_key(&self, key: &Key) -> String {
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
