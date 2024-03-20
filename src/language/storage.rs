use super::forest::Forest;
use super::language_set::{
    compile_language, compile_notation_set, Language, LanguageCompiled, LanguageSpec,
    NotationSetSpec,
};
use super::node::{Node, NodeData, NodeId};
use super::LanguageError;
use crate::util::IndexedMap;

/// Stores all documents and languages.
pub struct Storage {
    pub(super) languages: IndexedMap<LanguageCompiled>,
    pub(super) forest: Forest<NodeData>,
    pub(super) next_id: NodeId,
}

impl Storage {
    pub fn new() -> Storage {
        Storage {
            languages: IndexedMap::new(),
            forest: Forest::new(NodeData::invalid_dummy()),
            next_id: NodeId(0),
        }
    }

    pub fn add_language(&mut self, language_spec: LanguageSpec) -> Result<(), LanguageError> {
        let language = compile_language(language_spec)?;
        self.languages
            .insert(language.name.clone(), language)
            .map_err(LanguageError::DuplicateLanguage)
    }

    pub fn add_notation_set(
        &mut self,
        language_name: &str,
        notation_set: NotationSetSpec,
    ) -> Result<(), LanguageError> {
        if let Some(language) = self.languages.get_by_name_mut(language_name) {
            let notation_set = compile_notation_set(notation_set, &language.grammar)?;
            language
                .notation_sets
                .insert(notation_set.name.clone(), notation_set)
                .map_err(|name| LanguageError::DuplicateNotationSet(language_name.to_owned(), name))
        } else {
            Err(LanguageError::UndefinedLanguage(language_name.to_owned()))
        }
    }

    pub fn get_language(&self, name: &str) -> Option<Language> {
        Some(Language::from_id(self.languages.id(name)?))
    }

    pub(super) fn next_id(&mut self) -> NodeId {
        let id = self.next_id.0;
        self.next_id.0 += 1;
        NodeId(id)
    }
}

impl Default for Storage {
    fn default() -> Self {
        Storage::new()
    }
}
