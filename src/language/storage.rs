use super::forest::Forest;
use super::language_set::{LanguageSet, LanguageSpec, NotationSetSpec};
use super::node::{Node, NodeData, NodeId};
use super::LanguageError;

/// Stores all documents and languages.
pub struct Storage {
    pub(super) language_set: LanguageSet,
    pub(super) forest: Forest<NodeData>,
    pub(super) next_id: NodeId,
}

impl Storage {
    pub fn new() -> Storage {
        Storage {
            language_set: LanguageSet::new(),
            forest: Forest::new(NodeData::invalid_dummy()),
            next_id: NodeId(0),
        }
    }

    pub fn add_language(&mut self, language_spec: LanguageSpec) -> Result<(), LanguageError> {
        self.language_set.add_language(language_spec)
    }

    pub fn add_notation_set(
        &mut self,
        language_name: &str,
        notation_set: NotationSetSpec,
    ) -> Result<(), LanguageError> {
        self.language_set
            .add_notation_set(language_name, notation_set)
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
