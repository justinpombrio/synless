use pretty::{ColorTheme, Pos};

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
    type Error;

    /// Construct a new frontend.
    fn new(theme: ColorTheme) -> Result<Self, Self::Error>;

    /// Block until an event (eg. keypress) occurs, then return it. None means the event stream ended.
    fn next_event(&mut self) -> Option<Result<Event, Self::Error>>;

    /// Return the current size of the screen in characters.
    fn size(&self) -> Result<Pos, Self::Error>;

    /// Prepare to start modifying a fresh new frame.
    fn start_frame(&mut self) -> Result<(), Self::Error>;

    /// Show the modified frame to the user.
    fn show_frame(&mut self) -> Result<(), Self::Error>;
}
