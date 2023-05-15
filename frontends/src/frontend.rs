use partial_pretty_printer::pane::PrettyWindow;
use partial_pretty_printer::Pos;

pub use super::key::Key;
pub use super::ColorTheme;

// TODO: mouse events

/// An input event.
pub enum Event {
    /// A key was pressed down.
    KeyEvent(Key),
    /// The left mouse button was pressed at the given character
    /// position (relative to the terminal window).
    MouseEvent(Pos),
}

/// A front end for the editor. It knows how to render a frame and how to
/// receive keyboard events.
pub trait Frontend: Sized + PrettyWindow {
    type Error: std::error::Error;
    type Window: PrettyWindow;

    /// Construct a new frontend.
    fn new(theme: ColorTheme) -> Result<Self, <Self as Frontend>::Error>;

    /// Block until an event (eg. keypress) occurs, then return it. None means the event stream ended.
    fn next_event(&mut self) -> Option<Result<Event, <Self as Frontend>::Error>>;
}
