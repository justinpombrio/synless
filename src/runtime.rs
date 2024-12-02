use crate::engine::{
    BookmarkCommand, ClipboardCommand, DocDisplayLabel, DocName, Engine, Settings, TextEdCommand,
    TextNavCommand, TreeEdCommand, TreeNavCommand,
};
use crate::frontends::{Event, Frontend, Key};
use crate::keymap::{KeyLookupResult, KeyProg, Keymap, Layer, LayerManager, MenuSelectionCmd};
use crate::language::{Construct, Language};
use crate::style::Style;
use crate::tree::{Mode, Node};
use crate::util::{error, fs_util, log, LogEntry, LogLevel, SynlessBug, SynlessError};
use partial_pretty_printer::pane;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

// TODO: Rename Runtime -> Editor, put it in src/editor.rs?

const KEYHINTS_DOC_LABEL: &str = "keyhints";
const CANDIDATE_SELECTION_DOC_LABEL: &str = "selection_menu";
const MENU_NAME_LABEL: &str = "menu_name";
const MODE_LABEL: &str = "mode";
const FILENAME_LABEL: &str = "filename";
const SIBLING_INDEX_LABEL: &str = "sibling_index";
const LAST_LOG_LABEL: &str = "last_log";

const KEYHINTS_PANE_WIDTH: usize = 15;

const LOG_LEVEL_TO_DISPLAY: LogLevel = LogLevel::Info;

pub struct Runtime<F: Frontend<Style = Style>> {
    engine: Engine,
    default_pane_notation: pane::PaneNotation<DocDisplayLabel, Style>,
    menu_pane_notation: pane::PaneNotation<DocDisplayLabel, Style>,
    frontend: F,
    layers: LayerManager,
    last_log: Option<LogEntry>,
    cli_args: rhai::Map,
}

impl<F: Frontend<Style = Style> + 'static> Runtime<F> {
    pub fn new(settings: Settings, frontend: F, cli_args: rhai::Map) -> Runtime<F> {
        let mut engine = Engine::new(settings);

        // Magic initialization
        engine.add_parser("json", crate::parsing::JsonParser);

        Runtime {
            engine,
            default_pane_notation: make_pane_notation(false),
            menu_pane_notation: make_pane_notation(true),
            frontend,
            layers: LayerManager::new(),
            last_log: None,
            cli_args,
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

    pub fn open_menu(&mut self, menu: MenuBuilder) -> Result<(), SynlessError> {
        let doc_name = self.engine.visible_doc_name();
        self.layers.open_menu(
            doc_name,
            menu.name,
            menu.description,
            menu.keymap,
            menu.is_candidate_menu,
            menu.default_to_custom_candidate,
        )
    }

    pub fn close_menu(&mut self) {
        self.layers.close_menu();
    }

    pub fn menu_selection_up(&mut self) -> Result<(), SynlessError> {
        self.layers.edit_menu_selection(MenuSelectionCmd::Up)
    }

    pub fn menu_selection_down(&mut self) -> Result<(), SynlessError> {
        self.layers.edit_menu_selection(MenuSelectionCmd::Down)
    }

    pub fn menu_selection_backspace(&mut self) -> Result<(), SynlessError> {
        self.layers.edit_menu_selection(MenuSelectionCmd::Backspace)
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
                    if let Some(prog) = self.handle_key(key)? {
                        return Ok(prog);
                    }
                    // wait for another key press
                }
                Event::Resize => self.display()?,
                Event::Mouse(_) => (),
                Event::Paste(_) => (), // TODO: OS paste support
            }
        }
    }

    /***********
     * Logging *
     ***********/

    pub fn log_error(&mut self, message: String) {
        self.log(LogLevel::Error, message);
    }

    pub fn log_warn(&mut self, message: String) {
        self.log(LogLevel::Warn, message);
    }

    pub fn log_info(&mut self, message: String) {
        self.log(LogLevel::Info, message);
    }

    pub fn log_debug(&mut self, message: String) {
        self.log(LogLevel::Debug, message);
    }

    pub fn log_trace(&mut self, message: String) {
        self.log(LogLevel::Trace, message);
    }

    fn log(&mut self, level: LogLevel, message: String) {
        let entry = LogEntry::new(level, message);
        if level >= LOG_LEVEL_TO_DISPLAY
            && self
                .last_log
                .as_ref()
                .map(|old| level > old.level)
                .unwrap_or(true)
        {
            self.last_log = Some(entry.clone());
        }
        entry.log();
    }

    pub fn clear_last_log(&mut self) {
        self.last_log = None;
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
        let note = if self.layers.has_open_menu() {
            &self.menu_pane_notation
        } else {
            &self.default_pane_notation
        };
        pane::display_pane(&mut self.frontend, note, &Style::default(), &get_content)?;

        self.frontend
            .end_frame()
            .map_err(|err| error!(Frontend, "{}", err))
    }

    fn update_auxilliary_docs(&mut self) {
        for (name, node) in [
            self.make_keyhint_doc(),
            self.make_candidate_selection_doc(),
            self.make_menu_name_doc(),
            self.make_mode_doc(),
            self.make_filename_doc(),
            self.make_sibling_index_doc(),
            self.make_last_log_doc(),
        ] {
            let _ = self.engine.delete_doc(&name);
            if let Some(node) = node {
                // is_saved = true because auxilliary docs should be treated as never having
                // unsaved changes.
                self.engine.add_doc(&name, node, true).bug();
            }
        }
    }

    fn make_candidate_selection_doc(&mut self) -> (DocName, Option<Node>) {
        let storage = self.engine.raw_storage_mut();
        let node = self.layers.make_candidate_selection_doc(storage);
        (
            DocName::Auxilliary(CANDIDATE_SELECTION_DOC_LABEL.to_owned()),
            node,
        )
    }

    fn make_keyhint_doc(&mut self) -> (DocName, Option<Node>) {
        let visible_doc_name = self.engine.visible_doc_name().cloned();
        let mode = self.engine.mode();
        let storage = self.engine.raw_storage_mut();
        let node = self
            .layers
            .make_keyhint_doc(storage, mode, visible_doc_name.as_ref());
        (DocName::Auxilliary(KEYHINTS_DOC_LABEL.to_owned()), node)
    }

    fn make_menu_name_doc(&mut self) -> (DocName, Option<Node>) {
        let opt_node = self
            .layers
            .menu_description()
            .map(|menu_name| self.engine.make_string_doc(menu_name.to_owned(), None));
        (DocName::Auxilliary(MENU_NAME_LABEL.to_owned()), opt_node)
    }

    fn make_mode_doc(&mut self) -> (DocName, Option<Node>) {
        use crate::style::Base16Color;

        let (mode, color) = match self.engine.mode() {
            Mode::Tree => ("[TREE]".to_owned(), None),
            Mode::Text => ("[TEXT]".to_owned(), Some(Base16Color::Base0B)),
        };
        let node = self.engine.make_string_doc(mode, color);
        (DocName::Auxilliary(MODE_LABEL.to_owned()), Some(node))
    }

    fn make_filename_doc(&mut self) -> (DocName, Option<Node>) {
        let opt_doc_name = self.engine.visible_doc_name();
        let opt_label = opt_doc_name.map(|doc_name| match doc_name {
            DocName::File(path) => {
                let os_str = path.file_name().unwrap_or_else(|| path.as_os_str());
                let name = os_str.to_string_lossy().into_owned();
                if self.engine.has_unsaved_changes() {
                    format!("{}*", name)
                } else {
                    name
                }
            }
            DocName::Metadata(label) => format!("metadata:{}", label),
            DocName::Auxilliary(label) => format!("auxilliary:{}", label),
        });
        let opt_node = opt_label.map(|label| self.engine.make_string_doc(label, None));
        (DocName::Auxilliary(FILENAME_LABEL.to_owned()), opt_node)
    }

    fn make_sibling_index_doc(&mut self) -> (DocName, Option<Node>) {
        let opt_label = self.engine.visible_doc().map(|doc| {
            let cursor = doc.cursor();
            let s = self.engine.raw_storage();
            let (numerator, denominator) = cursor.sibling_index_info(s);
            format!("sibling {}/{}", numerator, denominator)
        });
        let opt_node = opt_label.map(|label| self.engine.make_string_doc(label, None));
        (
            DocName::Auxilliary(SIBLING_INDEX_LABEL.to_owned()),
            opt_node,
        )
    }

    fn make_last_log_doc(&mut self) -> (DocName, Option<Node>) {
        let opt_message = self.last_log.as_ref().map(|entry| entry.to_string());
        let opt_node = opt_message.map(|msg| self.engine.make_string_doc(msg, None));
        (DocName::Auxilliary(LAST_LOG_LABEL.to_owned()), opt_node)
    }

    /******************
     * Doc Management *
     ******************/

    /// If there is a visible doc, return the directory it's in. Fall back to the cwd.
    pub fn current_dir(&self) -> Result<String, SynlessError> {
        if let Some(DocName::File(path)) = self.engine.visible_doc_name() {
            if let Some(parent_path) = path.parent() {
                return fs_util::path_to_string(parent_path);
            }
        }

        let cwd = std::env::current_dir().map_err(|err| {
            error!(
                FileSystem,
                "Failed to get current working directory ({err})"
            )
        })?;
        fs_util::path_to_string(&cwd)
    }

    pub fn new_doc(&mut self, path: &str) -> Result<(), SynlessError> {
        use std::path::PathBuf;

        let path_buf = PathBuf::from(path);
        if path_buf.exists() {
            return Err(error!(
                FileSystem,
                "File already exists: {}",
                path_buf.display()
            ));
        }
        let language_name = self.language_name_from_file_extension(&path_buf)?;
        let doc_name = DocName::File(path_buf);
        self.engine.add_empty_doc(&doc_name, &language_name)?;
        self.engine.set_visible_doc(&doc_name)
    }

    pub fn open_doc(&mut self, path: &str) -> Result<(), SynlessError> {
        use std::fs::read_to_string;
        use std::path::PathBuf;

        let source = read_to_string(path)
            .map_err(|err| error!(FileSystem, "Failed to read file at '{path}' ({err})"))?;
        let path_buf = PathBuf::from(path);
        let language_name = self.language_name_from_file_extension(&path_buf)?;
        let doc_name = DocName::File(path_buf);
        self.engine
            .load_doc_from_source(doc_name.clone(), &language_name, &source)?;
        self.engine.set_visible_doc(&doc_name)
    }

    fn language_name_from_file_extension(
        &self,
        path: &std::path::Path,
    ) -> Result<String, SynlessError> {
        let extension = path
            .extension()
            .ok_or_else(|| {
                error!(
                    Doc,
                    "Can't determine language of '{}' because it doesn't have an extension",
                    path.display()
                )
            })?
            .to_str()
            .ok_or_else(|| {
                error!(
                    Doc,
                    "Can't determine language of '{}' because its extension is not valid Unicode",
                    path.display()
                )
            })?;
        Ok(self
            .engine
            .lookup_file_extension(&format!(".{extension}"))
            .ok_or_else(|| error!(Doc, "No language registered for extension '{extension}'"))?
            .to_owned())
    }

    pub fn doc_switching_candidates(&self) -> Result<Vec<rhai::Dynamic>, SynlessError> {
        self.engine
            .doc_switching_candidates()
            .into_iter()
            .map(|path| Ok(rhai::Dynamic::from(fs_util::path_to_string(path)?)))
            .collect::<Result<Vec<_>, _>>()
    }

    pub fn switch_to_doc(&mut self, path: &str) -> Result<(), SynlessError> {
        use std::path::PathBuf;

        self.engine
            .set_visible_doc(&DocName::File(PathBuf::from(path)))
    }

    pub fn has_visible_doc(&self) -> bool {
        self.engine.visible_doc().is_some()
    }

    pub fn has_unsaved_changes(&self) -> bool {
        self.engine.has_unsaved_changes()
    }

    pub fn force_close_visible_doc(&mut self) -> Result<(), SynlessError> {
        self.engine.close_visible_doc()
    }

    pub fn save_doc(&mut self) -> Result<(), SynlessError> {
        self.save_doc_impl(None)
    }

    pub fn save_doc_as(&mut self, path: String) -> Result<(), SynlessError> {
        self.save_doc_impl(Some(path))
    }

    fn save_doc_impl(&mut self, path: Option<String>) -> Result<(), SynlessError> {
        if let Some(doc_name) = self.engine.visible_doc_name().cloned() {
            let source = self.engine.print_source(&doc_name)?;
            let path = if let Some(path) = path {
                path
            } else if let DocName::File(path_buf) = &doc_name {
                fs_util::path_to_string(path_buf)?
            } else {
                return Err(error!(Doc, "Document does not have a path. Try save-as."));
            };
            std::fs::write(&path, source)
                .map_err(|err| error!(FileSystem, "Failed to write to file '{path}' ({err})"))?;
            self.engine.mark_doc_as_saved(&doc_name)
        } else {
            Err(error!(Doc, "No open document"))
        }
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

    pub fn get_language(&mut self, language_name: &str) -> Result<Language, SynlessError> {
        self.engine.get_language(language_name)
    }

    pub fn language_constructs(&mut self, language: Language) -> Vec<rhai::Dynamic> {
        language
            .constructs(self.engine.raw_storage())
            .map(rhai::Dynamic::from)
            .collect()
    }

    pub fn construct_name(&self, construct: Construct) -> String {
        construct.name(self.engine.raw_storage()).to_owned()
    }

    pub fn construct_key(&self, construct: Construct) -> String {
        construct
            .key(self.engine.raw_storage())
            .map(|c| c.to_string())
            .unwrap_or_default()
    }

    /***********
     * Editing *
     ***********/

    pub fn undo(&mut self) -> Result<(), SynlessError> {
        self.engine.undo()
    }

    pub fn redo(&mut self) -> Result<(), SynlessError> {
        self.engine.redo()
    }

    pub fn insert_node(&mut self, construct: Construct) -> Result<(), SynlessError> {
        let node = Node::new_with_auto_fill(self.engine.raw_storage_mut(), construct);
        self.engine.execute(TreeEdCommand::Insert(node))?;
        self.engine.execute(TreeNavCommand::FirstInsertLoc)
    }

    /*************
     * Clipboard *
     *************/

    pub fn cut(&mut self) -> Result<(), SynlessError> {
        self.engine.execute(ClipboardCommand::Copy)?;
        self.engine.execute(TreeEdCommand::Backspace)
    }

    /**************************
     * Command Line Interface *
     **************************/

    pub fn cli_args(&self) -> rhai::Map {
        self.cli_args.clone()
    }

    /***********
     * Private *
     ***********/

    /// If the `key` is bound to a prog that needs to be executed by rhai, then returns `Some(prog)`.
    /// Otherwise (if the `key` is not bound or is bound to something that was already handled),
    /// then returns `None`.
    fn handle_key(&mut self, key: Key) -> Result<Option<KeyProg>, SynlessError> {
        let (mode, doc_name) = {
            if let Some(doc_name) = self.engine.visible_doc_name() {
                let doc = self.engine.get_doc(doc_name).bug();
                (doc.mode(), Some(doc_name))
            } else {
                (Mode::Tree, None)
            }
        };
        match self.layers.lookup_key(mode, doc_name, key) {
            None => Ok(None),
            Some(KeyLookupResult::KeyProg(key_prog)) => {
                // Each keypress in tree mode should be a separate undo group, but multiple text
                // edits (and multiple edits made in a menu) should be grouped together.
                if mode != Mode::Text && !self.layers.has_open_menu() {
                    let _ = self.engine.end_undo_group();
                }
                Ok(Some(key_prog))
            }
            Some(KeyLookupResult::Redisplay) => {
                self.display()?;
                Ok(None)
            }
            Some(KeyLookupResult::InsertChar(ch)) => {
                self.engine.execute(TextEdCommand::Insert(ch))?;
                self.display()?;
                Ok(None)
            }
        }
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

/*********************
 * Menu Construction *
 *********************/

#[derive(Debug, Clone)]
pub struct MenuBuilder {
    name: String,
    description: String,
    keymap: Option<Keymap>,
    is_candidate_menu: bool,
    default_to_custom_candidate: bool,
}

pub fn make_menu(menu_name: String, description: String) -> MenuBuilder {
    MenuBuilder {
        name: menu_name,
        description,
        keymap: None,
        is_candidate_menu: false,
        default_to_custom_candidate: false,
    }
}

pub fn set_menu_is_candidate_menu(menu: &mut MenuBuilder, setting: bool) {
    menu.is_candidate_menu = setting;
}

pub fn set_menu_keymap(menu: &mut MenuBuilder, keymap: Keymap) {
    menu.keymap = Some(keymap);
}

pub fn set_menu_default_to_custom_candidate(menu: &mut MenuBuilder, setting: bool) {
    menu.default_to_custom_candidate = setting;
}

/******************
 * Pane Notations *
 ******************/

fn make_pane_notation(include_menu: bool) -> pane::PaneNotation<DocDisplayLabel, Style> {
    use crate::style::{Base16Color, Priority};
    use pane::{PaneNotation, PaneSize};

    let bar_style = Style::default()
        .with_bg(Base16Color::Base04, Priority::Low)
        .with_fg(Base16Color::Base00, Priority::Low)
        .with_bold(true, Priority::Low);
    let status_bar_style = Style::default()
        .with_bg(Base16Color::Base06, Priority::Low)
        .with_fg(Base16Color::Base00, Priority::Low)
        .with_bold(true, Priority::Low);

    let divider = PaneNotation::Style {
        style: bar_style.clone(),
        notation: Box::new(PaneNotation::Fill { ch: ' ' }),
    };
    let padding = PaneNotation::Fill { ch: ' ' };

    let keyhints_doc = PaneNotation::Doc {
        label: DocDisplayLabel::Auxilliary(KEYHINTS_DOC_LABEL.to_owned()),
    };
    let keyhints = PaneNotation::Vert(vec![
        (PaneSize::Proportional(1), padding.clone()),
        (PaneSize::Dynamic, keyhints_doc),
        (PaneSize::Fixed(1), padding.clone()),
    ]);

    let main_doc = PaneNotation::Doc {
        label: DocDisplayLabel::Visible,
    };
    let menu_doc = PaneNotation::Doc {
        label: DocDisplayLabel::Auxilliary(CANDIDATE_SELECTION_DOC_LABEL.to_owned()),
    };
    let menu_name = PaneNotation::Doc {
        label: DocDisplayLabel::Auxilliary(MENU_NAME_LABEL.to_owned()),
    };
    let menu_bar = PaneNotation::Style {
        style: bar_style.clone(),
        notation: Box::new(PaneNotation::Horz(vec![
            (PaneSize::Dynamic, menu_name),
            (PaneSize::Proportional(1), padding.clone()),
        ])),
    };

    let mode_doc = PaneNotation::Doc {
        label: DocDisplayLabel::Auxilliary(MODE_LABEL.to_owned()),
    };
    let filename_doc = PaneNotation::Doc {
        label: DocDisplayLabel::Auxilliary(FILENAME_LABEL.to_owned()),
    };
    let sibling_index_doc = PaneNotation::Doc {
        label: DocDisplayLabel::Auxilliary(SIBLING_INDEX_LABEL.to_owned()),
    };
    let status_bar = PaneNotation::Style {
        style: status_bar_style,
        notation: Box::new(PaneNotation::Horz(vec![
            (PaneSize::Dynamic, mode_doc),
            (PaneSize::Fixed(1), padding.clone()),
            (PaneSize::Dynamic, filename_doc),
            (PaneSize::Proportional(1), padding.clone()),
            (PaneSize::Dynamic, sibling_index_doc),
            (PaneSize::Fixed(1), padding),
        ])),
    };
    let log_doc = PaneNotation::Doc {
        label: DocDisplayLabel::Auxilliary(LAST_LOG_LABEL.to_owned()),
    };

    let mut main_doc_and_menu = vec![(PaneSize::Proportional(1), main_doc)];
    if include_menu {
        main_doc_and_menu.push((PaneSize::Fixed(1), menu_bar));
        main_doc_and_menu.push((PaneSize::Dynamic, menu_doc));
    }

    PaneNotation::Vert(vec![
        (
            PaneSize::Proportional(1),
            PaneNotation::Horz(vec![
                (
                    PaneSize::Proportional(1),
                    PaneNotation::Vert(main_doc_and_menu),
                ),
                (PaneSize::Fixed(1), divider),
                (PaneSize::Fixed(KEYHINTS_PANE_WIDTH), keyhints),
            ]),
        ),
        (PaneSize::Fixed(1), status_bar),
        (PaneSize::Fixed(1), log_doc),
    ])
}

/***********
 * Keymaps *
 ***********/

fn escape() -> Result<(), SynlessError> {
    Err(error!(Escape, "Escape"))
}

/**************
 * Filesystem *
 **************/

fn list_files_and_dirs(dir: &str) -> Result<rhai::Map, SynlessError> {
    use std::fs::read_dir;

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
    ($module:expr, $runtime:ident, $command:ident :: $variant:ident as $name:ident) => {
        let rt = $runtime.clone();
        let closure = move || {
            rt.borrow_mut().engine.execute($command::$variant)
                .map_err(|err| Box::<rhai::EvalAltResult>::from(err))
        };
        rhai::FuncRegistration::new(stringify!($name))
            .in_internal_namespace()
            .set_into_module($module, closure);
    };
    ($module:expr, $runtime:ident, $command:ident :: $variant:ident ($( $param:ident : $type:ty ),*) as $name:ident) => {
        let rt = $runtime.clone();
        let closure = move | $( $param : $type ),* | {
            rt.borrow_mut().engine.execute($command::$variant( $( $param ),* ))
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
        use fs_util::{join_path, path_file_name};

        // Keymaps
        register!(module, rt.register_layer(layer: Layer));
        register!(module, rt.add_global_layer(layer_name: &str)?);
        register!(module, rt.remove_global_layer(layer_name: &str)?);
        register!(module, make_menu);
        register!(module, set_menu_keymap);
        register!(module, set_menu_default_to_custom_candidate);
        register!(module, set_menu_is_candidate_menu);
        register!(module, rt.open_menu(menu: MenuBuilder)?);
        register!(module, rt.close_menu());
        register!(module, escape()?);
        register!(module, rt.menu_selection_up()?);
        register!(module, rt.menu_selection_down()?);
        register!(module, rt.menu_selection_backspace()?);

        // Filesystem
        register!(module, list_files_and_dirs(dir: &str)?);
        register!(module, path_file_name(path: &str)?);
        register!(module, join_path(path_1: &str, path_2: &str)?);

        // Doc management
        register!(module, rt.current_dir()?);
        register!(module, rt.new_doc(path: &str)?);
        register!(module, rt.open_doc(path: &str)?);
        register!(module, rt.doc_switching_candidates()?);
        register!(module, rt.switch_to_doc(path: &str)?);
        register!(module, rt.has_visible_doc());
        register!(module, rt.has_unsaved_changes());
        register!(module, rt.force_close_visible_doc()?);
        register!(module, rt.save_doc()?);
        register!(module, rt.save_doc_as(path: String)?);

        // Languages
        register!(module, rt.load_language(path: &str)?);
        register!(module, rt.get_language(language_name: &str)?);
        register!(module, rt.language_constructs(language: Language));
        register!(module, rt.construct_name(construct: Construct));
        register!(module, rt.construct_key(construct: Construct));

        // Editing: Tree Nav
        register!(module, rt, TreeNavCommand::Prev as tree_nav_prev);
        register!(module, rt, TreeNavCommand::First as tree_nav_first);
        register!(module, rt, TreeNavCommand::Next as tree_nav_next);
        register!(module, rt, TreeNavCommand::Last as tree_nav_last);
        register!(
            module,
            rt,
            TreeNavCommand::BeforeFirstChild as tree_nav_before_first_child
        );
        register!(
            module,
            rt,
            TreeNavCommand::FirstChild as tree_nav_first_child
        );
        register!(module, rt, TreeNavCommand::PrevLeaf as tree_nav_prev_leaf);
        register!(module, rt, TreeNavCommand::NextLeaf as tree_nav_next_leaf);
        register!(module, rt, TreeNavCommand::PrevText as tree_nav_prev_text);
        register!(module, rt, TreeNavCommand::NextText as tree_nav_next_text);
        register!(module, rt, TreeNavCommand::LastChild as tree_nav_last_child);
        register!(module, rt, TreeNavCommand::Parent as tree_nav_parent);
        register!(module, rt, TreeNavCommand::EnterText as tree_nav_enter_text);

        // Editing: Tree Ed
        register!(module, rt, TreeEdCommand::Backspace as tree_ed_backspace);
        register!(module, rt, TreeEdCommand::Delete as tree_ed_delete);
        register!(module, rt.insert_node(construct: Construct)?);

        // Editing: Text Nav
        register!(module, rt, TextNavCommand::Left as text_nav_left);
        register!(module, rt, TextNavCommand::Right as text_nav_right);
        register!(module, rt, TextNavCommand::Beginning as text_nav_beginning);
        register!(module, rt, TextNavCommand::End as text_nav_end);
        register!(module, rt, TextNavCommand::ExitText as text_nav_exit);

        // Editing: Text Ed
        register!(module, rt, TextEdCommand::Backspace as text_ed_backspace);
        register!(module, rt, TextEdCommand::Delete as text_ed_delete);
        register!(module, rt, TextEdCommand::Insert(ch: char) as text_ed_insert);

        // Editing: Bookmark
        register!(module, rt, BookmarkCommand::Save(ch: char) as save_bookmark);
        register!(module, rt, BookmarkCommand::Goto(ch: char) as goto_bookmark);

        // Clipboard
        register!(module, rt.cut()?);
        register!(module, rt, ClipboardCommand::Copy as copy);
        register!(module, rt, ClipboardCommand::Paste as paste);
        register!(module, rt, ClipboardCommand::PasteSwap as paste_swap);
        register!(module, rt, ClipboardCommand::Dup as dup_clipboard);
        register!(module, rt, ClipboardCommand::Pop as pop_clipboard);

        // Editing: Meta
        register!(module, rt.undo()?);
        register!(module, rt.redo()?);

        // Variables
        register!(module, rt.cli_args());

        // Logging
        register!(module, rt.log_trace(msg: String));
        register!(module, rt.log_debug(msg: String));
        register!(module, rt.log_info(msg: String));
        register!(module, rt.log_warn(msg: String));
        register!(module, rt.log_error(msg: String));
        register!(module, rt.clear_last_log());
    }
}
