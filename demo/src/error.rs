use std::io;
use termion::event::Key;

use editor::DocError;
use frontends::terminal;
use language::{ConstructName, LanguageName};
use pretty::PaneError;

#[derive(Debug)]
pub enum Error {
    UnknownKey(Key),
    UnknownKeymap(String),
    NoKeymap,
    UnknownEvent,
    KeyboardInterrupt,
    UnknownLang(LanguageName),
    UnknownConstruct {
        construct: ConstructName,
        lang: LanguageName,
    },
    ExpectedWord(String),
    EmptyStack,
    Pane(PaneError<terminal::Error>),
    DocExec(DocError<'static>),
    Io(io::Error),
    Term(terminal::Error),
}

impl From<PaneError<terminal::Error>> for Error {
    fn from(e: PaneError<terminal::Error>) -> Error {
        Error::Pane(e)
    }
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

impl From<DocError<'static>> for Error {
    fn from(e: DocError<'static>) -> Error {
        Error::DocExec(e)
    }
}
