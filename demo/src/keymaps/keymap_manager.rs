use std::collections::HashMap;
use termion::event::Key;

use crate::error::ShellError;
use crate::prog::{Prog, Value, Word};

use super::factory::{FilterContext, TreeKeymapFactory};
use super::keymap::{Keymap, Menu, MenuName, Mode, ModeName};

pub struct KeymapManager<'l> {
    pub mode_stack: Vec<ModeName>,
    pub active_menu: Option<MenuName>,
    modes: HashMap<ModeName, Mode<'l>>,
    menus: HashMap<MenuName, Menu<'l>>,
    text_keymap: HashMap<Key, Prog<'l>>,
}

impl<'l> KeymapManager<'l> {
    pub fn new() -> Self {
        KeymapManager {
            modes: HashMap::new(),
            mode_stack: Vec::new(),
            menus: HashMap::new(),
            active_menu: None,
            text_keymap: HashMap::new(),
        }
    }

    pub fn insert_mode(&mut self, name: ModeName, factory: TreeKeymapFactory<'l>) {
        self.modes.insert(name.clone(), Mode { factory, name });
    }

    pub fn insert_menu(&mut self, name: MenuName, factory: TreeKeymapFactory<'l>) {
        self.menus.insert(name.clone(), Menu { factory, name });
    }

    pub fn set_text_keymap(&mut self, text_keymap: HashMap<Key, Prog<'l>>) {
        self.text_keymap = text_keymap;
    }

    pub fn lookup(&self, key: Key, kmap: &Keymap) -> Result<Prog<'l>, ShellError> {
        let prog = match kmap {
            Keymap::Mode {
                filtered_keys,
                name,
            } => {
                if filtered_keys.contains(&key) {
                    self.modes.get(name).unwrap().get(&key).cloned()
                } else {
                    None
                }
            }
            Keymap::Menu {
                filtered_keys,
                name,
            } => {
                if filtered_keys.contains(&key) {
                    self.menus.get(name).unwrap().get(&key).cloned()
                } else {
                    None
                }
            }
            Keymap::Text => {
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

    pub fn hints(&self, kmap: &Keymap) -> Vec<(String, String)> {
        let keys_and_names: Vec<(_, _)> = match kmap {
            Keymap::Mode {
                filtered_keys,
                name,
            } => filtered_keys
                .iter()
                .map(|key| (key, self.modes.get(name).unwrap().get(key).unwrap().name()))
                .collect(),
            Keymap::Menu {
                filtered_keys,
                name,
            } => filtered_keys
                .iter()
                .map(|key| (key, self.menus.get(name).unwrap().get(key).unwrap().name()))
                .collect(),
            Keymap::Text => self
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
    ) -> Result<Keymap, ShellError> {
        if !in_tree_mode {
            // TODO avoid cloning every time!
            Ok(Keymap::Text)
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
