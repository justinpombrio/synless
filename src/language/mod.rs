mod forest;
mod language_set;

use std::fmt;

#[derive(thiserror::Error, fmt::Debug)]
pub enum LanguageError {
    #[error("Missing notation for construct '{0}'")]
    MissingNotation(String),
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
    // TODO: Check for cycles
    // #[error("Sort '{0}' refers to itself")]
    // InfiniteSort(String),
}
