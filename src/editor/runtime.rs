use super::layer::LayerManager;
use super::stack::Prog;
use super::EditorError;
use crate::engine::{DocDisplayLabel, Engine};
use crate::frontends::{Event, Frontend, Key};
use crate::style::Style;
use crate::tree::Mode;
use crate::util::SynlessBug;
use partial_pretty_printer as ppp;
use partial_pretty_printer::pane;
use std::error::Error;
use std::time::Duration;

pub struct Runtime {
    pub engine: Engine,
    pub pane_notation: pane::PaneNotation<DocDisplayLabel, Style>,
    layer_manager: LayerManager,
}

impl Runtime {
    pub fn lookup_key(&mut self, key: Key) -> Option<Prog> {
        let (mode, doc_name) = {
            if let Some(doc_name) = self.engine.visible_doc() {
                let doc = self.engine.get_doc(doc_name).bug();
                (doc.mode(), Some(doc_name))
            } else {
                (Mode::Tree, None)
            }
        };

        self.layer_manager.lookup_key(mode, doc_name, key)
    }
}
