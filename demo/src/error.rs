use std::io;
use thiserror;

use editor::DocError;
use frontends::{terminal::TermError, Key};
use language::{ConstructName, LanguageName};
use pretty::{DocLabel, PaneError};

use crate::keymaps::{MenuName, ModeName};

#[derive(thiserror::Error, Debug)]
pub enum ServerError<'l> {
    #[error("not in keymap: {0:?}")]
    UnknownKey(Key),

    #[error("unknown keymap mode: {0:?}")]
    UnknownModeName(ModeName),

    #[error("unknown keymap menu: {0:?}")]
    UnknownMenuName(MenuName),

    #[error("no keymap mode selected")]
    NoMode,

    #[error("unknown user input event")]
    UnknownEvent,

    #[error("received keyboard interrupt")]
    KeyboardInterrupt,

    // TODO include actual type too
    #[error("expected value of type {0} on data stack")]
    ExpectedValue(String),

    #[error("data stack was unexpectedly empty")]
    EmptyDataStack,

    #[error("input/output error: {0}")]
    Io(#[from] io::Error),

    #[error("terminal error: {0}")]
    Term(#[from] TermError),

    // Note: we can't use the inner EngineError as the error source because it
    // contains a non-static lifetime.
    #[error("engine error: {0}")]
    Engine(EngineError<'l>),
}

impl<'l> From<EngineError<'l>> for ServerError<'l> {
    fn from(e: EngineError<'l>) -> ServerError<'l> {
        ServerError::Engine(e)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum EngineError<'l> {
    #[error("unknown language: {0}")]
    UnknownLang(LanguageName),

    #[error("language {lang} does not contain construct {construct}")]
    UnknownConstruct {
        construct: ConstructName,
        lang: LanguageName,
    },

    #[error("no bookmark stored with this key")]
    UnknownBookmark,

    #[error("no document with label {0:?}")]
    UnknownDocLabel(DocLabel),

    #[error("pane error: {0}")]
    Pane(#[from] PaneError),

    // Note: we can't use the inner DocError as the error source because it
    // contains a non-static lifetime.
    #[error("doc error: {0}")]
    DocExec(DocError<'l>),
}

impl<'l> From<DocError<'l>> for EngineError<'l> {
    fn from(e: DocError<'l>) -> EngineError<'l> {
        EngineError::DocExec(e)
    }
}
