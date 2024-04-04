use super::compiled::{compile_language, LanguageCompiled};
use super::interface::Language;
use super::specs::{LanguageSpec, NotationSetSpec};
use super::LanguageError;
use crate::tree::NodeForest;
use crate::util::IndexedMap;

/// Stores all documents and languages.
#[derive(Debug)]
pub struct Storage {
    pub(super) languages: IndexedMap<LanguageCompiled>,
    pub(crate) node_forest: NodeForest,
}

impl Storage {
    pub fn new() -> Storage {
        Storage {
            languages: IndexedMap::new(),
            node_forest: NodeForest::new(),
        }
    }

    pub fn add_language(&mut self, language_spec: LanguageSpec) -> Result<(), LanguageError> {
        let language = compile_language(language_spec)?;
        self.languages
            .insert(language.name.clone(), language)
            .map_err(LanguageError::DuplicateLanguage)
    }

    pub fn language(&self, name: &str) -> Result<Language, LanguageError> {
        let language_id = self
            .languages
            .id(name)
            .ok_or_else(|| LanguageError::UndefinedLanguage(name.to_owned()))?;
        Ok(Language::from_id(language_id))
    }
}

impl Default for Storage {
    fn default() -> Self {
        Storage::new()
    }
}
