use super::doc::Doc;
use super::Settings;
use crate::language::Storage;
use crate::pretty_doc::DocRef;
use crate::util::SynlessBug;
use partial_pretty_printer as ppp;
use partial_pretty_printer::pane;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

type DocIndex = usize;

/// Label for documents that might be displayed on the screen.
///
/// Sample PaneNotation, and its corresponding DocLabels:
///
/// ```text
/// +----------------------------+
/// |  doc1 |*doc2*| doc3 |      |
/// +----------------------------+
/// |                            |
/// | This is the visible doc.   |
/// |                            |
/// +----------------------------+
/// | doc2.rs               27,1 |
/// +----------------------------+
/// |i->insert    h->left        |
/// |s->save      l->right       |
/// +----------------------------+
///
/// +----------------------------+
/// |  Aux(tab_bar)              |
/// +----------------------------+
/// |                            |
/// | Visible                    |
/// |                            |
/// +----------------------------+
/// | Meta(name)   Meta(linecol) |
/// +----------------------------+
/// |                            |
/// |  Aux(key_hints)            |
/// +----------------------------+
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DocLabel {
    /// A "real" document that the user is viewing and editing.
    Visible,
    /// An auto-generated doc containing info about the `Visible` doc, for use in a status bar.
    Metadata(String),
    /// An auto-generated doc used to implement UI elements like menus.
    Auxilliary(String),
}

#[derive(Debug)]
pub struct DocSet {
    file_path_to_doc: HashMap<PathBuf, DocIndex>,
    /// INVARIANT: DocLabel::Visible is always present.
    label_to_doc: HashMap<DocLabel, DocIndex>,
    /// DocIndex -> Doc
    docs: Vec<Doc>,
}

impl DocSet {
    pub fn new(starting_doc: Doc) -> DocSet {
        let mut doc_set = DocSet {
            file_path_to_doc: HashMap::new(),
            label_to_doc: HashMap::new(),
            docs: Vec::new(),
        };
        let starting_doc_index = doc_set.insert_doc(starting_doc);
        doc_set
            .label_to_doc
            .insert(DocLabel::Visible, starting_doc_index);
        doc_set
    }

    pub fn visible_doc(&self) -> &Doc {
        let doc_index = *self
            .label_to_doc
            .get(&DocLabel::Visible)
            .bug_msg("VisibleDoc not found");
        self.docs.get(doc_index).bug()
    }

    pub fn metadata_doc(&self, name: &str) -> Option<&Doc> {
        let doc_index = *self
            .label_to_doc
            .get(&DocLabel::Metadata(name.to_owned()))?;
        Some(self.docs.get(doc_index).bug())
    }

    pub fn auxilliary_doc(&self, name: &str) -> Option<&Doc> {
        let doc_index = *self
            .label_to_doc
            .get(&DocLabel::Auxilliary(name.to_owned()))?;
        Some(self.docs.get(doc_index).bug())
    }

    pub fn file_doc(&self, file_path: &Path) -> Option<&Doc> {
        let doc_index = *self.file_path_to_doc.get(file_path)?;
        Some(self.docs.get(doc_index).bug())
    }

    pub fn get_content<'s>(
        &self,
        s: &'s Storage,
        label: DocLabel,
        settings: &Settings,
    ) -> Option<(DocRef<'s>, pane::PrintingOptions)> {
        let meta_and_aux_options = pane::PrintingOptions {
            focus_path: vec![],
            focus_target: ppp::FocusTarget::Start,
            focus_height: 0.0,
            width_strategy: pane::WidthStrategy::Full,
            set_focus: false,
        };

        let (doc, opts) = match label {
            DocLabel::Visible => {
                let doc = self.visible_doc();
                let (focus_path, focus_target) = doc.cursor().path_from_root(s);
                let options = pane::PrintingOptions {
                    focus_path,
                    focus_target,
                    focus_height: settings.focus_height,
                    width_strategy: pane::WidthStrategy::NoMoreThan(settings.max_doc_width),
                    set_focus: true,
                };
                (doc, options)
            }
            DocLabel::Metadata(name) => {
                let doc = self.metadata_doc(&name)?;
                (doc, meta_and_aux_options)
            }
            DocLabel::Auxilliary(name) => {
                let doc = self.auxilliary_doc(&name)?;
                (doc, meta_and_aux_options)
            }
        };
        Some((doc.doc_ref_display(s), opts))
    }

    fn insert_doc(&mut self, doc: Doc) -> usize {
        let doc_index = self.docs.len();
        self.docs.push(doc);
        doc_index
    }
}
