use partial_pretty_printer as ppp;
use std::fmt;
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
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
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
            KeyCode::Esc => write!(f, "esc"),
            KeyCode::F(num) => write!(f, "f{}", num),
            KeyCode::Char(' ') => write!(f, "space"),
            KeyCode::Char(c) => write!(f, "{}", c),
        }
    }
}
