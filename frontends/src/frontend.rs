use pretty::{ColorTheme, Pos, PrettyScreen, Style};

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
pub trait Frontend: Sized + PrettyScreen {
    /// Construct a new frontend.
    fn new(theme: ColorTheme) -> Result<Self, Self::Error>;

    /// Clear the whole screen.
    fn clear(&mut self) -> Result<(), Self::Error>;

    /// Iterate over all key and mouse events, blocking on `next()`.
    fn next_event(&mut self) -> Option<Result<Event, Self::Error>>;

    /// Render a string with plain style.
    /// No newlines allowed.
    fn simple_print(&mut self, text: &str, pos: Pos) -> Result<(), Self::Error> {
        self.print(pos, text, Style::plain())
    }

    /// Render a character with the given style at the given position.
    /// No newlines allowed.
    fn print_char(&mut self, ch: char, pos: Pos, style: Style) -> Result<(), Self::Error> {
        self.print(pos, &ch.to_string(), style)
    }

    /// Return the current size of the screen in characters.
    fn size(&self) -> Result<Pos, Self::Error> {
        let region = self.region()?;
        Ok(Pos {
            col: region.bound.width,
            row: region.bound.height,
        })
    }
}
