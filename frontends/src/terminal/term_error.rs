use std::io;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    OutOfBounds,
    UnknownKey,
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}
