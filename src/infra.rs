use std::fmt;
use std::panic::Location;

pub use crate::bug;

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
    #[track_caller]
    fn bug(self) -> T {
        match self {
            Some(val) => val,
            None => {
                panic!(
                    "{}",
                    format_bug_at(Location::caller(), "Tried to unwrap a `None` value")
                );
            }
        }
    }

    #[track_caller]
    fn bug_msg(self, msg: &str) -> T {
        match self {
            Some(val) => val,
            None => {
                panic!("{}", format_bug_at(Location::caller(), msg));
            }
        }
    }
}

impl<T, E: fmt::Display> SynlessBug<T> for Result<T, E> {
    #[track_caller]
    fn bug(self) -> T {
        match self {
            Ok(ok) => ok,
            Err(err) => panic!("{}", format_bug_at(Location::caller(), &format!("{}", err))),
        }
    }

    #[track_caller]
    fn bug_msg(self, msg: &str) -> T {
        match self {
            Ok(ok) => ok,
            Err(err) => panic!(
                "{}",
                format_bug_at(Location::caller(), &format!("{}\n{}", msg, err))
            ),
        }
    }
}

pub(crate) fn format_bug_at(loc: &Location, message: &str) -> String {
    let loc_str = format!("{}:{}:{}", loc.file(), loc.line(), loc.column());
    format_bug(loc_str, message)
}

pub(crate) fn format_bug(location: String, message: &str) -> String {
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
                &format!($message, $( $arg ),*)
            )
        )
    };
}
