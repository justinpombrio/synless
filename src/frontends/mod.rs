mod frontend;
mod screen_buf;
mod terminal;

pub use frontend::{Event, Frontend, Key, KeyCode, KeyModifiers, MouseButton, MouseEvent};
pub use terminal::{Terminal, TerminalError};

use crate::util::{error, SynlessError};
use partial_pretty_printer::pane;
use std::error::Error;

impl<W: Error + 'static, E: Error + 'static> From<pane::PaneError<W, E>> for SynlessError {
    fn from(error: pane::PaneError<W, E>) -> SynlessError {
        use pane::PaneError::{InvalidUseOfDynamic, PrettyWindowError, PrintingError};

        match &error {
            InvalidUseOfDynamic => error!(Frontend, "{}", error.to_string()),
            PrettyWindowError(err) => error!(Frontend, "{}", err.to_string()),
            PrintingError(err) => error!(Printing, "{}", err.to_string()),
        }
    }
}
