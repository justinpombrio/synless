use super::stack::Prog;
use super::EditorError;
use crate::engine::{DocDisplayLabel, Engine};
use crate::frontends::{Event, Frontend, Key};
use crate::style::Style;
use partial_pretty_printer as ppp;
use partial_pretty_printer::pane;
use std::error::Error;
use std::time::Duration;

pub struct Runtime<F: Frontend<Style = Style>> {
    pub engine: Engine,
    pane_notation: pane::PaneNotation<DocDisplayLabel, Style>,
    frontend: F,
}

impl<F: Frontend<Style = Style>> Runtime<F> {
    /// Block until the next input event.
    pub fn next_event(&mut self) -> Result<Event, EditorError> {
        loop {
            match self.frontend.next_event(Duration::from_secs(1)) {
                Ok(None) => (), // continue waiting
                Ok(Some(event)) => return Ok(event),
                Err(err) => return Err(EditorError::FrontendError(Box::new(err))),
            }
        }
    }

    pub fn display(&mut self) -> Result<(), EditorError> {
        self.frontend
            .start_frame()
            .map_err(|err| EditorError::FrontendError(Box::new(err)))?;

        let get_content = |doc_label| self.engine.get_content(doc_label);
        pane::display_pane(&mut self.frontend, &self.pane_notation, &get_content)
            .map_err(|err| EditorError::PaneError(Box::new(err)))?;

        self.frontend
            .end_frame()
            .map_err(|err| EditorError::FrontendError(Box::new(err)))
    }

    pub fn lookup_key(&self, key: Key) -> Option<Prog> {
        // TODO: must handle char insertion in text mode
        todo!()
    }
}
