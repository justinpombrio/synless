use std::collections::HashMap;
use termion::event::Key;

use crate::error::ShellError;
use crate::prog::{Prog, Value, Word};
use language::{ArityType, Sort};

/// Rules for when a particular item should be included in a keymap
#[derive(Clone, Debug)]
pub enum KmapFilter {
    Always,
    Sort(Sort),
    ParentArity(Vec<ArityType>),
    SelfArity(Vec<ArityType>),
}

pub struct FilterContext {
    pub required_sort: Sort,
    pub parent_arity: ArityType,
    pub self_arity: ArityType,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct ModeName(pub String);

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct MenuName(pub String);

pub struct Mode<'l> {
    factory: TreeKmapFactory<'l>,
    name: ModeName,
}

pub struct Menu<'l> {
    factory: TreeKmapFactory<'l>,
    name: MenuName,
}

pub struct Keymaps<'l> {
    pub mode_stack: Vec<ModeName>,
    pub active_menu: Option<MenuName>,
    modes: HashMap<ModeName, Mode<'l>>,
    menus: HashMap<MenuName, Menu<'l>>,
    text_keymap: HashMap<Key, Prog<'l>>,
}

pub struct TreeKmapFactory<'l>(HashMap<Key, (KmapFilter, Prog<'l>)>);

// INVARIANT: The filtered keys must be present in the given mode or menu
#[derive(Clone)]
pub enum Kmap {
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

impl<'l> Keymaps<'l> {
    pub fn new() -> Self {
        Keymaps {
            modes: HashMap::new(),
            mode_stack: Vec::new(),
            menus: HashMap::new(),
            active_menu: None,
            text_keymap: HashMap::new(),
        }
    }

    pub fn insert_mode(&mut self, name: ModeName, factory: TreeKmapFactory<'l>) {
        self.modes.insert(name.clone(), Mode { factory, name });
    }

    pub fn insert_menu(&mut self, name: MenuName, factory: TreeKmapFactory<'l>) {
        self.menus.insert(name.clone(), Menu { factory, name });
    }

    pub fn set_text_keymap(&mut self, text_keymap: HashMap<Key, Prog<'l>>) {
        self.text_keymap = text_keymap;
    }

    pub fn lookup(&self, key: Key, kmap: &Kmap) -> Result<Prog<'l>, ShellError> {
        let prog = match kmap {
            Kmap::Mode {
                filtered_keys,
                name,
            } => {
                if filtered_keys.contains(&key) {
                    self.modes.get(name).unwrap().get(&key).cloned()
                } else {
                    None
                }
            }
            Kmap::Menu {
                filtered_keys,
                name,
            } => {
                if filtered_keys.contains(&key) {
                    self.menus.get(name).unwrap().get(&key).cloned()
                } else {
                    None
                }
            }
            Kmap::Text => {
                if let Some(prog) = self.text_keymap.get(&key) {
                    Some(prog.to_owned())
                } else if let Key::Char(c) = key {
                    Some(Prog::named(
                        c,
                        &[Word::Literal(Value::Char(c)), Word::InsertChar],
                    ))
                } else {
                    None
                }
            }
        };
        prog.ok_or(ShellError::UnknownKey(key))
    }

    pub fn hints(&self, kmap: &Kmap) -> Vec<(String, String)> {
        let keys_and_names: Vec<(_, _)> = match kmap {
            Kmap::Mode {
                filtered_keys,
                name,
            } => filtered_keys
                .iter()
                .map(|key| (key, self.modes.get(name).unwrap().get(key).unwrap().name()))
                .collect(),
            Kmap::Menu {
                filtered_keys,
                name,
            } => filtered_keys
                .iter()
                .map(|key| (key, self.menus.get(name).unwrap().get(key).unwrap().name()))
                .collect(),
            Kmap::Text => self
                .text_keymap
                .iter()
                .map(|(key, prog)| (key, prog.name()))
                .collect(),
        };

        let mut hints: Vec<_> = keys_and_names
            .into_iter()
            .map(|(key, name)| (format_key(key), name.unwrap_or("...").to_owned()))
            .collect();
        hints.sort_unstable();
        hints
    }

    pub fn active_keymap(
        &self,
        in_tree_mode: bool,
        context: &FilterContext,
    ) -> Result<Kmap, ShellError> {
        if !in_tree_mode {
            // TODO avoid cloning every time!
            Ok(Kmap::Text)
        } else {
            if let Some(menu_name) = &self.active_menu {
                let menu = self
                    .menus
                    .get(menu_name)
                    .ok_or_else(|| ShellError::UnknownMenuName(menu_name.to_owned()))?;
                Ok(menu.filter(&context))
            } else {
                let mode_name = self.mode_stack.last().ok_or(ShellError::NoKeymap)?;
                let mode = self
                    .modes
                    .get(mode_name)
                    .ok_or_else(|| ShellError::UnknownModeName(mode_name.to_owned()))?;
                Ok(mode.filter(&context))
            }
        }
    }
}

impl<'l> Mode<'l> {
    pub fn get<'a>(&'a self, key: &Key) -> Option<&'a Prog<'l>> {
        self.factory.get(key)
    }

    pub fn filter(&self, context: &FilterContext) -> Kmap {
        Kmap::Mode {
            filtered_keys: self.factory.filter(context),
            name: self.name.clone(),
        }
    }
}

impl<'l> Menu<'l> {
    pub fn get<'a>(&'a self, key: &Key) -> Option<&'a Prog<'l>> {
        self.factory.get(key)
    }

    pub fn filter(&self, context: &FilterContext) -> Kmap {
        Kmap::Menu {
            filtered_keys: self.factory.filter(context),
            name: self.name.clone(),
        }
    }
}

impl<'l> TreeKmapFactory<'l> {
    pub fn new(v: Vec<(Key, KmapFilter, Prog<'l>)>) -> Self {
        TreeKmapFactory(
            v.into_iter()
                .map(|(key, filter, prog)| (key, (filter, prog)))
                .collect(),
        )
    }

    fn get<'a>(&'a self, key: &Key) -> Option<&'a Prog<'l>> {
        self.0.get(key).map(|(_filter, prog)| prog)
    }

    fn filter<'a>(&'a self, context: &FilterContext) -> Vec<Key> {
        self.0
            .iter()
            .filter_map(|(&key, (filter, _))| match filter {
                KmapFilter::Always => Some(key),
                KmapFilter::Sort(sort) => {
                    if context.required_sort.accepts(sort) {
                        Some(key)
                    } else {
                        None
                    }
                }
                KmapFilter::ParentArity(arity_types) => {
                    if arity_types.contains(&context.parent_arity) {
                        Some(key)
                    } else {
                        None
                    }
                }
                KmapFilter::SelfArity(arity_types) => {
                    if arity_types.contains(&context.self_arity) {
                        Some(key)
                    } else {
                        None
                    }
                }
            })
            .collect()
    }
}

impl Kmap {
    pub fn name(&self) -> String {
        match self {
            Kmap::Menu { name, .. } => name.into(),
            Kmap::Mode { name, .. } => name.into(),
            Kmap::Text => "text".into(),
        }
    }
}

fn format_key(key: &Key) -> String {
    match key {
        Key::Backspace => "Bksp".to_string(),
        Key::Left => "←".to_string(),
        Key::Right => "→".to_string(),
        Key::Up => "↑".to_string(),
        Key::Down => "↓".to_string(),
        Key::Home => "Home".to_string(),
        Key::End => "End".to_string(),
        Key::PageUp => "PgUp".to_string(),
        Key::PageDown => "PgDn".to_string(),
        Key::Delete => "Del".to_string(),
        Key::Insert => "Ins".to_string(),
        Key::F(num) => format!("F{}", num),
        Key::Char(' ') => "Spc".to_string(),
        Key::Char(c) => c.to_string(),
        Key::Alt(' ') => "A-Spc".to_string(),
        Key::Alt(c) => format!("A-{}", c),
        Key::Ctrl(' ') => "C-Spc".to_string(),
        Key::Ctrl(c) => format!("C-{}", c),
        Key::Null => "Null".to_string(),
        Key::Esc => "Esc".to_string(),
        _ => "(unknown)".to_string(),
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
