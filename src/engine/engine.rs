#![allow(clippy::module_inception)]

use super::doc::Doc;
use super::doc_set::{DocLabel, DocSet};
use super::Settings;
use crate::language::{Language, LanguageError, LanguageSpec, NotationSetSpec, Storage};
use crate::pretty_doc::{DocRef, PrettyDocError};
use crate::style::Style;
use crate::tree::{Bookmark, Node};
use crate::util::SynlessBug;
use partial_pretty_printer as ppp;
use partial_pretty_printer::pane;
use std::path::Path;

// TODO: think about error types
#[derive(thiserror::Error, Debug)]
pub enum EngineError {
    #[error("Did not find doc named '{0}'")]
    DocNotFound(String),
    #[error("{0}")]
    PrintingError(#[from] ppp::PrintingError<PrettyDocError>),
    #[error("{0}")]
    LanguageError(#[from] LanguageError),
}

#[derive(Debug)]
pub struct Engine {
    storage: Storage,
    doc_set: DocSet,
    clipboard: Vec<Node>,
    settings: Settings,
}

impl Engine {
    pub fn new(settings: Settings) -> Engine {
        todo!()
    }

    /*************
     * Languages *
     *************/

    pub fn add_language(&mut self, language_spec: LanguageSpec) -> Result<(), EngineError> {
        self.storage.add_language(language_spec)?;
        Ok(())
    }

    pub fn set_display_notation(
        &mut self,
        language_name: &str,
        notation_set: NotationSetSpec,
    ) -> Result<(), EngineError> {
        let notation_set_name = notation_set.name.clone();
        let lang = self.storage.language(language_name)?;
        lang.add_notation(&mut self.storage, notation_set)?;
        lang.set_display_notation(&mut self.storage, &notation_set_name)?;
        Ok(())
    }

    pub fn set_source_notation(
        &mut self,
        language_name: &str,
        notation_set: NotationSetSpec,
    ) -> Result<(), EngineError> {
        let notation_set_name = notation_set.name.clone();
        let lang = self.storage.language(language_name)?;
        lang.add_notation(&mut self.storage, notation_set)?;
        lang.set_source_notation(&mut self.storage, &notation_set_name)?;
        Ok(())
    }

    pub fn unset_source_notation(&mut self, language_name: &str) -> Result<(), EngineError> {
        let lang = self.storage.language(language_name)?;
        lang.unset_source_notation(&mut self.storage)?;
        Ok(())
    }

    /******************
     * Doc Management *
     ******************/

    pub fn make_empty_doc(&mut self, doc_name: &str, language: Language) {
        todo!()
    }

    /************
     * Printing *
     ************/

    fn print_source(&self, doc_path: &Path) -> Result<String, EngineError> {
        let doc = self
            .doc_set
            .file_doc(doc_path)
            .ok_or_else(|| EngineError::DocNotFound(doc_path.to_string_lossy().into_owned()))?;
        let doc_ref = doc.doc_ref_source(&self.storage);
        let source = ppp::pretty_print_to_string(doc_ref, self.settings.source_width)?;
        Ok(source)
    }

    fn get_content(&self, label: DocLabel) -> Option<(DocRef, pane::PrintingOptions)> {
        self.doc_set
            .get_content(&self.storage, label, &self.settings)
    }
}
