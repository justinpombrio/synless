#![allow(clippy::module_inception)]

use super::command::Command;
use super::doc::Doc;
use super::doc_set::{DocDisplayLabel, DocName, DocSet};
use super::Settings;
use crate::language::{Language, LanguageSpec, NotationSetSpec, Storage};
use crate::parsing::{self, Parse, ParseError};
use crate::pretty_doc::DocRef;
use crate::style::Base16Color;
use crate::tree::{Mode, Node};
use crate::util::{bug, error, SynlessBug, SynlessError};
use partial_pretty_printer as ppp;
use partial_pretty_printer::pane;
use std::collections::HashMap;
use std::path::Path;

const STRING_LANGUAGE_NAME: &str = "string";

#[derive(thiserror::Error, Debug)]
pub enum DocError {
    #[error("Did not find doc named '{0}'")]
    DocNotFound(DocName),
    #[error("Document '{0}' is already open")]
    DocAlreadyOpen(DocName),
    #[error("There is no visible doc to act on")]
    NoVisibleDoc,
    #[error("Can't create document because it doesn't have a valid root node")]
    InvalidRootNode,
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

    pub fn get_language(&self, name: &str) -> Result<Language, SynlessError> {
        Ok(self.storage.language(name)?)
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

    pub fn add_parser(&mut self, language_name: &str, parser: impl Parse + 'static) {
        self.parsers
            .insert(language_name.to_owned(), Box::new(parser));
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
        let doc = Doc::new(&self.storage, root_node, false).bug_msg("Invalid root");
        if !self.doc_set.add_doc(doc_name.to_owned(), doc) {
            Err(DocError::DocAlreadyOpen(doc_name.to_owned()))?;
        }
        Ok(())
    }

    pub fn add_doc(
        &mut self,
        doc_name: &DocName,
        root_node: Node,
        is_saved: bool,
    ) -> Result<(), SynlessError> {
        let doc = Doc::new(&self.storage, root_node, is_saved).ok_or(DocError::InvalidRootNode)?;
        if !self.doc_set.add_doc(doc_name.to_owned(), doc) {
            Err(DocError::DocAlreadyOpen(doc_name.to_owned()))?;
        }
        Ok(())
    }

    pub fn delete_doc(&mut self, doc_name: &DocName) -> Result<(), SynlessError> {
        if self.doc_set.delete_doc(doc_name) {
            Err(DocError::DocNotFound(doc_name.to_owned()))?;
        }
        Ok(())
    }

    pub fn visible_doc_name(&self) -> Option<&DocName> {
        self.doc_set.visible_doc_name()
    }

    pub fn visible_doc(&self) -> Option<&Doc> {
        self.doc_set.visible_doc()
    }

    pub fn set_visible_doc(&mut self, doc_name: &DocName) -> Result<(), SynlessError> {
        if self.doc_set.set_visible_doc(doc_name) {
            Ok(())
        } else {
            Err(DocError::DocNotFound(doc_name.to_owned()).into())
        }
    }

    pub fn close_visible_doc(&mut self) -> Result<(), SynlessError> {
        if let Some(doc_name) = self.doc_set.visible_doc_name().cloned() {
            if self.doc_set.delete_doc(&doc_name) {
                Ok(())
            } else {
                bug!("close_visible_doc: doc '{}' not found", doc_name)
            }
        } else {
            Err(DocError::NoVisibleDoc.into())
        }
    }

    pub fn has_unsaved_changes(&self) -> bool {
        self.doc_set
            .visible_doc()
            .map(|doc| doc.has_unsaved_changes())
            .unwrap_or(false)
    }

    pub fn mark_doc_as_saved(&mut self, doc_name: &DocName) -> Result<(), SynlessError> {
        if let Some(doc) = self.doc_set.get_doc_mut(doc_name) {
            doc.mark_as_saved();
            Ok(())
        } else {
            Err(DocError::DocNotFound(doc_name.to_owned()).into())
        }
    }

    pub fn get_doc(&self, doc_name: &DocName) -> Option<&Doc> {
        self.doc_set.get_doc(doc_name)
    }

    pub fn get_doc_mut(&mut self, doc_name: &DocName) -> Option<&mut Doc> {
        self.doc_set.get_doc_mut(doc_name)
    }

    /// Docs that can become the visible doc. Excludes the current visible doc, and sorts by most
    /// recently visible.
    pub fn doc_switching_candidates(&self) -> Vec<&Path> {
        self.doc_set.doc_switching_candidates()
    }

    pub fn mode(&self) -> Mode {
        self.doc_set
            .visible_doc()
            .map(|doc| doc.mode())
            .unwrap_or(Mode::Tree)
    }

    /****************************
     * Doc Loading and Printing *
     ****************************/

    pub fn load_doc_from_sexpr(
        &self,
        _doc_name: DocName,
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
        let hole_syntax = self
            .storage
            .language(language_name)?
            .hole_syntax(&self.storage)
            .ok_or_else(|| {
                error!(
                    Language,
                    "No hole syntax for language {}, but it's required for loading from source",
                    language_name
                )
            })?
            .to_owned();

        let source = &parsing::preprocess(source, &hole_syntax.invalid, &hole_syntax.valid);
        let root_node = parser.parse(&mut self.storage, &doc_name.to_string(), source)?;
        parsing::postprocess(&mut self.storage, root_node, &hole_syntax.text);

        let doc = Doc::new(&self.storage, root_node, true).bug_msg("Invalid root");
        if !self.doc_set.add_doc(doc_name.clone(), doc) {
            return Err(DocError::DocAlreadyOpen(doc_name).into());
        }
        Ok(())
    }

    pub fn print_source(&self, doc_name: &DocName) -> Result<String, SynlessError> {
        // TODO (optimization): consider returning an iterator of lines for memory efficiency
        let doc = self
            .doc_set
            .get_doc(doc_name)
            .ok_or_else(|| DocError::DocNotFound(doc_name.to_owned()))?;
        let doc_ref = doc.doc_ref_source(&self.storage, false);
        let source = ppp::pretty_print_to_string(doc_ref, self.settings.max_source_width)?;
        Ok(source)
    }

    pub fn get_content(&self, label: DocDisplayLabel) -> Option<(DocRef, pane::PrintingOptions)> {
        self.doc_set
            .get_content(&self.storage, label, &self.settings)
    }

    pub fn make_string_doc(&mut self, string: String, bg_color: Option<Base16Color>) -> Node {
        let lang = self
            .storage
            .language(STRING_LANGUAGE_NAME)
            .bug_msg("Missing String lang");
        let c_root = lang.root_construct(&self.storage);
        let c_string = lang.construct(&self.storage, "String").bug();
        let string_node = Node::with_text(&mut self.storage, c_string, string).bug();
        let node = if let Some(color) = bg_color {
            let c_color = match color {
                Base16Color::Base08 => lang.construct(&self.storage, "BgBase08").bug(),
                Base16Color::Base0B => lang.construct(&self.storage, "BgBase0B").bug(),
                _ => bug!("make_string_doc: specified bg color not yet supported"),
            };
            Node::with_children(&mut self.storage, c_color, [string_node]).bug()
        } else {
            string_node
        };
        Node::with_children(&mut self.storage, c_root, [node]).bug()
    }

    /*************
     * Accessing *
     *************/

    pub fn node_at_cursor(&mut self, deep_copy: bool) -> Result<Node, SynlessError> {
        let doc = self.doc_set.visible_doc().ok_or(DocError::NoVisibleDoc)?;
        let mut node = doc.node_at_cursor(&self.storage)?;
        if deep_copy {
            node = node.deep_copy(&mut self.storage);
        }
        Ok(node)
    }

    /***********
     * Editing *
     ***********/

    pub fn execute(&mut self, cmd: impl Into<Command>) -> Result<(), SynlessError> {
        let doc = self
            .doc_set
            .visible_doc_mut()
            .ok_or(DocError::NoVisibleDoc)?;
        doc.execute(&mut self.storage, cmd.into(), &mut self.clipboard)?;
        Ok(())
    }

    pub fn undo(&mut self) -> Result<(), SynlessError> {
        let doc = self
            .doc_set
            .visible_doc_mut()
            .ok_or(DocError::NoVisibleDoc)?;
        doc.undo(&mut self.storage)?;
        Ok(())
    }

    pub fn redo(&mut self) -> Result<(), SynlessError> {
        let doc = self
            .doc_set
            .visible_doc_mut()
            .ok_or(DocError::NoVisibleDoc)?;
        doc.redo(&mut self.storage)?;
        Ok(())
    }

    pub fn end_undo_group(&mut self) -> Result<(), SynlessError> {
        let doc = self
            .doc_set
            .visible_doc_mut()
            .ok_or(DocError::NoVisibleDoc)?;
        doc.end_undo_group();
        Ok(())
    }

    pub fn revert_undo_group(&mut self) -> Result<(), SynlessError> {
        let doc = self
            .doc_set
            .visible_doc_mut()
            .ok_or(DocError::NoVisibleDoc)?;
        doc.revert_undo_group(&mut self.storage);
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
