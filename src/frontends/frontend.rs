use partial_pretty_printer as ppp;

pub use super::key::Key;
pub use crate::style::ColorTheme;

// TODO: mouse events

/// An input event.
pub enum Event {
    /// A key was pressed down.
    KeyEvent(Key),
    /// The left mouse button was pressed at the given character
    /// position (relative to the terminal window).
    MouseEvent(ppp::Pos),
}

/// A front end for the editor. It knows how to render a frame and how to
/// receive keyboard events.
pub trait Frontend: Sized + ppp::pane::PrettyWindow {
    /// Construct a new frontend.
    fn new(theme: ColorTheme) -> Result<Self, Self::Error>;

    /// Block until an event (eg. keypress) occurs, then return it. None means the event stream ended.
    fn next_event(&mut self) -> Option<Result<Event, Self::Error>>;

    /// Prepare to start modifying a fresh new frame. This should be called before pretty-printing.
    fn start_frame(&mut self) -> Result<(), Self::Error>;

    /// Show the modified frame to the user. This should be called after pretty-printing.
    fn show_frame(&mut self) -> Result<(), Self::Error>;
}
