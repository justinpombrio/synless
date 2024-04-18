#![allow(clippy::module_inception)]

use super::command::Command;
use super::doc::Doc;
use super::doc_set::{DocDisplayLabel, DocName, DocSet};
use super::Settings;
use crate::language::{Language, LanguageError, LanguageSpec, NotationSetSpec, Storage};
use crate::parsing::{Parse, ParseError};
use crate::pretty_doc::{DocRef, PrettyDocError};
use crate::tree::Node;
use crate::util::{error, SynlessBug, SynlessError};
use partial_pretty_printer as ppp;
use partial_pretty_printer::pane;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::path::Path;

#[derive(thiserror::Error, Debug)]
pub enum DocError {
    #[error("Did not find doc named '{0}'")]
    DocNotFound(DocName),
    #[error("Document '{0}' is already open")]
    DocAlreadyOpen(DocName),
    #[error("There is no visible doc to act on")]
    NoVisibleDoc,
}

impl From<DocError> for SynlessError {
    fn from(error: DocError) -> SynlessError {
        error!(Doc, "{}", error)
    }
}

#[derive(Debug)]
pub struct Engine {
    storage: Storage,
    doc_set: DocSet,
    parsers: HashMap<String, Box<dyn Parse + 'static>>,
    clipboard: Vec<Node>,
    settings: Settings,
}

impl Engine {
    pub fn new(settings: Settings) -> Engine {
        Engine {
            storage: Storage::new(),
            doc_set: DocSet::new(),
            parsers: HashMap::new(),
            clipboard: Vec::new(),
            settings,
        }
    }

    /******************
     * Error Handling *
     ******************/

    pub fn report_error(&mut self, _error: &impl Error) {
        // make sure to display the actual cause
        todo!()
    }

    /*************
     * Languages *
     *************/

    pub fn load_language_ron(
        &mut self,
        filepath: &Path,
        language_spec_ron: &str,
    ) -> Result<String, SynlessError> {
        let language_spec = ron::from_str::<LanguageSpec>(language_spec_ron)
            .map_err(|err| ParseError::from_ron_error(filepath, err))?;
        let language_name = language_spec.name.clone();
        self.storage.add_language(language_spec)?;
        Ok(language_name)
    }

    pub fn load_notation_ron(
        &mut self,
        language_name: &str,
        filepath: &Path,
        notation_ron: &str,
    ) -> Result<String, SynlessError> {
        let notation_spec = ron::from_str::<NotationSetSpec>(notation_ron)
            .map_err(|err| ParseError::from_ron_error(filepath, err))?;
        let notation_name = notation_spec.name.clone();
        let lang = self.storage.language(language_name)?;
        lang.add_notation(&mut self.storage, notation_spec)?;
        Ok(notation_name)
    }

    pub fn set_display_notation(
        &mut self,
        language_name: &str,
        notation_name: &str,
    ) -> Result<(), SynlessError> {
        let lang = self.storage.language(language_name)?;
        lang.set_display_notation(&mut self.storage, notation_name)?;
        Ok(())
    }

    pub fn set_source_notation(
        &mut self,
        language_name: &str,
        notation_name: &str,
    ) -> Result<(), SynlessError> {
        let lang = self.storage.language(language_name)?;
        lang.set_source_notation(&mut self.storage, notation_name)?;
        Ok(())
    }

    pub fn unset_source_notation(&mut self, language_name: &str) -> Result<(), SynlessError> {
        let lang = self.storage.language(language_name)?;
        lang.unset_source_notation(&mut self.storage)?;
        Ok(())
    }

    pub fn register_file_extension(
        &mut self,
        extension: String,
        language_name: &str,
    ) -> Result<(), SynlessError> {
        let lang = self.storage.language(language_name)?;
        self.storage.register_file_extension(extension, lang);
        Ok(())
    }

    pub fn lookup_file_extension(&self, extension: &str) -> Option<&str> {
        let language = self.storage.lookup_file_extension(extension)?;
        Some(language.name(&self.storage))
    }

    /***********
     * Parsers *
     ***********/

    pub fn add_parser(
        &mut self,
        language_name: &str,
        parser: impl Parse + 'static,
    ) -> Result<(), SynlessError> {
        self.parsers
            .insert(language_name.to_owned(), Box::new(parser));
        Ok(())
    }

    /******************
     * Doc Management *
     ******************/

    pub fn add_empty_doc(
        &mut self,
        doc_name: &DocName,
        language_name: &str,
    ) -> Result<(), SynlessError> {
        let language = self.storage.language(language_name)?;
        let root_construct = language.root_construct(&self.storage);
        let root_node = Node::new(&mut self.storage, root_construct);
        let doc = Doc::new(&self.storage, root_node).bug_msg("Invalid root");
        if !self.doc_set.add_doc(doc_name.to_owned(), doc) {
            Err(DocError::DocAlreadyOpen(doc_name.to_owned()))?;
        }
        Ok(())
    }

    pub fn visible_doc_name(&self) -> Option<&DocName> {
        self.doc_set.visible_doc_name()
    }

    pub fn get_doc(&self, doc_name: &DocName) -> Option<&Doc> {
        self.doc_set.get_doc(doc_name)
    }

    pub fn get_doc_mut(&mut self, doc_name: &DocName) -> Option<&mut Doc> {
        self.doc_set.get_doc_mut(doc_name)
    }

    /****************************
     * Doc Loading and Printing *
     ****************************/

    pub fn load_doc_from_sexpr(
        &self,
        doc_name: DocName,
        _source: &str,
    ) -> Result<(), SynlessError> {
        todo!()
    }

    pub fn print_sexpr(&self, _doc_name: &DocName) -> Result<String, SynlessError> {
        todo!()
    }

    pub fn load_doc_from_source(
        &mut self,
        doc_name: DocName,
        language_name: &str,
        source: &str,
    ) -> Result<(), SynlessError> {
        let parser = self
            .parsers
            .get_mut(language_name)
            .ok_or_else(|| error!(Language, "No parser for language {}", language_name))?;
        let root_node = parser.parse(&mut self.storage, &doc_name.to_string(), source)?;
        let doc = Doc::new(&self.storage, root_node).bug_msg("Invalid root");
        if !self.doc_set.add_doc(doc_name.clone(), doc) {
            return Err(DocError::DocAlreadyOpen(doc_name).into());
        }
        Ok(())
    }

    pub fn set_visible_doc(&mut self, doc_name: &DocName) -> Result<(), SynlessError> {
        if self.doc_set.set_visible_doc(doc_name) {
            Ok(())
        } else {
            Err(DocError::DocNotFound(doc_name.to_owned()).into())
        }
    }

    pub fn print_source(&self, doc_name: &DocName) -> Result<String, SynlessError> {
        let doc = self
            .doc_set
            .get_doc(doc_name)
            .ok_or_else(|| DocError::DocNotFound(doc_name.to_owned()))?;
        let doc_ref = doc.doc_ref_source(&self.storage);
        let source = ppp::pretty_print_to_string(doc_ref, self.settings.max_source_width)?;
        Ok(source)
    }

    pub fn get_content(&self, label: DocDisplayLabel) -> Option<(DocRef, pane::PrintingOptions)> {
        self.doc_set
            .get_content(&self.storage, label, &self.settings)
    }

    /***********
     * Editing *
     ***********/

    pub fn execute(&mut self, cmd: Command) -> Result<(), SynlessError> {
        let doc = self
            .doc_set
            .visible_doc_mut()
            .ok_or(DocError::NoVisibleDoc)?;
        doc.execute(&mut self.storage, cmd, &mut self.clipboard)?;
        Ok(())
    }

    /**********************
     * Raw Storage Access *
     **********************/

    pub fn raw_storage(&self) -> &Storage {
        &self.storage
    }

    pub fn raw_storage_mut(&mut self) -> &mut Storage {
        &mut self.storage
    }
}
