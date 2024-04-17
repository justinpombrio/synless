use crate::engine::{DocDisplayLabel, Engine, Settings};
use crate::frontends::{Event, Frontend, Key, MouseEvent};
use crate::keymap::{KeyProg, LayerManager};
use crate::style::Style;
use crate::tree::Mode;
use crate::util::{error, log, SynlessBug, SynlessError};
use partial_pretty_printer as ppp;
use partial_pretty_printer::pane;
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use std::time::Duration;

// TODO: Rename Runtime -> Editor, put it in src/editor.rs?

pub struct Runtime<F: Frontend<Style = Style>> {
    engine: Engine,
    pane_notation: pane::PaneNotation<DocDisplayLabel, Style>,
    frontend: F,
    layers: LayerManager,
}

impl<F: Frontend<Style = Style> + 'static> Runtime<F> {
    pub fn new(settings: Settings, frontend: F) -> Runtime<F> {
        let engine = Engine::new(settings);
        let pane_notation = pane::PaneNotation::Doc {
            label: DocDisplayLabel::Visible,
        };
        Runtime {
            engine,
            pane_notation,
            frontend,
            layers: LayerManager::new(),
        }
    }

    pub fn close_menu(&mut self) {
        self.layers.close_menu();
    }

    pub fn prepare_to_abort(&mut self) {
        log!(Error, "Synless is aborting!");
        // TODO try to save docs
    }

    pub fn display(&mut self) -> Result<(), SynlessError> {
        self.frontend
            .start_frame()
            .map_err(|err| error!(Frontend, "{}", err))?;

        let get_content = |doc_label| self.engine.get_content(doc_label);
        pane::display_pane(&mut self.frontend, &self.pane_notation, &get_content)?;

        self.frontend
            .end_frame()
            .map_err(|err| error!(Frontend, "{}", err))
    }

    pub fn block_on_key(&mut self) -> Result<KeyProg, SynlessError> {
        use std::str::FromStr;

        let ctrl_c = Key::from_str("C-c").bug();

        loop {
            match self.next_event()? {
                // TODO: Remove Ctrl-c. It's only for testing.
                Event::Key(key) if key == ctrl_c => {
                    return Err(error!(Abort, "I was rudely interrupted by Ctrl-C"));
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

        self.layers.lookup_key(mode, doc_name, key)
    }

    /// Block until the next input event.
    fn next_event(&mut self) -> Result<Event, SynlessError> {
        loop {
            match self.frontend.next_event(Duration::from_secs(1)) {
                Ok(None) => (), // continue waiting
                Ok(Some(event)) => return Ok(event),
                Err(err) => return Err(error!(Frontend, "{}", err)),
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
    ($module:expr, $runtime:ident . $method:ident($( $param:ident : $type:ty ),*) ? ) => {
        let rt = $runtime.clone();
        let closure = move | $( $param : $type ),* | {
            rt.borrow_mut().$method( $( $param ),* )
                .map_err(|err| Box::<rhai::EvalAltResult>::from(err))
        };
        rhai::FuncRegistration::new(stringify!($method))
            .in_internal_namespace()
            .set_into_module($module, closure);
    };
}

impl<F: Frontend<Style = Style> + 'static> Runtime<F> {
    pub fn register_internal_methods(rt: Rc<RefCell<Runtime<F>>>, module: &mut rhai::Module) {
        register!(module, rt.prepare_to_abort());
        register!(module, rt.block_on_key()?);
    }

    pub fn register_external_methods(rt: Rc<RefCell<Runtime<F>>>, module: &mut rhai::Module) {
        register!(module, rt.close_menu());

        rhai::FuncRegistration::new("log_trace")
            .in_internal_namespace()
            .set_into_module(module, |msg: String| log!(Trace, "{}", msg));
        rhai::FuncRegistration::new("log_debug")
            .in_internal_namespace()
            .set_into_module(module, |msg: String| log!(Debug, "{}", msg));
        rhai::FuncRegistration::new("log_info")
            .in_internal_namespace()
            .set_into_module(module, |msg: String| log!(Info, "{}", msg));
        rhai::FuncRegistration::new("log_warn")
            .in_internal_namespace()
            .set_into_module(module, |msg: String| log!(Warn, "{}", msg));
        rhai::FuncRegistration::new("log_error")
            .in_internal_namespace()
            .set_into_module(module, |msg: String| log!(Error, "{}", msg));
    }
}
