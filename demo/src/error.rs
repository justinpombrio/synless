use std::io;
use termion::event::Key;

use editor::DocError;
use frontends::terminal;
use language::{ConstructName, LanguageName};
use pretty::{DocLabel, PaneError};

use crate::keymaps::{MenuName, ModeName};

#[derive(Debug)]
pub enum ShellError {
    UnknownKey(Key),
    UnknownModeName(ModeName),
    UnknownMenuName(MenuName),
    NoKeymap,
    UnknownEvent,
    KeyboardInterrupt,
    ExpectedValue(String),
    EmptyStack,
    Io(io::Error),
    Term(terminal::Error),
    Core(CoreError<'static>),
}

#[derive(Debug)]
pub enum CoreError<'l> {
    UnknownLang(LanguageName),
    UnknownConstruct {
        construct: ConstructName,
        lang: LanguageName,
    },
    UnknownBookmark,
    UnknownDocLabel(DocLabel),
    Pane(PaneError<terminal::Error>),
    DocExec(DocError<'l>),
}

impl<'l> From<PaneError<terminal::Error>> for CoreError<'l> {
    fn from(e: PaneError<terminal::Error>) -> CoreError<'l> {
        CoreError::Pane(e)
    }
}

impl<'l> From<DocError<'l>> for CoreError<'l> {
    fn from(e: DocError<'l>) -> CoreError<'l> {
        CoreError::DocExec(e)
    }
}

impl From<CoreError<'static>> for ShellError {
    fn from(e: CoreError<'static>) -> ShellError {
        ShellError::Core(e)
    }
}

impl<'l> From<io::Error> for ShellError {
    fn from(e: io::Error) -> ShellError {
        ShellError::Io(e)
    }
}

impl<'l> From<terminal::Error> for ShellError {
    fn from(e: terminal::Error) -> ShellError {
        ShellError::Term(e)
    }
}
