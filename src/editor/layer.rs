use super::keymap::{Candidate, Keymap};
use super::stack::Prog;
use crate::engine::DocName;
use crate::frontends::Key;
use crate::tree::Mode;
use crate::util::IndexedMap;
use std::collections::HashMap;

// TODO:
// - filtering by sort
// - docs
// - proofread keymap & layer
// - add tests

type MenuName = String;
type LayerIndex = usize;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum KeymapLabel {
    Menu(MenuName),
    Mode(Mode),
}

/********
 * Menu *
 ********/

pub enum MenuSelectionCmd {
    Up,
    Down,
    Backspace,
    Insert(char),
}

struct Menu {
    name: MenuName,
    keymap: Keymap,
    /// The user has input `String`, and selected the candidate at index `u32`.
    selection: Option<MenuSelection>,
}

struct MenuSelection {
    input: String,
    index: u32,
    default_index: u32,
}

impl MenuSelection {
    fn new(keymap: &Keymap) -> Option<MenuSelection> {
        if keymap.has_candidates() {
            let default_index = if keymap.has_custom() { 1 } else { 0 };
            Some(MenuSelection {
                input: String::new(),
                index: default_index,
                default_index,
            })
        } else {
            None
        }
    }

    fn execute(&mut self, cmd: MenuSelectionCmd) {
        use MenuSelectionCmd::{Backspace, Down, Insert, Up};

        match cmd {
            Up => self.index = self.index.saturating_sub(1),
            Down => self.index += 1,
            Backspace => {
                self.input.pop();
            }
            Insert(ch) => self.input.push(ch),
        }
    }
}

impl Menu {
    fn new(name: MenuName, keymap: Keymap) -> Menu {
        Menu {
            name,
            selection: MenuSelection::new(&keymap),
            keymap,
        }
    }
}

/*********
 * Layer *
 *********/

// TODO: doc
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

    pub fn add_menu(&mut self, menu_name: MenuName, keymap: Keymap) {
        self.keymaps.insert(KeymapLabel::Menu(menu_name), keymap);
    }

    pub fn add_mode(&mut self, mode: Mode, keymap: Keymap) {
        self.keymaps.insert(KeymapLabel::Mode(mode), keymap);
    }

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

/****************
 * LayerManager *
 ****************/

/// Manage keymap layers.
///
/// Layers added later has priority over layers added earlier,
/// though every local layer has priority over every global layer.
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

    pub fn register_layer(&mut self, layer: Layer) {
        self.layers.insert(layer.name.clone(), layer);
    }

    /// Add a global keymap layer. Returns `Err` if the layer has not been registered.
    pub fn add_global_layer(&mut self, layer_name: &str) -> Result<(), ()> {
        add_layer(&self.layers, &mut self.global_layers, layer_name)?;
        self.cached_composite_layers.clear();
        Ok(())
    }

    /// Add a keymap layer for this document. Returns `Err` if the layer has not been registered.
    pub fn add_local_layer(&mut self, doc_name: &DocName, layer_name: &str) -> Result<(), ()> {
        let mut local_layers = self.local_layers.entry(doc_name.to_owned()).or_default();
        add_layer(&self.layers, local_layers, layer_name)?;
        self.cached_composite_layers.clear();
        Ok(())
    }

    /// Remove a global keymap layer. Returns `Err` if the layer has not been registered.
    pub fn remove_global_layer(&mut self, layer_name: &str) -> Result<(), ()> {
        remove_layer(&self.layers, &mut self.global_layers, layer_name)?;
        self.cached_composite_layers.clear();
        Ok(())
    }

    /// Remove a keymap layer for this document. Returns `Err` if the layer has not been
    /// registered.
    pub fn remove_local_layer(&mut self, doc_name: &DocName, layer_name: &str) -> Result<(), ()> {
        let mut local_layers = self.local_layers.entry(doc_name.to_owned()).or_default();
        remove_layer(&self.layers, local_layers, layer_name)?;
        self.cached_composite_layers.clear();
        Ok(())
    }

    /// Iterate over all active global layer names.
    pub fn global_layers(&self) -> impl Iterator<Item = &str> {
        self.global_layers
            .iter()
            .map(|i| self.layers[*i].name.as_ref())
    }

    /// Iterate over all active local layer names for the given document.
    pub fn local_layers(&self, doc_name: &DocName) -> impl Iterator<Item = &str> {
        self.local_layers
            .get(doc_name)
            .into_iter()
            .flat_map(|layers| layers.iter().map(|i| self.layers[*i].name.as_ref()))
    }

    /*********
     * Menus *
     *********/

    /// Open the named menu. If `dynamic_keymap` is `Some`, layer it on top of the existing
    /// keymaps for the menu. Returns `false` if none of the layers have a menu of this name.
    #[must_use]
    pub fn enter_menu(
        &mut self,
        doc_name: Option<&DocName>,
        menu_name: String,
        dynamic_keymap: Option<Keymap>,
    ) -> bool {
        let composite_layer = self.composite_layer(doc_name);
        let label = KeymapLabel::Menu(menu_name.clone());
        let menu = match (dynamic_keymap, composite_layer.keymaps.get(&label)) {
            (None, None) => return false,
            (Some(keymap), None) => Menu::new(menu_name, keymap),
            (Some(dyn_keymap), Some(composite_keymap)) => {
                let mut keymap = composite_keymap.to_owned();
                keymap.append(dyn_keymap);
                Menu::new(menu_name, keymap)
            }
            (None, Some(keymap)) => Menu::new(menu_name, keymap.to_owned()),
        };
        self.active_menu = Some(menu);
        true
    }

    pub fn exit_menu(&mut self) {
        self.active_menu = None;
    }

    /// Edit the menu input selection.
    /// Returns `false` if there is no menu open, or it does not have candidate selection.
    #[must_use]
    pub fn edit_menu_selection(&mut self, cmd: MenuSelectionCmd) -> bool {
        if let Some(selection) = self
            .active_menu
            .as_mut()
            .and_then(|menu| menu.selection.as_mut())
        {
            selection.execute(cmd);
            true
        } else {
            false
        }
    }

    /*********
     * ??? *
     *********/

    pub fn has_candidates(&mut self, mode: Mode, doc_name: Option<&DocName>) -> bool {
        self.composite_keymap(mode, doc_name)
            .map(|keymap| keymap.has_candidates())
            .unwrap_or(false)
    }

    pub fn filtered_candidates<'a>(
        &'a mut self,
        mode: Mode,
        doc_name: Option<&DocName>,
        pattern: &'a str,
    ) -> impl Iterator<Item = Candidate<'a>> {
        self.composite_keymap(mode, doc_name)
            .into_iter()
            .flat_map(|keymap| keymap.filtered_candidates(pattern))
    }

    pub fn lookup_key(
        &mut self,
        mode: Mode,
        doc_name: Option<&DocName>,
        key: Key,
        candidate: Option<Candidate>,
    ) -> Option<Prog> {
        self.composite_keymap(mode, doc_name)?
            .lookup(key, candidate)
    }

    pub fn available_keys<'a>(
        &'a mut self,
        mode: Mode,
        doc_name: Option<&DocName>,
        candidate: Option<Candidate<'a>>,
    ) -> impl Iterator<Item = (Key, &'a str)> {
        self.composite_keymap(mode, doc_name)
            .into_iter()
            .flat_map(move |keymap| keymap.available_keys(candidate))
    }

    /***********
     * Private *
     ***********/

    /// Get a composite keymap that merges together all active keymaps for the given mode&document.
    /// It is cached.
    fn composite_keymap(&mut self, mode: Mode, doc_name: Option<&DocName>) -> Option<&Keymap> {
        // It would be nicer to just call `composite_layer()`, but the borrow checker
        // dislikes that.
        let layer_indices = match &self.active_menu {
            Some(_) => None,
            None => Some(self.cache_composite_layer(doc_name)),
        };
        match &self.active_menu {
            Some(menu) => Some(&menu.keymap),
            None => {
                let layer = &self.cached_composite_layers[&layer_indices.unwrap()];
                layer.keymaps.get(&KeymapLabel::Mode(mode))
            }
        }
    }

    /// Get a composite layer that merges together all active layers. It is cached.
    fn composite_layer(&mut self, doc_name: Option<&DocName>) -> &Layer {
        let layer_indices = self.cache_composite_layer(doc_name);
        &self.cached_composite_layers[&layer_indices]
    }

    /// Cache a composite layer that merges together all active layers. It can subsequently be
    /// found by looking up the return value (layer indices) in `cached_composite_layers`.
    fn cache_composite_layer(&mut self, doc_name: Option<&DocName>) -> Vec<usize> {
        let layer_indices = self.active_layers(doc_name).collect::<Vec<_>>();
        if !self.cached_composite_layers.contains_key(&layer_indices) {
            let layers_iter = layer_indices.iter().map(|i| &self.layers[*i]).cloned();
            let composite_layer = Layer::merge("COMPOSITE_LAYER".to_owned(), layers_iter);
            self.cached_composite_layers
                .insert(layer_indices.clone(), composite_layer);
        }
        layer_indices
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
) -> Result<(), ()> {
    let layer_index = layers.id(layer_name).ok_or(())?;
    active_layers.retain(|i| *i != layer_index); // remove lower priority duplicate
    active_layers.push(layer_index);
    Ok(())
}

fn remove_layer(
    layers: &IndexedMap<Layer>,
    active_layers: &mut Vec<LayerIndex>,
    layer_name: &str,
) -> Result<(), ()> {
    let layer_index = layers.id(layer_name).ok_or(())?;
    active_layers.retain(|i| *i != layer_index);
    Ok(())
}
