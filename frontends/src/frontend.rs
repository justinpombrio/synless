use pretty::{ColorTheme, Pos, Style};

pub use termion::event::Key;

// TODO: mouse events

/// An input event.
pub enum Event {
    /// A key was pressed down.
    KeyEvent(Key), // termion key. TODO: don't have trait depend on termion
    /// The left mouse button was pressed at the given character
    /// position (relative to the terminal window).
    MouseEvent(Pos),
}

/// A front end for the editor. It knows how to render to a screen,
/// and how to receive keyboard events.
pub trait Frontend: Sized {
    type Error: Sized;

    /// Construct a new frontend.
    fn new(theme: ColorTheme) -> Result<Self, Self::Error>;

    /// Flip the buffer to show the screen. Nothing will show up until
    /// you call this function!
    fn present(&mut self) -> Result<(), Self::Error>;

    /// Render a string with plain style.
    /// No newlines allowed.
    fn simple_print(&mut self, text: &str, pos: Pos) -> Result<(), Self::Error> {
        self.print_str(text, pos, Style::plain())
    }

    /// Render a string with the given style at the given position.
    /// No newlines allowed.
    fn print_str(&mut self, text: &str, pos: Pos, style: Style) -> Result<(), Self::Error>;

    /// Render a character with the given style at the given position.
    /// No newlines allowed.
    fn print_char(&mut self, ch: char, pos: Pos, style: Style) -> Result<(), Self::Error> {
        self.print_str(&ch.to_string(), pos, style)
    }

    /// Clear the whole screen.
    fn clear(&mut self) -> Result<(), Self::Error>;

    /// Return the current size of the screen in characters.
    fn size(&self) -> Result<Pos, Self::Error>;

    /// Iterate over all key and mouse events, blocking on `next()`.
    fn next_event(&mut self) -> Option<Result<Event, Self::Error>>;
}
