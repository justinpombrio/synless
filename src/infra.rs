use crate::bug;
use std::fmt;

#[doc(hidden)]
#[macro_export]
/// You can add this to the top of the body of a function to include it in
/// flamegraphs when `--features profile` is used.
macro_rules! trace {
    ($name:expr) => {
        #[cfg(feature = "profile")]
        no_nonsense_flamegraphs::span!($name);
    };
}

/// Versions of `.unwrap()` and `.expect()` that print a Synless-specific
/// error message, and format errors with Display instead of Debug.
pub trait SynlessBug<T>: Sized {
    /// Like `.unwrap()`, but with a better error message.
    fn bug(self) -> T;
    /// Like `.expect()`, but with a better error message.
    fn bug_msg(self, msg: &str) -> T;
}

impl<T> SynlessBug<T> for Option<T> {
    fn bug(self) -> T {
        match self {
            Some(val) => val,
            None => bug!("Tried to unwrap a `None` value"),
        }
    }

    fn bug_msg(self, msg: &str) -> T {
        match self {
            Some(val) => val,
            None => bug!("{}", msg),
        }
    }
}

impl<T, E: fmt::Display> SynlessBug<T> for Result<T, E> {
    fn bug(self) -> T {
        match self {
            Ok(ok) => ok,
            Err(err) => bug!("{}", err),
        }
    }

    fn bug_msg(self, msg: &str) -> T {
        match self {
            Ok(ok) => ok,
            Err(err) => bug!("{}\n{}", msg, err),
        }
    }
}

#[track_caller]
pub(crate) fn format_bug(location: String, message: String) -> String {
    let mut output = "\n*** Bug in Synless.".to_owned();
    output
        .push_str("\n*** Please open an issue at https://github.com/justinpombrio/synless/issues.");
    output.push_str("\n*** Location:");
    output.push_str("\n***   ");
    output.push_str(&location);
    output.push_str("\n*** Error message:");
    for line in message.lines() {
        output.push_str("\n***   ");
        output.push_str(line);
    }
    output.push('\n');
    output
}

#[doc(hidden)]
#[macro_export]
/// Like `panic!()`, but with a better error message.
macro_rules! bug {
    ($message:literal) => {
        bug!($message,)
    };
    ($message:literal, $( $arg:expr ),*) => {
        panic!("{}",
            $crate::infra::format_bug(
                format!("{}:{}:{}", file!(), line!(), column!()),
                format!($message, $( $arg ),*)
            )
        )
    };
}
