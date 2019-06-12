use frontends::terminal;
use std::io;

#[derive(Debug)]
pub enum Error {
    UnknownKey(char),
    UnknownEvent,
    Io(io::Error),
    Term(terminal::Error),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

impl From<terminal::Error> for Error {
    fn from(e: terminal::Error) -> Error {
        Error::Term(e)
    }
}
