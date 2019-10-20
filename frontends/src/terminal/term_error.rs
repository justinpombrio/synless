use std::io;
use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum TermError {
    #[error("terminal input/output error: {0}")]
    Io(#[from] io::Error),

    #[error("position outside window boundary")]
    OutOfBounds,

    #[error("unknown key pressed")]
    UnknownKey,
}
