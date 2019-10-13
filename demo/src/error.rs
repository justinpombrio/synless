use std::io;
use termion::event::Key;

use editor::DocError;
use frontends::terminal;
use language::{ConstructName, LanguageName};
use pretty::{DocLabel, PaneError};

#[derive(Debug)]
pub enum ShellError {
    UnknownKey(Key),
    UnknownKeymap(String),
    NoKeymap,
    UnknownEvent,
    KeyboardInterrupt,
    ExpectedValue(String),
    EmptyStack,
    Io(io::Error),
    Term(terminal::Error),
    Core(CoreError),
}

#[derive(Debug)]
pub enum CoreError {
    UnknownLang(LanguageName),
    UnknownConstruct {
        construct: ConstructName,
        lang: LanguageName,
    },
    UnknownBookmark,
    UnknownDocLabel(DocLabel),
    Pane(PaneError<terminal::Error>),
    DocExec(DocError<'static>),
}

impl From<PaneError<terminal::Error>> for CoreError {
    fn from(e: PaneError<terminal::Error>) -> CoreError {
        CoreError::Pane(e)
    }
}

impl From<DocError<'static>> for CoreError {
    fn from(e: DocError<'static>) -> CoreError {
        CoreError::DocExec(e)
    }
}

impl From<CoreError> for ShellError {
    fn from(e: CoreError) -> ShellError {
        ShellError::Core(e)
    }
}

impl From<io::Error> for ShellError {
    fn from(e: io::Error) -> ShellError {
        ShellError::Io(e)
    }
}

impl From<terminal::Error> for ShellError {
    fn from(e: terminal::Error) -> ShellError {
        ShellError::Term(e)
    }
}
