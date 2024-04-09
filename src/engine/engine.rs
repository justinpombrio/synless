#![allow(clippy::module_inception)]

use super::doc::Doc;
use super::doc_set::{DocDisplayLabel, DocName, DocSet};
use super::Settings;
use crate::language::{Language, LanguageError, LanguageSpec, NotationSetSpec, Storage};
use crate::parsing::{Parse, ParseError};
use crate::pretty_doc::{DocRef, PrettyDocError};
use crate::tree::Node;
use partial_pretty_printer as ppp;
use partial_pretty_printer::pane;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

#[derive(thiserror::Error, Debug)]
pub enum EngineError {
    #[error("Did not find doc named '{0:?}'")]
    DocNotFound(DocName),
    #[error("Invalid root node in language '{0}'")]
    InvalidRoot(String),
    #[error("Document '{0:?}' is already open")]
    DocAlreadyOpen(DocName),
    #[error("Printing error")]
    PrintingError(#[from] ppp::PrintingError<PrettyDocError>),
    #[error("Language error")]
    LanguageError(#[from] LanguageError),
    #[error("No parser for language '{0}'")]
    NoParser(String),
    #[error("Parse error in '{doc_name:?}' at '{pos:?}':\n{message}")]
    ParseError {
        doc_name: DocName,
        pos: Option<ppp::Pos>,
        message: String,
    },
}

impl EngineError {
    fn from_ron_error(filename: &Path, error: ron::error::SpannedError) -> EngineError {
        // Serde ron uses 1-indexed positions, with 0,0 as a sentinel value.
        // We use 0-indexed positions.
        let (row, col) = (
            error.position.line as ppp::Row,
            error.position.col as ppp::Col,
        );
        let pos = if row == 0 || col == 0 {
            None
        } else {
            Some(ppp::Pos {
                row: row - 1,
                col: col - 1,
            })
        };
        EngineError::ParseError {
            doc_name: DocName::File(filename.to_owned()),
            pos,
            message: format!("{}", error.code),
        }
    }

    fn from_parse_error(doc_name: &DocName, error: ParseError) -> EngineError {
        EngineError::ParseError {
            doc_name: doc_name.to_owned(),
            pos: error.pos,
            message: error.message,
        }
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
    ) -> Result<String, EngineError> {
        let language_spec = ron::from_str::<LanguageSpec>(language_spec_ron)
            .map_err(|err| EngineError::from_ron_error(filepath, err))?;
        let language_name = language_spec.name.clone();
        self.storage.add_language(language_spec)?;
        Ok(language_name)
    }

    pub fn load_notation_ron(
        &mut self,
        language_name: &str,
        filepath: &Path,
        notation_ron: &str,
    ) -> Result<String, EngineError> {
        let notation_spec = ron::from_str::<NotationSetSpec>(notation_ron)
            .map_err(|err| EngineError::from_ron_error(filepath, err))?;
        let notation_name = notation_spec.name.clone();
        let lang = self.storage.language(language_name)?;
        lang.add_notation(&mut self.storage, notation_spec)?;
        Ok(notation_name)
    }

    pub fn set_display_notation(
        &mut self,
        language_name: &str,
        notation_name: &str,
    ) -> Result<(), EngineError> {
        let lang = self.storage.language(language_name)?;
        lang.set_display_notation(&mut self.storage, notation_name)?;
        Ok(())
    }

    pub fn set_source_notation(
        &mut self,
        language_name: &str,
        notation_name: &str,
    ) -> Result<(), EngineError> {
        let lang = self.storage.language(language_name)?;
        lang.set_source_notation(&mut self.storage, notation_name)?;
        Ok(())
    }

    pub fn unset_source_notation(&mut self, language_name: &str) -> Result<(), EngineError> {
        let lang = self.storage.language(language_name)?;
        lang.unset_source_notation(&mut self.storage)?;
        Ok(())
    }

    /***********
     * Parsers *
     ***********/

    pub fn add_parser(
        &mut self,
        language_name: &str,
        parser: impl Parse + 'static,
    ) -> Result<(), EngineError> {
        self.parsers
            .insert(language_name.to_owned(), Box::new(parser));
        Ok(())
    }

    /******************
     * Doc Management *
     ******************/

    pub fn make_empty_doc(&mut self, _doc_name: &DocName, _language: Language) {
        todo!()
    }

    /****************************
     * Doc Loading and Printing *
     ****************************/

    pub fn load_doc_from_sexpr(
        &self,
        doc_name: &DocName,
        _source: &str,
    ) -> Result<(), EngineError> {
        todo!()
    }

    pub fn print_sexpr(&self, _doc_name: &DocName) -> Result<String, EngineError> {
        todo!()
    }

    pub fn load_doc_from_source(
        &mut self,
        doc_name: &DocName,
        language_name: &str,
        source: &str,
    ) -> Result<(), EngineError> {
        let parser = self
            .parsers
            .get_mut(language_name)
            .ok_or_else(|| EngineError::NoParser(language_name.to_owned()))?;
        let root_node = parser
            .parse(&mut self.storage, source)
            .map_err(|err| EngineError::from_parse_error(doc_name, err))?;
        let doc = Doc::new(&self.storage, root_node)
            .ok_or_else(|| EngineError::InvalidRoot(language_name.to_owned()))?;
        if !self.doc_set.add_doc(doc_name.to_owned(), doc) {
            return Err(EngineError::DocAlreadyOpen(doc_name.to_owned()));
        }
        Ok(())
    }

    pub fn set_visible_doc(&mut self, doc_name: &DocName) -> Result<(), EngineError> {
        if self.doc_set.set_visible_doc(doc_name) {
            Ok(())
        } else {
            Err(EngineError::DocNotFound(doc_name.to_owned()))
        }
    }

    pub fn print_source(&self, doc_name: &DocName) -> Result<String, EngineError> {
        let doc = self
            .doc_set
            .get_doc(doc_name)
            .ok_or_else(|| EngineError::DocNotFound(doc_name.to_owned()))?;
        let doc_ref = doc.doc_ref_source(&self.storage);
        let source = ppp::pretty_print_to_string(doc_ref, self.settings.max_source_width)?;
        Ok(source)
    }

    pub fn get_content(&self, label: DocDisplayLabel) -> Option<(DocRef, pane::PrintingOptions)> {
        self.doc_set
            .get_content(&self.storage, label, &self.settings)
    }
}
