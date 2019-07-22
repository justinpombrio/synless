use pretty::{ColorTheme, Pane, Pos, PrettyWindow};

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

/// A front end for the editor. It knows how to render a frame and how to
/// receive keyboard events.
pub trait Frontend: Sized {
    type Error;
    type Window: PrettyWindow;

    /// Construct a new frontend.
    fn new(theme: ColorTheme) -> Result<Self, Self::Error>;

    /// Block until an event (eg. keypress) occurs, then return it. None means the event stream ended.
    fn next_event(&mut self) -> Option<Result<Event, Self::Error>>;

    /// Use the given `drawer` closure to draw a complete frame to this Frontend's window.
    fn draw_frame<F>(&mut self, drawer: F) -> Result<(), Self::Error>
    where
        F: Fn(Pane<Self::Window>) -> Result<(), Self::Error>;
}
