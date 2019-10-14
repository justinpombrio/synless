use termion::event::Key;

use crate::prog::Prog;

use super::factory::{FilterContext, TreeKmapFactory};

// INVARIANT: The filtered keys must be present in the given mode or menu
#[derive(Clone)]
pub enum Keymap {
    Mode {
        filtered_keys: Vec<Key>,
        name: ModeName,
    },
    Menu {
        filtered_keys: Vec<Key>,
        name: MenuName,
    },
    Text,
}

// TODO use constructor instead of pub(super)?
pub struct Mode<'l> {
    pub(super) factory: TreeKmapFactory<'l>,
    pub(super) name: ModeName,
}

pub struct Menu<'l> {
    pub(super) factory: TreeKmapFactory<'l>,
    pub(super) name: MenuName,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct ModeName(pub String);

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct MenuName(pub String);

impl<'l> Mode<'l> {
    pub fn get<'a>(&'a self, key: &Key) -> Option<&'a Prog<'l>> {
        self.factory.get(key)
    }

    pub fn filter(&self, context: &FilterContext) -> Keymap {
        Keymap::Mode {
            filtered_keys: self.factory.filter(context),
            name: self.name.clone(),
        }
    }
}

impl<'l> Menu<'l> {
    pub fn get<'a>(&'a self, key: &Key) -> Option<&'a Prog<'l>> {
        self.factory.get(key)
    }

    pub fn filter(&self, context: &FilterContext) -> Keymap {
        Keymap::Menu {
            filtered_keys: self.factory.filter(context),
            name: self.name.clone(),
        }
    }
}

impl Keymap {
    pub fn name(&self) -> String {
        match self {
            Keymap::Menu { name, .. } => name.into(),
            Keymap::Mode { name, .. } => name.into(),
            Keymap::Text => "text".into(),
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
