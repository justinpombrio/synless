use std::fmt;

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

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Key::Backspace => write!(f, "Bksp"),
            Key::Left => write!(f, "←"),
            Key::Right => write!(f, "→"),
            Key::Up => write!(f, "↑"),
            Key::Down => write!(f, "↓"),
            Key::Home => write!(f, "Home"),
            Key::End => write!(f, "End"),
            Key::PageUp => write!(f, "PgUp"),
            Key::PageDown => write!(f, "PgDn"),
            Key::Delete => write!(f, "Del"),
            Key::Insert => write!(f, "Ins"),
            Key::F(num) => write!(f, "F{}", num),
            Key::Char(' ') => write!(f, "Spc"),
            Key::Char(c) => write!(f, "{}", c),
            Key::Alt(' ') => write!(f, "A-Spc"),
            Key::Alt(c) => write!(f, "A-{}", c),
            Key::Ctrl(' ') => write!(f, "C-Spc"),
            Key::Ctrl(c) => write!(f, "C-{}", c),
            Key::Null => write!(f, "Null"),
            Key::Esc => write!(f, "Esc"),
        }
    }
}
