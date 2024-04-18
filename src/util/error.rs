use std::error::Error;
use std::fmt;

pub use crate::error;

#[derive(Debug, Clone)]
pub struct SynlessError {
    pub message: String,
    pub category: ErrorCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    Keymap,
    FileSystem,
    Doc,
    Edit,
    Frontend,
    Language,
    Parse,
    Printing,
    Escape,
    Abort,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ErrorCategory {
    fn is_fatal(&self) -> bool {
        !matches!(self, ErrorCategory::Edit)
    }
}

impl SynlessError {
    pub fn is_fatal(&self) -> bool {
        self.category.is_fatal()
    }
}

impl fmt::Display for SynlessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} Error: {}", self.category, self.message)
    }
}

/// Construct a [`SynlessError`].
#[macro_export]
macro_rules! error {
    ($category:ident, $message:literal) => {
        $crate::error!($category, $message,)
    };
    ($category:ident, $message:literal, $( $arg:expr ),*) => {
        $crate::util::SynlessError {
            message: format!($message, $( $arg ),*),
            category: $crate::util::ErrorCategory::$category,
        }
    };
}

impl rhai::CustomType for SynlessError {
    fn build(mut builder: rhai::TypeBuilder<Self>) {
        builder
            .with_name("SynlessError")
            .with_get("message", |err: &mut SynlessError| -> String {
                err.message.clone()
            })
            .with_get("category", |err: &mut SynlessError| -> String {
                format!("{}", err.category)
            })
            .with_fn("is_fatal", |err: &mut SynlessError| -> bool {
                err.is_fatal()
            });
    }
}

impl From<SynlessError> for Box<rhai::EvalAltResult> {
    fn from(error: SynlessError) -> Box<rhai::EvalAltResult> {
        Box::new(rhai::EvalAltResult::ErrorRuntime(
            rhai::Dynamic::from(error),
            rhai::Position::NONE,
        ))
    }
}
