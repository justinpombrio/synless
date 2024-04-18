use super::compiled::{compile_language, LanguageCompiled};
use super::interface::Language;
use super::specs::LanguageSpec;
use super::LanguageError;
use crate::tree::NodeForest;
use crate::util::IndexedMap;
use std::collections::HashMap;

/// Stores all documents and languages.
#[derive(Debug)]
pub struct Storage {
    pub(super) languages: IndexedMap<LanguageCompiled>,
    pub(crate) node_forest: NodeForest,
    /// Map from file extension (including the `.`) to language.
    file_extensions: HashMap<String, Language>,
}

impl Storage {
    pub fn new() -> Storage {
        Storage {
            languages: IndexedMap::new(),
            node_forest: NodeForest::new(),
            file_extensions: HashMap::new(),
        }
    }

    pub fn add_language(&mut self, language_spec: LanguageSpec) -> Result<(), LanguageError> {
        let language = compile_language(language_spec)?;
        let extensions = language.file_extensions.clone();
        let id = self
            .languages
            .insert(language.name.clone(), language)
            .map_err(LanguageError::DuplicateLanguage)?;
        for ext in extensions {
            self.file_extensions.insert(ext, Language::from_id(id));
        }
        Ok(())
    }

    pub fn language(&self, name: &str) -> Result<Language, LanguageError> {
        let language_id = self
            .languages
            .id(name)
            .ok_or_else(|| LanguageError::UndefinedLanguage(name.to_owned()))?;
        Ok(Language::from_id(language_id))
    }

    /// Use the given language to load files with the given extension.
    /// Extensions must include the `.`.
    pub fn register_file_extension(&mut self, extension: String, language: Language) {
        self.file_extensions.insert(extension, language);
    }

    pub fn lookup_file_extension(&self, extension: &str) -> Option<Language> {
        self.file_extensions.get(extension).copied()
    }
}

impl Default for Storage {
    fn default() -> Self {
        Storage::new()
    }
}
