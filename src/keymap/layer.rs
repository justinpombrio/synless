use super::keymap::{KeyProg, Keymap};
use super::menu::{Menu, MenuName, MenuSelectionCmd};
use crate::engine::DocName;
use crate::frontends::Key;
use crate::language::Storage;
use crate::tree::Mode;
use crate::tree::Node;
use crate::util::{error, IndexedMap, SynlessError};
use std::collections::HashMap;

type LayerIndex = usize;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum KeymapLabel {
    Menu(MenuName),
    Mode(Mode),
}

pub enum KeyLookupResult {
    KeyProg(KeyProg),
    InsertChar(char),
    Redisplay,
}

/*********
 * Layer *
 *********/

/// A collection of Keymaps, with up to one Keymap per `MenuName` or `Mode`. Layers can be stacked
/// on top of each other to combine their functionality; see [`LayerManager`].
#[derive(Debug, Clone)]
pub struct Layer {
    name: String,
    keymaps: HashMap<KeymapLabel, Keymap>,
}

impl Layer {
    pub fn new(name: String) -> Layer {
        Layer {
            name,
            keymaps: HashMap::new(),
        }
    }

    pub fn add_menu_keymap(&mut self, menu_name: MenuName, keymap: Keymap) {
        self.keymaps.insert(KeymapLabel::Menu(menu_name), keymap);
    }

    pub fn add_mode_keymap(&mut self, mode: Mode, keymap: Keymap) {
        self.keymaps.insert(KeymapLabel::Mode(mode), keymap);
    }

    // If the same KeymapLabel is used in multiple layers, later layers override earlier layers
    fn merge(name: String, layers: impl IntoIterator<Item = Layer>) -> Layer {
        let mut keymaps = HashMap::<KeymapLabel, Keymap>::new();
        for layer in layers {
            for (label, keymap) in layer.keymaps {
                if let Some(merged_keymap) = keymaps.get_mut(&label) {
                    merged_keymap.append(keymap);
                } else {
                    keymaps.insert(label, keymap);
                }
            }
        }
        Layer { name, keymaps }
    }
}

impl rhai::CustomType for Layer {
    fn build(mut builder: rhai::TypeBuilder<Self>) {
        use std::str::FromStr;

        builder
            .with_name("Layer")
            .with_get("name", |layer: &mut Layer| -> String { layer.name.clone() })
            .with_fn("new_layer", Layer::new)
            .with_fn("add_menu_keymap", Layer::add_menu_keymap)
            .with_fn(
                "add_mode_keymap",
                |layer: &mut Layer,
                 mode_str: &str,
                 keymap: Keymap|
                 -> Result<(), Box<rhai::EvalAltResult>> {
                    let mode = Mode::from_str(mode_str)
                        .map_err(|err| error!(Keymap, "{err}: {mode_str}"))?;
                    layer.add_mode_keymap(mode, keymap);
                    Ok(())
                },
            );
    }
}

/****************
 * LayerManager *
 ****************/

/// Manage [`Layer`]s of [`Keymap`]s, and track the active menu.
///
/// Layers can be stacked on top of each other. There is a global stack of layers that apply to all
/// documents, as well as a local stack of layers for each individual document. When different
/// layers have conflicting bindings, layers higher in the stack take priority over lower layers,
/// and local layers take priority over global layers.
pub struct LayerManager {
    global_layers: Vec<LayerIndex>,
    local_layers: HashMap<DocName, Vec<LayerIndex>>,
    layers: IndexedMap<Layer>,
    active_menu: Option<Menu>,
    cached_composite_layers: HashMap<Vec<LayerIndex>, Layer>,
}

impl LayerManager {
    pub fn new() -> LayerManager {
        LayerManager {
            global_layers: Vec::new(),
            local_layers: HashMap::new(),
            layers: IndexedMap::new(),
            active_menu: None,
            cached_composite_layers: HashMap::new(),
        }
    }

    /**********
     * Layers *
     **********/

    /// Register the layer, so that it can later be added or removed by name.
    pub fn register_layer(&mut self, layer: Layer) {
        self.layers.insert(layer.name.clone(), layer);
    }

    /// Add a global keymap layer to the top of the global layer stack. Returns `Err` if the layer
    /// has not been registered.
    pub fn add_global_layer(&mut self, layer_name: &str) -> Result<(), SynlessError> {
        add_layer(&self.layers, &mut self.global_layers, layer_name)?;
        self.cached_composite_layers.clear();
        Ok(())
    }

    /// Add a keymap layer to the top of the given document's local layer stack. Returns `Err` if
    /// the layer has not been registered.
    pub fn add_local_layer(
        &mut self,
        doc_name: &DocName,
        layer_name: &str,
    ) -> Result<(), SynlessError> {
        let local_layers = self.local_layers.entry(doc_name.to_owned()).or_default();
        add_layer(&self.layers, local_layers, layer_name)?;
        self.cached_composite_layers.clear();
        Ok(())
    }

    /// Remove a global keymap layer from wherever it is in the global layer stack. Returns `Err`
    /// if the layer has not been registered.
    pub fn remove_global_layer(&mut self, layer_name: &str) -> Result<(), SynlessError> {
        remove_layer(&self.layers, &mut self.global_layers, layer_name)?;
        self.cached_composite_layers.clear();
        Ok(())
    }

    /// Remove a keymap layer from wherever it is in the given document's local layer stack.
    /// Returns `Err` if the layer has not been registered.
    pub fn remove_local_layer(
        &mut self,
        doc_name: &DocName,
        layer_name: &str,
    ) -> Result<(), SynlessError> {
        let local_layers = self.local_layers.entry(doc_name.to_owned()).or_default();
        remove_layer(&self.layers, local_layers, layer_name)?;
        self.cached_composite_layers.clear();
        Ok(())
    }

    /// Iterate over all global layer names, from the bottom to top of the stack.
    pub fn global_layers(&self) -> impl Iterator<Item = &str> {
        self.global_layers
            .iter()
            .map(|i| self.layers[*i].name.as_ref())
    }

    /// Iterate over all local layer names for the given document, from the bottom to top of the
    /// stack.
    pub fn local_layers(&self, doc_name: &DocName) -> impl Iterator<Item = &str> {
        self.local_layers
            .get(doc_name)
            .into_iter()
            .flat_map(|layers| layers.iter().map(|i| self.layers[*i].name.as_ref()))
    }

    /// Iterate over all registered layer names, regardless of whether they're currently in use, in
    /// arbitrary order.
    pub fn all_layers(&self) -> impl Iterator<Item = &str> {
        self.layers.names()
    }

    /*********
     * Menus *
     *********/

    /// Open the named menu. If `dynamic_keymap` is `Some`, layer it on top of the existing keymaps
    /// for the menu. Returns `false` and does nothing if there's no menu to open (this happens
    /// when none of the layers have a menu of this name and `dynamic_keymap` is `None`).
    pub fn open_menu(
        &mut self,
        doc_name: Option<&DocName>,
        menu_name: String,
        dynamic_keymap: Option<Keymap>,
    ) -> Result<(), SynlessError> {
        let composite_layer = self.composite_layer(doc_name);
        let label = KeymapLabel::Menu(menu_name.clone());
        let menu = match (dynamic_keymap, composite_layer.keymaps.get(&label)) {
            (None, None) => return Err(error!(Keymap, "No keymap for menu '{menu_name}'")),
            (Some(keymap), None) => Menu::new(menu_name, keymap),
            (Some(dyn_keymap), Some(composite_keymap)) => {
                let mut keymap = composite_keymap.to_owned();
                keymap.append(dyn_keymap);
                Menu::new(menu_name, keymap)
            }
            (None, Some(keymap)) => Menu::new(menu_name, keymap.to_owned()),
        };
        self.active_menu = Some(menu);
        Ok(())
    }

    pub fn close_menu(&mut self) {
        self.active_menu = None;
    }

    /// Manipulate the menu's candidate selection. Returns `false` and does nothing if there is no
    /// menu open, or it does not have candidate selection.
    pub fn edit_menu_selection(&mut self, cmd: MenuSelectionCmd) -> Result<(), SynlessError> {
        let is_ok = if let Some(menu) = &mut self.active_menu {
            menu.execute(cmd)
        } else {
            false
        };
        if is_ok {
            Ok(())
        } else {
            Err(error!(Keymap, "No selection menu to edit"))
        }
    }

    /*********
     * Input *
     *********/

    /// Lookup the program to run when the given key is pressed, given the current mode and active
    /// document.
    pub fn lookup_key(
        &mut self,
        mode: Mode,
        doc_name: Option<&DocName>,
        key: Key,
    ) -> Option<KeyLookupResult> {
        if let Some(menu) = &mut self.active_menu {
            if let Some(key_prog) = menu.lookup(key) {
                return Some(KeyLookupResult::KeyProg(key_prog));
            }
            if let Some(ch) = key.as_plain_char() {
                if menu.execute(MenuSelectionCmd::Insert(ch)) {
                    return Some(KeyLookupResult::Redisplay);
                }
            }
        } else {
            let layer = self.composite_layer(doc_name);
            let keymap = layer.keymaps.get(&KeymapLabel::Mode(mode))?;
            if let Some(key_prog) = keymap.lookup(key, None) {
                return Some(KeyLookupResult::KeyProg(key_prog));
            }
            if mode == Mode::Text {
                if let Some(ch) = key.as_plain_char() {
                    return Some(KeyLookupResult::InsertChar(ch));
                }
            }
        }
        None
    }

    /***********
     * Display *
     ***********/

    pub fn make_candidate_selection_doc(&self, s: &mut Storage) -> Option<Node> {
        self.active_menu
            .as_ref()
            .and_then(|menu| menu.make_candidate_selection_doc(s))
    }

    pub fn make_keyhint_doc(
        &mut self,
        s: &mut Storage,
        mode: Mode,
        doc_name: Option<&DocName>,
    ) -> Option<Node> {
        if let Some(menu) = &self.active_menu {
            Some(menu.make_keyhint_doc(s))
        } else {
            let layer = self.composite_layer(doc_name);
            let keymap = layer.keymaps.get(&KeymapLabel::Mode(mode))?;
            Some(keymap.make_keyhint_doc(s, None))
        }
    }

    /***********
     * Private *
     ***********/

    /// Get a composite layer that merges together all active layers. It is cached.
    fn composite_layer(&mut self, doc_name: Option<&DocName>) -> &Layer {
        let layer_indices = self.active_layers(doc_name).collect::<Vec<_>>();
        if !self.cached_composite_layers.contains_key(&layer_indices) {
            let layers_iter = layer_indices.iter().map(|i| &self.layers[*i]).cloned();
            let composite_layer = Layer::merge("COMPOSITE_LAYER".to_owned(), layers_iter);
            self.cached_composite_layers
                .insert(layer_indices.clone(), composite_layer);
        }
        &self.cached_composite_layers[&layer_indices]
    }

    /// Iterates over layers, yielding the lowest priority layers first.
    fn active_layers(&self, doc_name: Option<&DocName>) -> impl Iterator<Item = usize> + '_ {
        let global_layer_indices = self.global_layers.iter();
        let local_layer_indices = doc_name
            .and_then(|doc_name| self.local_layers.get(doc_name))
            .into_iter()
            .flat_map(|indices| indices.iter());

        local_layer_indices.chain(global_layer_indices).copied()
    }
}

fn add_layer(
    layers: &IndexedMap<Layer>,
    active_layers: &mut Vec<LayerIndex>,
    layer_name: &str,
) -> Result<(), SynlessError> {
    let layer_index = layers.id(layer_name).ok_or_else(|| {
        error!(
            Keymap,
            "Layer {layer_name} cannot be added because it has not been registered"
        )
    })?;
    active_layers.retain(|i| *i != layer_index); // remove lower priority duplicate
    active_layers.push(layer_index);
    Ok(())
}

fn remove_layer(
    layers: &IndexedMap<Layer>,
    active_layers: &mut Vec<LayerIndex>,
    layer_name: &str,
) -> Result<(), SynlessError> {
    let layer_index = layers.id(layer_name).ok_or_else(|| {
        error!(
            Keymap,
            "Layer {layer_name} cannot be removed because it has not been registered"
        )
    })?;
    active_layers.retain(|i| *i != layer_index);
    Ok(())
}
