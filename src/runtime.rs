use crate::engine::{DocDisplayLabel, DocName, Engine, Settings};
use crate::frontends::{Event, Frontend, Key, MouseEvent};
use crate::keymap::{KeyProg, Keymap, Layer, LayerManager};
use crate::language::Construct;
use crate::style::Style;
use crate::tree::Mode;
use crate::util::{error, log, SynlessBug, SynlessError};
use partial_pretty_printer as ppp;
use partial_pretty_printer::pane;
use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;
use std::time::Duration;

// TODO: Rename Runtime -> Editor, put it in src/editor.rs?

const KEYHINTS_DOC_NAME: &str = "keyhints";
const CANDIDATE_SELECTION_DOC_NAME: &str = "selection_menu";

pub struct Runtime<F: Frontend<Style = Style>> {
    engine: Engine,
    pane_notation: pane::PaneNotation<DocDisplayLabel, Style>,
    frontend: F,
    layers: LayerManager,
}

impl<F: Frontend<Style = Style> + 'static> Runtime<F> {
    pub fn new(settings: Settings, frontend: F) -> Runtime<F> {
        use crate::style::{Base16Color, Priority};

        let mut engine = Engine::new(settings);

        // Magic initialization
        engine.add_parser("json", crate::parsing::JsonParser);

        let pane_notation = pane::PaneNotation::Vert(vec![
            (
                pane::PaneSize::Proportional(1),
                pane::PaneNotation::Doc {
                    label: DocDisplayLabel::Visible,
                },
            ),
            (
                pane::PaneSize::Fixed(1),
                pane::PaneNotation::Fill {
                    ch: ' ',
                    style: Style::default().with_bg(Base16Color::Base04, Priority::Low),
                },
            ),
            (
                pane::PaneSize::Dynamic,
                pane::PaneNotation::Doc {
                    label: DocDisplayLabel::Auxilliary(CANDIDATE_SELECTION_DOC_NAME.to_owned()),
                },
            ),
            (
                pane::PaneSize::Dynamic,
                pane::PaneNotation::Doc {
                    label: DocDisplayLabel::Auxilliary(KEYHINTS_DOC_NAME.to_owned()),
                },
            ),
        ]);
        Runtime {
            engine,
            pane_notation,
            frontend,
            layers: LayerManager::new(),
        }
    }

    /***********
     * Keymaps *
     ***********/

    pub fn register_layer(&mut self, layer: Layer) {
        self.layers.register_layer(layer);
    }

    pub fn add_global_layer(&mut self, layer_name: &str) -> Result<(), SynlessError> {
        self.layers.add_global_layer(layer_name)
    }

    pub fn remove_global_layer(&mut self, layer_name: &str) -> Result<(), SynlessError> {
        self.layers.remove_global_layer(layer_name)
    }

    pub fn open_menu(&mut self, menu_name: String) -> Result<(), SynlessError> {
        let doc_name = self.engine.visible_doc_name();
        self.layers.open_menu(doc_name, menu_name, None)
    }

    pub fn open_menu_with_keymap(
        &mut self,
        menu_name: String,
        keymap: Keymap,
    ) -> Result<(), SynlessError> {
        let doc_name = self.engine.visible_doc_name();
        self.layers.open_menu(doc_name, menu_name, Some(keymap))
    }

    pub fn close_menu(&mut self) {
        self.layers.close_menu();
    }

    /****************
     * Control Flow *
     ****************/

    pub fn prepare_to_abort(&mut self) {
        log!(Error, "Synless is aborting!");
        // TODO try to save docs
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

    /***********
     * Display *
     ***********/

    pub fn display(&mut self) -> Result<(), SynlessError> {
        self.update_auxilliary_docs();

        self.frontend
            .start_frame()
            .map_err(|err| error!(Frontend, "{}", err))?;

        let get_content = |doc_label| self.engine.get_content(doc_label);
        pane::display_pane(&mut self.frontend, &self.pane_notation, &get_content)?;

        self.frontend
            .end_frame()
            .map_err(|err| error!(Frontend, "{}", err))
    }

    fn update_auxilliary_docs(&mut self) {
        // Candidate Selection
        let selection_doc_name = DocName::Auxilliary(CANDIDATE_SELECTION_DOC_NAME.to_owned());
        self.engine.delete_doc(&selection_doc_name);
        let storage = self.engine.raw_storage_mut();
        if let Some(node) = self.layers.make_candidate_selection_doc(storage) {
            self.engine.add_doc(node, &selection_doc_name).bug();
        };

        // Keyhints
        let keyhints_doc_name = DocName::Auxilliary(KEYHINTS_DOC_NAME.to_owned());
        self.engine.delete_doc(&keyhints_doc_name);
        let visible_doc_name = self.engine.visible_doc_name().cloned();
        let mode = if let Some(doc) = self.engine.visible_doc() {
            doc.mode()
        } else {
            Mode::Tree
        };
        let storage = self.engine.raw_storage_mut();
        if let Some(node) = self
            .layers
            .make_keyhint_doc(storage, mode, visible_doc_name.as_ref())
        {
            self.engine.add_doc(node, &keyhints_doc_name).bug();
        };
    }

    /******************
     * Doc Management *
     ******************/

    /// If there is a visible doc, return the directory it's in. Fall back to the cwd.
    pub fn current_dir(&self) -> Result<String, SynlessError> {
        use std::path::Path;

        fn path_to_string(path: &Path) -> Result<String, SynlessError> {
            path.to_str().map(|s| s.to_owned()).ok_or_else(|| {
                error!(
                    FileSystem,
                    "Path '{}' is not valid Unicode",
                    path.to_string_lossy()
                )
            })
        }

        if let Some(DocName::File(path)) = self.engine.visible_doc_name() {
            if let Some(parent_path) = path.parent() {
                return path_to_string(parent_path);
            }
        }

        let cwd = std::env::current_dir().map_err(|err| {
            error!(
                FileSystem,
                "Failed to get current working directory ({err})"
            )
        })?;
        path_to_string(&cwd)
    }

    pub fn open_doc(&mut self, path: &str) -> Result<(), SynlessError> {
        use std::fs::read_to_string;
        use std::path::PathBuf;

        let source = read_to_string(path)
            .map_err(|err| error!(FileSystem, "Failed to read file at '{path}' ({err})"))?;
        let path_buf = PathBuf::from(path);
        let ext = path_buf
            .extension()
            .ok_or_else(|| {
                error!(
                    Doc,
                    "Can't open file at '{path}' because it doesn't have an extension"
                )
            })?
            .to_str()
            .ok_or_else(|| {
                error!(
                    Doc,
                    "Can't open file at '{path}' because its extension is not valid Unicode"
                )
            })?;
        let language_name = self
            .engine
            .lookup_file_extension(&format!(".{ext}"))
            .ok_or_else(|| error!(Doc, "No language registered for extension '{ext}'"))?
            .to_owned();
        let doc_name = DocName::File(path_buf);
        self.engine
            .load_doc_from_source(doc_name.clone(), &language_name, &source)?;
        self.engine.set_visible_doc(&doc_name)
    }

    /*************
     * Languages *
     *************/

    pub fn load_language(&mut self, path: &str) -> Result<String, SynlessError> {
        use std::fs::read_to_string;
        use std::path::Path;

        let ron_string = read_to_string(path)
            .map_err(|err| error!(FileSystem, "Failed to read file at '{path}' ({err})"))?;
        self.engine.load_language_ron(Path::new(path), &ron_string)
    }

    /***********
     * Editing *
     ***********/

    /***********
     * Private *
     ***********/

    fn lookup_key(&mut self, key: Key) -> Option<KeyProg> {
        let (mode, doc_name) = {
            if let Some(doc_name) = self.engine.visible_doc_name() {
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

/**************
 * Filesystem *
 **************/

fn list_files_and_dirs(dir: &str) -> Result<rhai::Map, SynlessError> {
    use std::fs::read_dir;
    use std::path::{Path, PathBuf};

    let entries = read_dir(dir).map_err(|err| {
        error!(
            FileSystem,
            "Failed to list files in directory '{dir}' ({err})"
        )
    })?;

    let mut files = Vec::new();
    let mut dirs = Vec::new();
    for entry in entries {
        if let Ok(path) = entry.and_then(|e| e.path().canonicalize()) {
            if let Some(path_string) = path.to_str().map(|s| s.to_owned()) {
                if path.is_dir() {
                    dirs.push(path_string);
                } else if path.is_file() {
                    files.push(path_string);
                }
            }
        }
    }

    let mut map = rhai::Map::new();
    map.insert("files".into(), files.into());
    map.insert("dirs".into(), dirs.into());
    Ok(map)
}

macro_rules! register {
    ($module:expr, $runtime:ident . $method:ident($( $param:ident : $type:ty ),*)) => {
        register!($module, $runtime . $method($( $param : $type ),*) as $method)
    };
    ($module:expr, $runtime:ident . $method:ident($( $param:ident : $type:ty ),*) as $name:ident) => {
        let rt = $runtime.clone();
        let closure = move | $( $param : $type ),* | {
            rt.borrow_mut().$method( $( $param ),* )
        };
        rhai::FuncRegistration::new(stringify!($name))
            .in_internal_namespace()
            .set_into_module($module, closure);
    };
    ($module:expr, $runtime:ident . $method:ident($( $param:ident : $type:ty ),*) ?) => {
        register!($module, $runtime . $method($( $param : $type ),*)? as $method)
    };
    ($module:expr, $runtime:ident . $method:ident($( $param:ident : $type:ty ),*) ? as $name:ident) => {
        let rt = $runtime.clone();
        let closure = move | $( $param : $type ),* | {
            rt.borrow_mut().$method( $( $param ),* )
                .map_err(|err| Box::<rhai::EvalAltResult>::from(err))
        };
        rhai::FuncRegistration::new(stringify!($name))
            .in_internal_namespace()
            .set_into_module($module, closure);
    };
    ($module:expr, $function:ident) => {
        register!($module, $function as $function)
    };
    ($module:expr, $function:ident as $name:ident) => {
        rhai::FuncRegistration::new(stringify!($name))
            .in_internal_namespace()
            .set_into_module($module, $function);
    };
    ($module:expr, $function:ident($( $param:ident : $type:ty ),*) ?) => {
        register!($module, $function($( $param: $type),*) ? as $function)
    };
    ($module:expr, $function:ident($( $param:ident : $type:ty ),*) ? as $name:ident) => {
        let closure = move | $( $param : $type ),* | {
            $function( $( $param ),* )
                .map_err(|err| Box::<rhai::EvalAltResult>::from(err))
        };
        rhai::FuncRegistration::new(stringify!($name))
            .in_internal_namespace()
            .set_into_module($module, closure);
    };
}

impl<F: Frontend<Style = Style> + 'static> Runtime<F> {
    pub fn register_internal_methods(rt: Rc<RefCell<Runtime<F>>>, module: &mut rhai::Module) {
        // Control Flow
        register!(module, rt.prepare_to_abort());
        register!(module, rt.block_on_key()?);

        // Display
        register!(module, rt.display()?);
    }

    pub fn register_external_methods(rt: Rc<RefCell<Runtime<F>>>, module: &mut rhai::Module) {
        // Keymaps
        register!(module, rt.register_layer(layer: Layer));
        register!(module, rt.add_global_layer(layer_name: &str)?);
        register!(module, rt.remove_global_layer(layer_name: &str)?);
        register!(module, rt.open_menu(menu_name: String)?);
        register!(module, rt.open_menu_with_keymap(menu_name: String, keymap: Keymap)? as open_menu);
        register!(module, rt.close_menu());

        // Filesystem
        register!(module, list_files_and_dirs(dir: &str)?);

        // Doc management
        register!(module, rt.current_dir()?);
        register!(module, rt.open_doc(path: &str)?);

        // Languages
        register!(module, rt.load_language(path: &str)?);

        // Logging
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
