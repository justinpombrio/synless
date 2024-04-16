mod runtime;

use crate::engine::EngineError;
use crate::frontends::Frontend;
use std::error::Error;
use std::fmt;

pub use runtime::Runtime;

// TODO: Think about errors!

#[derive(Debug, Clone)]
pub struct SynlessError {
    is_fatal_to_script: bool,
    short_message: String,
    long_message: String,
}

impl fmt::Display for SynlessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.long_message)
    }
}

impl SynlessError {
    fn new_fatal(message_prefix: &str, error: impl Error + 'static) -> SynlessError {
        SynlessError::new(message_prefix, error, true)
    }

    fn new_non_fatal(message_prefix: &str, error: impl Error + 'static) -> SynlessError {
        SynlessError::new(message_prefix, error, false)
    }

    fn new(message_prefix: &str, error: impl Error + 'static, is_fatal: bool) -> SynlessError {
        use std::fmt::Write;

        let mut long_message = String::new();
        write!(long_message, "{}: ", message_prefix);
        let mut error = &error as &dyn Error;
        write!(long_message, "{}", error);
        while let Some(next_error) = error.source() {
            error = next_error;
            write!(long_message, ": {}", error);
        }
        let short_message = format!("{}", error);

        SynlessError {
            is_fatal_to_script: is_fatal,
            long_message,
            short_message,
        }
    }
}

impl rhai::CustomType for SynlessError {
    fn build(mut builder: rhai::TypeBuilder<Self>) {
        builder
            .with_name("SynlessError")
            .with_get("is_fatal", |err: &mut SynlessError| -> bool {
                err.is_fatal_to_script
            })
            .with_get("short_message", |err: &mut SynlessError| -> String {
                err.short_message.clone()
            })
            .with_get("long_message", |err: &mut SynlessError| -> String {
                err.long_message.clone()
            });
    }
}
