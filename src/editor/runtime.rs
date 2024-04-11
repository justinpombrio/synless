use super::layer::LayerManager;
use super::stack::Prog;
use super::EditorError;
use crate::engine::{DocDisplayLabel, Engine};
use crate::frontends::{Event, Frontend, Key};
use crate::style::Style;
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
    pub fn lookup_key(&self, key: Key) -> Option<Prog> {
        // TODO: must handle char insertion in text mode
        todo!()
    }
}
