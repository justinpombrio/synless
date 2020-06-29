use std::io;

use editor::DocError;
use frontends::{terminal::TermError, Key};
use language::{ConstructName, LanguageName};
use pretty::PaneError;

use crate::engine::DocLabel;
use crate::keymaps::{MenuName, ModeName};

#[derive(thiserror::Error, Debug)]
pub enum ServerError {
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
    Engine(EngineError),
}

impl From<EngineError> for ServerError {
    fn from(e: EngineError) -> ServerError {
        ServerError::Engine(e)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum EngineError {
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
    DocExec(DocError),
}

impl From<DocError> for EngineError {
    fn from(e: DocError) -> EngineError {
        EngineError::DocExec(e)
    }
}
