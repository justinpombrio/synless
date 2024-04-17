use std::error::Error;
use std::fmt;

pub use crate::error;

#[derive(Debug, Clone)]
pub struct SynlessError {
    pub message: String,
    pub category: ErrorCategory,
}

#[derive(Debug, Clone, Copy)]
pub enum ErrorCategory {
    Doc,
    Edit,
    Frontend,
    Language,
    Parse,
    Printing,
    Event,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ErrorCategory {
    fn is_fatal(&self) -> bool {
        use ErrorCategory::*;

        match self {
            Edit => false,
            Doc | Frontend | Language | Parse | Printing | Event => true,
        }
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
        SynlessError {
            message: format!($message, $( $arg ),*),
            category: $crate::util::ErrorCategory::$category,
        }
    };
}

impl rhai::CustomType for SynlessError {
    fn build(mut builder: rhai::TypeBuilder<Self>) {
        builder
            .with_name("Error")
            .with_get("message", |err: &mut SynlessError| -> String {
                err.message.clone()
            })
            .with_get("category", |err: &mut SynlessError| -> ErrorCategory {
                err.category
            })
            .with_fn("is_fatal", |err: &mut SynlessError| -> bool {
                err.is_fatal()
            });
    }
}
