mod compiled;
mod interface;
mod specs;
mod storage;

use crate::util::{error, SynlessError};
use partial_pretty_printer as ppp;
use std::fmt;

pub use interface::{Arity, Construct, Language};
pub use specs::{
    AritySpec, ConstructSpec, GrammarSpec, HoleSyntax, LanguageSpec, NotationSetSpec, SortSpec,
};
pub use storage::Storage;

#[derive(thiserror::Error, fmt::Debug)]
pub enum LanguageError {
    // Grammar
    #[error("Duplicate key '{0}' used for both construct '{1}' and construct '{2}")]
    DuplicateKey(char, String, String),
    #[error("Duplicate name '{0}' used for two constructs")]
    DuplicateConstruct(String),
    #[error("Duplicate name '{0}' used for two sorts")]
    DuplicateSort(String),
    #[error("Duplicate name '{0}' used for both a construct and a sort")]
    DuplicateConstructAndSort(String),
    #[error("Name '{0}' is not a known construct or sort")]
    UndefinedConstructOrSort(String),
    #[error("Name '{0}' is not a known construct")]
    UndefinedConstruct(String),
    #[error("Root construct '{0}' must not be texty")]
    TextyRoot(String),
    #[error("Failed to compile regex '{0}' for construct {1}: {2}")]
    InvalidRegex(String, String, String),
    // TODO: Check for cycles
    // #[error("Sort '{0}' refers to itself")]
    // InfiniteSort(String),

    // Notation sets
    #[error("The language '{0}' already has a notation set named '{1}'")]
    DuplicateNotationSet(String, String),
    #[error(
        "Notation set '{0}' gives a notation for '{1}', but there is no construct with that name"
    )]
    UndefinedNotation(String, String),
    #[error("Notation set '{0}' does not give a notation for construct '{1}'")]
    MissingNotation(String, String),
    #[error("Notation set '{0}' gives two notations for construct '{1}'")]
    DuplicateNotation(String, String),
    #[error("Invalid notation for construct '{1}' in notation set '{0}':\n{2}")]
    InvalidNotation(String, String, ppp::NotationError),
    #[error("Invalid notation for holes in language '{0}':\n{1}")]
    InvalidHoleNotation(String, ppp::NotationError),

    // Languages
    #[error("Duplicate name '{0}' used for two languages")]
    DuplicateLanguage(String),
    #[error("Name '{0}' is not a known language")]
    UndefinedLanguage(String),
    #[error("Name '{1}' is not a known notation set for language '{0}'")]
    UndefinedNotationSet(String, String),
}

impl From<LanguageError> for SynlessError {
    fn from(error: LanguageError) -> SynlessError {
        error!(Language, "{}", error.to_string())
    }
}
