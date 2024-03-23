use partial_pretty_printer as ppp;
use std::fmt;
use std::time::Duration;

pub use crate::style::ColorTheme;

/// A front end for the editor. It knows how to render a frame and how to
/// receive keyboard events.
pub trait Frontend: Sized + ppp::pane::PrettyWindow {
    /// Construct a new frontend.
    fn new(theme: ColorTheme) -> Result<Self, Self::Error>;

    /// Block until an event (eg. keypress) occurs, then return it. `None` means that no event
    /// occurred before the timeout elapsed.
    fn next_event(&mut self, timeout: Duration) -> Result<Option<Event>, Self::Error>;

    /// Prepare to start modifying a fresh new frame. This should be called before pretty-printing.
    fn start_frame(&mut self) -> Result<(), Self::Error>;

    /// Show the modified frame to the user. This should be called after pretty-printing.
    fn show_frame(&mut self) -> Result<(), Self::Error>;
}

/// An input event.
pub enum Event {
    Key(Key),
    Mouse(MouseEvent),
    /// The window was resized.
    Resize,
    /// For "bracketed paste", which not all terminal emulators support.
    Paste(String),
}

pub struct MouseEvent {
    /// A character position, relative to the terminal window.
    pub click_pos: ppp::Pos,
    /// Which mouse button was clicked.
    pub button: MouseButton,
}

pub enum MouseButton {
    Left,
    Right,
}

/// If the code is a capitalized character, the modifiers will NOT include shift.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Key {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub alt: bool,
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
    F(u8),
    Char(char),
    Null,
    Esc,
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.modifiers.ctrl {
            write!(f, "C-");
        }
        if self.modifiers.alt {
            write!(f, "A-");
        }
        if self.modifiers.shift {
            write!(f, "S-");
        }
        match self.code {
            KeyCode::Backspace => write!(f, "bksp"),
            KeyCode::Enter => write!(f, "enter"),
            KeyCode::Left => write!(f, "←"),
            KeyCode::Right => write!(f, "→"),
            KeyCode::Up => write!(f, "↑"),
            KeyCode::Down => write!(f, "↓"),
            KeyCode::Home => write!(f, "home"),
            KeyCode::End => write!(f, "end"),
            KeyCode::PageUp => write!(f, "pg_up"),
            KeyCode::PageDown => write!(f, "pg_dn"),
            KeyCode::Tab => write!(f, "tab"),
            KeyCode::Delete => write!(f, "del"),
            KeyCode::Insert => write!(f, "ins"),
            KeyCode::F(num) => write!(f, "f{}", num),
            KeyCode::Char(' ') => write!(f, "spc"),
            KeyCode::Char(c) => write!(f, "{}", c),
            KeyCode::Null => write!(f, "null"),
            KeyCode::Esc => write!(f, "esc"),
        }
    }
}
