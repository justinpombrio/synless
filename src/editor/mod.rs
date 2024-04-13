// TODO: pull out keymap, layer, menu

mod app;
mod interpreter;
mod runtime;
mod stack;

use crate::engine::EngineError;
use crate::frontends::Frontend;
use std::error::Error;
use std::fmt;

pub use stack::{Op, Prog, Value};

// TODO: Is this InterpreterError, AppError, a mix of both?
#[derive(thiserror::Error, fmt::Debug)]
pub enum EditorError {
    #[error("Error from the document engine")]
    EngineError(#[from] EngineError),
    #[error("Attempted to pop an empty data stack")]
    EmptyDataStack,
    #[error("Expected type '{expected}' but found '{actual}'")]
    TypeMismatch { actual: String, expected: String },
    #[error("I was rudely interrupted by Ctrl-C")]
    KeyboardInterrupt,
    #[error("Frontend error")]
    FrontendError(#[source] Box<dyn Error>),
    #[error("Pane error")]
    PaneError(#[source] Box<dyn Error>),
}
