use common::*;
use style::*;

pub use rustbox::Key;

// TODO: mouse events


/// An input event.
pub enum Event {
    /// A key was pressed down.
    KeyEvent(Key), // rustbox key
    /// The left mouse button was pressed at the given character
    /// position (relative to the terminal window).
    MouseEvent(Pos)
}


/// A front end for the editor. It knows how to render to a screen,
/// and how to receive keyboard events.
pub trait Frontend {
    /// Construct a new frontend.
    fn new(theme: ColorTheme) -> Self;

    /// Flip the buffer to show the screen. Nothing will show up until
    /// you call this function!
    fn present(&mut self);
    
    /// Render a string with plain style.
    /// No newlines allowed.
    fn simple_print(&mut self, text: &str, pos: Pos) {
        self.print_str(text, pos, Style::plain());
    }

    /// Render a string with the given style at the given position.
    /// No newlines allowed.
    fn print_str(&mut self, text: &str, pos: Pos, style: Style) {
        for (i, ch) in text.chars().enumerate() {
            let pos = Pos {
                row: pos.row + i as u32,
                col: pos.col
            };
            self.print_char(ch, pos, style);
        }
    }

    /// Render a character with the given style at the given position.
    /// No newlines allowed.
    fn print_char(&mut self, ch: char, pos: Pos, style: Style);
    
    /// Clear the whole screen.
    fn clear(&mut self);

    /// Return the current size of the screen in characters.
    fn size(&self) -> Pos;

    /// Poll for keyboard and mouse events. `None` means no event.
    fn poll_event(&self) -> Option<Event>;
}
