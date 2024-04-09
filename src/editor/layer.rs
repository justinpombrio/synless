use super::keymap::Keymap;
use crate::tree::Mode;
use crate::util::IndexedMap;
use std::collections::HashMap;

type MenuName = String;
type LayerIndex = usize;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum KeymapLabel {
    Menu(MenuName),
    Mode(Mode),
}

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
}

// TODO: Have LayerManager track DocName -> local_layers?
//
// local:  Buffer -> Vec<LayerIndex>
// global: Vec<LayerIndex>
//
// order: buffer, global

pub struct LayerManager {
    active_local_layers: Vec<LayerIndex>,
    active_global_layers: Vec<LayerIndex>,
    active_menu: Option<MenuName>,
    layers: IndexedMap<Layer>,
}

impl LayerManager {
    pub fn add_layer(&mut self, layer: Layer) {
        self.layers.insert(layer.name.clone(), layer);
    }

    pub fn enter_menu(&mut self, menu_name: String) {
        self.active_menu = Some(menu_name);
    }

    pub fn exit_menu(&mut self) {
        self.active_menu = None;
    }
}
