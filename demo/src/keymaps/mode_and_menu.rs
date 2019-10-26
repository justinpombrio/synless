use super::keymap::{FilterContext, TreeKeymap};
use crate::prog::Prog;
use frontends::Key;

/// A persistent Mode that specifies which keybindings will be available in which contexts.
/// Only applies when the `Doc` is in tree-mode, not text-mode. (Sorry that
/// there are two different things here called 'modes'.)
///
/// Intended for things like a mode for a specific programming language that
/// includes some convenient language-specific refactoring commands.
pub(super) struct Mode<'l> {
    pub(super) keymap: TreeKeymap<'l>,
    pub(super) name: ModeName,
}

/// A temporary Menu that specifies which keybindings will be available in which contexts.
/// Only applies when the `Doc` is in tree-mode, not text-mode.
///
/// Intended for things like selecting a node-type from a menu.
pub(super) struct Menu<'l> {
    pub(super) keymap: TreeKeymap<'l>,
    pub(super) name: MenuName,
}

/// The name of a `Mode`.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct ModeName(String);

/// The name of a `Menu`.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct MenuName(String);

/// A description of which keybindings are available to use, based on the [KeymapManager](crate::keymaps::KeymapManager) state and document context.
#[derive(Clone)]
pub enum AvailableKeys {
    /// Use this mode for looking up keys.
    Mode {
        name: ModeName,
        /// Which subset of the mode's keys are available.
        filtered_keys: Vec<Key>,
    },
    /// Use this menu for looking up keys.
    Menu {
        name: MenuName,
        /// Which subset of the menu's keys are available.
        filtered_keys: Vec<Key>,
    },
    /// Use the one-and-only text keymap for looking up keys.
    Text,
}

impl<'l> Mode<'l> {
    /// Get the program bound to the given key, if there is one.
    pub(super) fn get<'a>(&'a self, key: &Key) -> Option<&'a Prog<'l>> {
        self.keymap.get(key)
    }

    /// Produce a filtered keymap containing only the keys that would be
    /// appropriate to use in this context.
    pub(super) fn filter(&self, context: &FilterContext) -> AvailableKeys {
        AvailableKeys::Mode {
            filtered_keys: self.keymap.filter(context),
            name: self.name.clone(),
        }
    }
}

impl<'l> Menu<'l> {
    /// Get the program bound to the given key, if there is one.
    pub(super) fn get<'a>(&'a self, key: &Key) -> Option<&'a Prog<'l>> {
        self.keymap.get(key)
    }

    /// Produce a filtered keymap containing only the keys that would be
    /// appropriate to use in this context.
    pub(super) fn filter(&self, context: &FilterContext) -> AvailableKeys {
        AvailableKeys::Menu {
            filtered_keys: self.keymap.filter(context),
            name: self.name.clone(),
        }
    }
}

impl AvailableKeys {
    pub fn name(&self) -> String {
        match self {
            AvailableKeys::Menu { name, .. } => name.into(),
            AvailableKeys::Mode { name, .. } => name.into(),
            AvailableKeys::Text => "text".into(),
        }
    }
}

impl From<String> for ModeName {
    fn from(s: String) -> ModeName {
        ModeName(s)
    }
}

impl<'a> From<&'a str> for ModeName {
    fn from(s: &'a str) -> ModeName {
        ModeName(s.to_string())
    }
}

impl From<ModeName> for String {
    fn from(m: ModeName) -> String {
        m.0
    }
}

impl<'a> From<&'a ModeName> for String {
    fn from(m: &'a ModeName) -> String {
        m.0.to_owned()
    }
}

impl From<String> for MenuName {
    fn from(s: String) -> MenuName {
        MenuName(s)
    }
}

impl<'a> From<&'a str> for MenuName {
    fn from(s: &'a str) -> MenuName {
        MenuName(s.to_string())
    }
}

impl From<MenuName> for String {
    fn from(m: MenuName) -> String {
        m.0
    }
}

impl<'a> From<&'a MenuName> for String {
    fn from(m: &'a MenuName) -> String {
        m.0.to_owned()
    }
}
