use super::doc::Doc;
use super::Settings;
use crate::language::Storage;
use crate::pretty_doc::DocRef;
use crate::util::bug_assert;
use partial_pretty_printer as ppp;
use partial_pretty_printer::pane;
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

/// Label for documents that might be displayed on the screen.  Not every document will have such a
/// label, and multiple labels may refer to the same document.
///
/// Sample PaneNotation, and its corresponding DocDisplayLabels:
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
pub enum DocDisplayLabel {
    /// The "real" document that the user is viewing and editing.
    Visible,
    /// An auto-generated doc containing info about the `Visible` doc, for use in a status bar.
    Metadata(String),
    /// An auto-generated doc used to implement UI elements like menus.
    Auxilliary(String),
}

/// A unique name for a document.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DocName {
    File(PathBuf),
    Metadata(String),
    Auxilliary(String),
}

impl fmt::Display for DocName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use DocName::*;

        match self {
            File(path) => write!(f, "{}", path.to_string_lossy()),
            Metadata(name) => write!(f, "metadata:{}", name),
            Auxilliary(name) => write!(f, "auxilliary:{}", name),
        }
    }
}

/// Abstract counter saying when a doc was last viewed.
type Timestamp = u64;

#[derive(Debug)]
pub struct DocSet {
    // TODO: consider more efficient ways to store docs in DocSet
    /// Timestamp is when the Doc last became visible (bigger is later)
    docs: HashMap<DocName, (Doc, Timestamp)>,
    visible_doc: Option<DocName>,
    next_timestamp: Timestamp,
}

impl DocSet {
    pub fn new() -> DocSet {
        DocSet {
            docs: HashMap::new(),
            visible_doc: None,
            next_timestamp: 1,
        }
    }

    #[must_use]
    pub fn add_doc(&mut self, doc_name: DocName, doc: Doc) -> bool {
        if self.docs.contains_key(&doc_name) {
            return false;
        }
        self.docs.insert(doc_name, (doc, 0));
        true
    }

    #[must_use]
    pub fn delete_doc(&mut self, doc_name: &DocName) -> bool {
        let deleted = self.docs.remove(doc_name).is_some();
        if deleted && self.visible_doc.as_ref() == Some(doc_name) {
            if let Some(most_recent_doc_path) = self.doc_switching_candidates().first() {
                bug_assert!(
                    self.set_visible_doc(&DocName::File(PathBuf::from(most_recent_doc_path)))
                );
            } else {
                self.visible_doc = None;
            }
        }
        deleted
    }

    #[must_use]
    pub fn set_visible_doc(&mut self, doc_name: &DocName) -> bool {
        if let Some((_, timestamp)) = self.docs.get_mut(doc_name) {
            *timestamp = self.next_timestamp;
            self.next_timestamp += 1;
            self.visible_doc = Some(doc_name.to_owned());
            true
        } else {
            false
        }
    }

    pub fn visible_doc_name(&self) -> Option<&DocName> {
        self.visible_doc.as_ref()
    }

    pub fn visible_doc(&self) -> Option<&Doc> {
        self.docs
            .get(self.visible_doc.as_ref()?)
            .map(|(doc, _)| doc)
    }

    pub fn visible_doc_mut(&mut self) -> Option<&mut Doc> {
        self.docs
            .get_mut(self.visible_doc.as_ref()?)
            .map(|(doc, _)| doc)
    }

    pub fn contains_doc(&self, doc_name: &DocName) -> bool {
        self.docs.contains_key(doc_name)
    }

    pub fn get_doc(&self, doc_name: &DocName) -> Option<&Doc> {
        self.docs.get(doc_name).map(|(doc, _)| doc)
    }

    pub fn get_doc_mut(&mut self, doc_name: &DocName) -> Option<&mut Doc> {
        self.docs.get_mut(doc_name).map(|(doc, _)| doc)
    }

    /// Docs that can become the visible doc. Excludes the current visible doc, and sorts by most
    /// recently visible.
    pub fn doc_switching_candidates(&self) -> Vec<&Path> {
        let mut names_and_timestamps = self
            .docs
            .iter()
            .map(|(name, (_, ts))| (name, *ts))
            .collect::<Vec<_>>();
        names_and_timestamps.sort_by_key(|(_, ts)| -(*ts as i64));
        names_and_timestamps
            .into_iter()
            .filter_map(|(name, _)| {
                if Some(name) == self.visible_doc_name() {
                    return None;
                }
                match name {
                    DocName::File(path) => Some(path.as_ref()),
                    DocName::Metadata(_) | DocName::Auxilliary(_) => None,
                }
            })
            .collect::<Vec<_>>()
    }

    pub fn get_content<'s>(
        &self,
        s: &'s Storage,
        label: DocDisplayLabel,
        settings: &Settings,
    ) -> Option<(DocRef<'s>, pane::PrintingOptions)> {
        let meta_and_aux_options = pane::PrintingOptions {
            focus_path: vec![],
            focus_target: ppp::FocusTarget::Start,
            focus_height: 0.0,
            width_strategy: pane::WidthStrategy::Full,
            set_focus: false,
        };

        let (doc, opts, highlight_cursor) = match label {
            DocDisplayLabel::Visible => {
                let doc = self.get_doc(self.visible_doc_name()?)?;
                let (focus_path, focus_target) = doc.cursor().path_from_root(s);
                let options = pane::PrintingOptions {
                    focus_path,
                    focus_target,
                    focus_height: settings.focus_height,
                    width_strategy: pane::WidthStrategy::NoMoreThan(settings.max_display_width),
                    set_focus: doc.cursor().node(s).is_none(),
                };
                (doc, options, true)
            }
            DocDisplayLabel::Metadata(name) => {
                let doc = self.get_doc(&DocName::Metadata(name))?;
                (doc, meta_and_aux_options, false)
            }
            DocDisplayLabel::Auxilliary(name) => {
                let doc = self.get_doc(&DocName::Auxilliary(name))?;
                (doc, meta_and_aux_options, false)
            }
        };
        Some((doc.doc_ref_display(s, highlight_cursor), opts))
    }
}
