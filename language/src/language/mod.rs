use std::fmt;

mod grammar;
mod language;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstructId(usize);

#[derive(thiserror::Error, fmt::Debug)]
pub enum LanguageError {
    #[error("Missing notation for construct '{0}'")]
    MissingNotation(String),
    #[error("Duplicate key '{0}' used for both construct '{1}' and construct '{2}")]
    DuplicateKey(char, String, String),
}

pub use self::language::{Language, LanguageSet, LanguageStorage, NotationSet};
pub use grammar::{Arity, AritySpec, Construct, ConstructSpec, Grammar, Sort};
