use super::interpreter::Interpreter;
use super::runtime::Runtime;
use super::stack::{CallStack, DataStack, Op, Prog};
use super::EditorError;
use crate::engine::Engine;
use crate::frontends::{Event, Frontend, Key, MouseEvent};
use crate::style::Style;
use crate::util::SynlessBug;
use partial_pretty_printer::pane;
use std::time::Duration;

/*
 * ## Control flow
 *
 * To execute a program in the interpreter (w/ call stack, data stack, engine):
 *
 * ```
 * PUSH the program onto the call stack
 * REPEAT pop op off call stack and execute it
 * IF the op is `block` or the call stack becomes empty:
 *     THEN suspend the interpreter and return to the server.
 * ```
 *
 * To run the server:
 *
 * ```
 * SET the current keymap:
 *     IF there's a menu
 *     THEN use the menu's keymap
 *     ELSE use the current mode's tree or text keymap
 * DISPLAY the screen
 * BLOCK until a key is pressed
 *     THEN execute that key's prog in the interpreter
 * ```
 */

struct App<F: Frontend<Style = Style>> {
    runtime: Runtime,
    frontend: F,
    interpreter: Interpreter,
}

impl<F: Frontend<Style = Style>> App<F> {
    pub fn run_event_loop(&mut self) {
        loop {
            if let Err(err) = self.display() {
                self.abort(err);
            }
            let prog = match self.block_on_input() {
                Ok(prog) => prog,
                Err(err) => self.abort(err),
            };
            match self.interpreter.execute(&mut self.runtime, prog) {
                Ok(_) => (),
                Err(err) => self.runtime.engine.report_error(&err),
            }
        }
    }

    fn block_on_input(&mut self) -> Result<Prog, EditorError> {
        use std::str::FromStr;

        let ctrl_c = Key::from_str("C-c").bug();

        loop {
            match self.next_event()? {
                // TODO: Remove Ctrl-c. It's only for testing.
                Event::Key(ctrl_c) => return Err(EditorError::KeyboardInterrupt),
                Event::Key(key) => {
                    if let Some(prog) = self.runtime.lookup_key(key) {
                        return Ok(prog);
                    }
                    // wait for a better key press
                }
                Event::Resize => self.display()?,
                Event::Mouse(_) => (),
                Event::Paste(_) => (), // TODO: OS paste support
            }
        }
    }

    /// Block until the next input event.
    fn next_event(&mut self) -> Result<Event, EditorError> {
        loop {
            match self.frontend.next_event(Duration::from_secs(1)) {
                Ok(None) => (), // continue waiting
                Ok(Some(event)) => return Ok(event),
                Err(err) => return Err(EditorError::FrontendError(Box::new(err))),
            }
        }
    }

    fn abort(&mut self, err: EditorError) -> ! {
        self.runtime.engine.report_error(&err);
        // TODO: Save the error log, and maybe the docs
        panic!("{}", err);
    }

    pub fn display(&mut self) -> Result<(), EditorError> {
        self.frontend
            .start_frame()
            .map_err(|err| EditorError::FrontendError(Box::new(err)))?;

        let get_content = |doc_label| self.runtime.engine.get_content(doc_label);
        pane::display_pane(
            &mut self.frontend,
            &self.runtime.pane_notation,
            &get_content,
        )
        .map_err(|err| EditorError::PaneError(Box::new(err)))?;

        self.frontend
            .end_frame()
            .map_err(|err| EditorError::FrontendError(Box::new(err)))
    }
}
