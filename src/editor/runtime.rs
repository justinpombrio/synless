use super::SynlessError;
use crate::engine::{DocDisplayLabel, Engine};
use crate::frontends::{Event, Frontend, Key, MouseEvent};
use crate::keymap::{KeyProg, LayerManager};
use crate::style::Style;
use crate::tree::Mode;
use crate::util::SynlessBug;
use partial_pretty_printer as ppp;
use partial_pretty_printer::pane;
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use std::time::Duration;

pub struct Runtime<F: Frontend<Style = Style>> {
    engine: Engine,
    pane_notation: pane::PaneNotation<DocDisplayLabel, Style>,
    frontend: F,
    layer_manager: LayerManager,
}

impl<F: Frontend<Style = Style> + 'static> Runtime<F> {
    pub fn abort(&mut self, err: SynlessError) -> ! {
        //self.engine.report_error(&err);
        // TODO: Save the error log, and maybe the docs
        panic!("{}", err);
    }

    pub fn display(&mut self) -> Result<(), SynlessError> {
        self.frontend
            .start_frame()
            .map_err(|err| SynlessError::new_fatal("Frontend error", err))?;

        let get_content = |doc_label| self.engine.get_content(doc_label);
        pane::display_pane(&mut self.frontend, &self.pane_notation, &get_content)
            .map_err(|err| SynlessError::new_fatal("Pane error", err))?;

        self.frontend
            .end_frame()
            .map_err(|err| SynlessError::new_fatal("Frontend error", err))
    }

    pub fn block_on_key(&mut self) -> Result<KeyProg, SynlessError> {
        use std::str::FromStr;

        let ctrl_c = Key::from_str("C-c").bug();

        #[derive(thiserror::Error, Debug)]
        #[error("I was rudely interrupted by Ctrl-C")]
        struct KeyboardInterruptError;

        loop {
            match self.next_event()? {
                // TODO: Remove Ctrl-c. It's only for testing.
                Event::Key(ctrl_c) => {
                    return Err(SynlessError::new_fatal(
                        "Event error",
                        KeyboardInterruptError,
                    ))
                }
                Event::Key(key) => {
                    if let Some(prog) = self.lookup_key(key) {
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

    fn lookup_key(&mut self, key: Key) -> Option<KeyProg> {
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

    /// Block until the next input event.
    fn next_event(&mut self) -> Result<Event, SynlessError> {
        loop {
            match self.frontend.next_event(Duration::from_secs(1)) {
                Ok(None) => (), // continue waiting
                Ok(Some(event)) => return Ok(event),
                Err(err) => return Err(SynlessError::new_fatal("Frontend error", err)),
            }
        }
    }
}

macro_rules! register {
    ($module:expr, $runtime:ident . $method:ident($( $param:ident : $type:ty ),*) ) => {
        let rt = $runtime.clone();
        let closure = move | $( $param : $type ),* | {
            rt.borrow_mut().$method( $( $param ),* )
        };
        rhai::FuncRegistration::new(stringify!($method))
            .in_internal_namespace()
            .set_into_module($module, closure);
    };
}

impl<F: Frontend<Style = Style> + 'static> Runtime<F> {
    fn register_runtime_methods(self, module: &mut rhai::Module) {
        let rt = Rc::new(RefCell::new(self));

        register!(module, rt.block_on_key());
        register!(module, rt.abort(err: SynlessError));
    }
}
