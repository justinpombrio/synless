use std::convert::TryFrom;
use termion;

/// A keypress. Based on the `termion` crate's `event::Key` enum.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    Backspace,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Delete,
    Insert,
    F(u8),
    Char(char),
    Alt(char),
    Ctrl(char),
    Null,
    Esc,
}

impl TryFrom<termion::event::Key> for Key {
    type Error = ();
    fn try_from(termion_key: termion::event::Key) -> Result<Self, Self::Error> {
        Ok(match termion_key {
            termion::event::Key::Backspace => Key::Backspace,
            termion::event::Key::Left => Key::Left,
            termion::event::Key::Right => Key::Right,
            termion::event::Key::Up => Key::Up,
            termion::event::Key::Down => Key::Down,
            termion::event::Key::Home => Key::Home,
            termion::event::Key::End => Key::End,
            termion::event::Key::PageUp => Key::PageUp,
            termion::event::Key::PageDown => Key::PageDown,
            termion::event::Key::Delete => Key::Delete,
            termion::event::Key::Insert => Key::Insert,
            termion::event::Key::F(i) => Key::F(i),
            termion::event::Key::Char(c) => Key::Char(c),
            termion::event::Key::Alt(c) => Key::Alt(c),
            termion::event::Key::Ctrl(c) => Key::Ctrl(c),
            termion::event::Key::Null => Key::Null,
            termion::event::Key::Esc => Key::Esc,
            _ => return Err(()),
        })
    }
}
