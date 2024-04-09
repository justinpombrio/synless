use crate::util::SynlessBug;
use partial_pretty_printer as ppp;
use std::fmt;
use std::str::FromStr;
use std::time::Duration;

pub use crate::style::ColorTheme;

/// A front end for the editor. It knows how to render a frame and how to
/// receive keyboard and mouse events.
pub trait Frontend: Sized + ppp::pane::PrettyWindow {
    /// Construct a new frontend.
    fn new(theme: ColorTheme) -> Result<Self, Self::Error>;

    /// Set the color theme. Must not be called between `start_frame()` and `end_frame()`.
    fn set_color_theme(&mut self, theme: ColorTheme) -> Result<(), Self::Error>;

    /// Block until an event (eg. keypress) occurs, then return it. `None` means that no event
    /// occurred before the timeout elapsed.
    fn next_event(&mut self, timeout: Duration) -> Result<Option<Event>, Self::Error>;

    /// Prepare to start modifying a fresh new frame. This must be called before pretty-printing.
    fn start_frame(&mut self) -> Result<(), Self::Error>;

    /// Show the modified frame to the user. This must be called after pretty-printing.
    fn end_frame(&mut self) -> Result<(), Self::Error>;
}

/// An input event.
pub enum Event {
    Key(Key),
    Mouse(MouseEvent),
    /// The window was resized. Call `.size()` to get the new size.
    Resize,
    /// For "bracketed paste", which not all terminal emulators support.
    Paste(String),
}

pub struct MouseEvent {
    /// A character grid position, relative to the window.
    pub click_pos: ppp::Pos,
    /// Which mouse button was clicked.
    pub button: MouseButton,
}

pub enum MouseButton {
    Left,
    Middle,
    Right,
}

/// If the key code can be capitalized, then shift is indicated by capitalizing it and _not_
/// setting the shift modifier.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Key {
    code: KeyCode,
    modifiers: KeyModifiers,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub alt: bool,
    /// (See comment about shift on `Key`.)
    pub shift: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum KeyCode {
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    Delete,
    Insert,
    Esc,
    F(u8),
    Char(char),
}

impl Key {
    /// If the code is `KeyCode::Char(ch)`, then the modifiers must not contain `shift`.
    ///
    /// This is because we don't necessarily know what keycode we will receive from the Frontend if
    /// shift and `ch` are typed together, because that depends on the user's keyboard layout.
    /// Therefore we wouldn't be able to tell whether a Key we receive from the Frontend is the
    /// "same" as this Key.
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Option<Key> {
        match (code, modifiers.shift) {
            (KeyCode::Char(_), true) => None,
            _ => Some(Key { code, modifiers }),
        }
    }

    pub fn code(&self) -> KeyCode {
        self.code
    }

    pub fn modifiers(&self) -> KeyModifiers {
        self.modifiers
    }
}

#[derive(thiserror::Error, fmt::Debug)]
#[error("Failed to parse key from string")]
pub struct KeyParseError;

impl FromStr for Key {
    type Err = KeyParseError;

    fn from_str(s: &str) -> Result<Self, KeyParseError> {
        use KeyCode::*;

        // Parse modifiers
        let (ctrl, s) = match s.strip_prefix("C-") {
            None => (false, s),
            Some(suffix) => (true, suffix),
        };
        let (alt, s) = match s.strip_prefix("A-") {
            None => (false, s),
            Some(suffix) => (true, suffix),
        };
        let (shift, s) = match s.strip_prefix("S-") {
            None => (false, s),
            Some(suffix) => (true, suffix),
        };
        let modifiers = KeyModifiers { ctrl, alt, shift };

        // Parse key code
        let code = match s {
            "bksp" => Backspace,
            "enter" => Enter,
            "left" | "←" => Left,
            "right" | "→" => Right,
            "up" | "↑" => Up,
            "down" | "↓" => Down,
            "home" => Home,
            "end" => End,
            "pg_up" => PageUp,
            "pg_dn" => PageDown,
            "tab" => Tab,
            "del" => Delete,
            "ins" => Insert,
            "esc" => Esc,
            "space" => Char(' '),
            other => {
                if other.chars().count() == 1 {
                    Char(other.chars().next().bug())
                } else if let Some(numeral) = other.strip_prefix('F') {
                    match u8::from_str(numeral) {
                        Ok(number) => F(number),
                        Err(_) => return Err(KeyParseError),
                    }
                } else {
                    return Err(KeyParseError);
                }
            }
        };

        Key::new(code, modifiers).ok_or(KeyParseError)
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use KeyCode::*;

        if self.modifiers.ctrl {
            write!(f, "C-")?;
        }
        if self.modifiers.alt {
            write!(f, "A-")?;
        }
        if self.modifiers.shift {
            write!(f, "S-")?;
        }
        match self.code {
            Backspace => write!(f, "bksp"),
            Enter => write!(f, "enter"),
            Left => write!(f, "←"),
            Right => write!(f, "→"),
            Up => write!(f, "↑"),
            Down => write!(f, "↓"),
            Home => write!(f, "home"),
            End => write!(f, "end"),
            PageUp => write!(f, "pg_up"),
            PageDown => write!(f, "pg_dn"),
            Tab => write!(f, "tab"),
            Delete => write!(f, "del"),
            Insert => write!(f, "ins"),
            Esc => write!(f, "esc"),
            F(num) => write!(f, "F{}", num),
            Char(' ') => write!(f, "space"),
            Char(c) => write!(f, "{}", c),
        }
    }
}
