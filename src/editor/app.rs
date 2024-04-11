use super::runtime::Runtime;
use super::stack::{CallStack, DataStack, Op, Prog};
use super::EditorError;
use crate::engine::Engine;
use crate::frontends::{Event, Frontend, Key, MouseEvent};
use crate::style::Style;
use crate::util::SynlessBug;
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
    runtime: Runtime<F>,
    call_stack: CallStack,
    data_stack: DataStack,
}

impl<F: Frontend<Style = Style>> App<F> {
    pub fn run_event_loop(&mut self) {
        loop {
            if let Err(err) = self.runtime.display() {
                self.abort(err);
            }
            let prog = match self.block_on_input() {
                Ok(prog) => prog,
                Err(err) => self.abort(err),
            };
            match self.execute(prog) {
                Ok(()) => (),
                Err(err) => self.runtime.engine.report_error(&err),
            }
        }
    }

    fn execute(&mut self, prog: Prog) -> Result<(), EditorError> {
        self.call_stack.push(prog);
        while let Some(op) = self.call_stack.pop() {
            if op == Op::Block {
                return Ok(());
            } else {
                self.call(op)?;
            }
        }
        Ok(())
    }

    fn block_on_input(&mut self) -> Result<Prog, EditorError> {
        use std::str::FromStr;

        let ctrl_c = Key::from_str("C-c").bug();

        loop {
            match self.runtime.next_event()? {
                // TODO: Remove Ctrl-c. It's only for testing.
                Event::Key(ctrl_c) => return Err(EditorError::KeyboardInterrupt),
                Event::Key(key) => {
                    if let Some(prog) = self.runtime.lookup_key(key) {
                        return Ok(prog);
                    }
                    // wait for a better key press
                }
                Event::Resize => self.runtime.display()?,
                Event::Mouse(_) => (),
                Event::Paste(_) => (), // TODO: OS paste support
            }
        }
    }

    fn call(&mut self, op: Op) -> Result<(), EditorError> {
        todo!()
    }

    fn abort(&mut self, err: EditorError) -> ! {
        self.runtime.engine.report_error(&err);
        // TODO: Save the error log, and maybe the docs
        panic!("{}", err);
    }
}
