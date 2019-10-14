use frontends::Key;
use std::collections::HashMap;

use crate::error::ShellError;
use crate::prog::{Prog, Value, Word};

use super::factory::{FilterContext, TextKeymapFactory, TreeKeymapFactory};
use super::keymap::{FilteredKeymap, Menu, MenuName, Mode, ModeName};

/// Manage various forms of keymaps
pub struct KeymapManager<'l> {
    /// The top of the stack is the current, persistent mode. It's keymap will
    /// be used whenever the document is in tree-mode and there is no menu
    /// active.
    mode_stack: Vec<ModeName>,
    /// If there is an active menu, its keymap will be used instead of the current mode's.
    /// Menu's are meant to be shortlived (eg. deactivated after a single keypress).
    active_menu: Option<MenuName>,
    /// All known modes.
    modes: HashMap<ModeName, Mode<'l>>,
    /// All known menus.
    menus: HashMap<MenuName, Menu<'l>>,
    /// The one-and-only keymap used for entering text. Maybe we should allow
    /// multiple text keymaps, someday.
    text_keymap: TextKeymapFactory<'l>,
}

impl<'l> KeymapManager<'l> {
    pub fn new() -> Self {
        KeymapManager {
            modes: HashMap::new(),
            mode_stack: Vec::new(),
            menus: HashMap::new(),
            active_menu: None,
            text_keymap: TextKeymapFactory::empty(),
        }
    }

    /// Register a new mode for later use.
    pub fn register_mode(&mut self, name: ModeName, factory: TreeKeymapFactory<'l>) {
        self.modes.insert(name.clone(), Mode { factory, name });
    }

    /// Register a new menu for later use.
    pub fn register_menu(&mut self, name: MenuName, factory: TreeKeymapFactory<'l>) {
        self.menus.insert(name.clone(), Menu { factory, name });
    }

    /// Register a new text keymap for later use. Since we currently only
    /// support one text keymap at a time, this replaces the existing one.
    pub fn replace_text_keymap(&mut self, text_keymap: TextKeymapFactory<'l>) {
        self.text_keymap = text_keymap;
    }

    /// Push this mode onto the stack, making it the current mode. Return an
    /// error if the mode has not been registered.
    pub fn push_mode(&mut self, name: ModeName) -> Result<(), ShellError> {
        if self.modes.contains_key(&name) {
            self.mode_stack.push(name);
            Ok(())
        } else {
            Err(ShellError::UnknownModeName(name))
        }
    }

    /// Pop the mode stack, switching back to the previous mode.
    pub fn pop_mode(&mut self) -> Option<ModeName> {
        self.mode_stack.pop()
    }

    /// Activate this menu, temporarily overriding the current mode until it's
    /// deactivated.
    pub fn activate_menu(&mut self, name: MenuName) {
        self.active_menu = Some(name);
    }

    /// Deactivate the active menu, if there is one.
    pub fn deactivate_menu(&mut self) {
        self.active_menu = None;
    }

    /// True if there is an active menu overriding the current mode.
    pub fn has_active_menu(&self) -> bool {
        self.active_menu.is_some()
    }

    /// Return the program that's mapped to this key in the given keymap, or None if the key isn't found.
    pub fn lookup(&self, key: Key, keymap: &FilteredKeymap) -> Option<Prog<'l>> {
        match keymap {
            FilteredKeymap::Mode {
                filtered_keys,
                name,
            } => {
                if filtered_keys.contains(&key) {
                    self.modes.get(name).unwrap().get(&key).cloned()
                } else {
                    None
                }
            }
            FilteredKeymap::Menu {
                filtered_keys,
                name,
            } => {
                if filtered_keys.contains(&key) {
                    self.menus.get(name).unwrap().get(&key).cloned()
                } else {
                    None
                }
            }
            FilteredKeymap::Text => {
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
        }
    }

    /// Return a list of 'key name' and 'program name' pairs for the given keymap.
    pub fn hints(&self, keymap: &FilteredKeymap) -> Vec<(String, String)> {
        let keys_and_names: Vec<(_, _)> = match keymap {
            FilteredKeymap::Mode {
                filtered_keys,
                name,
            } => filtered_keys
                .iter()
                .map(|key| (key, self.modes.get(name).unwrap().get(key).unwrap().name()))
                .collect(),
            FilteredKeymap::Menu {
                filtered_keys,
                name,
            } => filtered_keys
                .iter()
                .map(|key| (key, self.menus.get(name).unwrap().get(key).unwrap().name()))
                .collect(),
            FilteredKeymap::Text => self.text_keymap.keys_and_names(),
        };

        let mut hints: Vec<_> = keys_and_names
            .into_iter()
            .map(|(key, name)| (format_key(key), name.unwrap_or("...").to_owned()))
            .collect();
        hints.sort_unstable();
        hints
    }

    /// Return the keymap that should be used to lookup keypresses, based on the current state of the
    /// KeymapManager and the context within a particular document.
    ///
    /// If the document is in text-mode, `tree_context` should be None.
    pub fn get_active_keymap(
        &self,
        tree_context: Option<FilterContext>,
    ) -> Result<FilteredKeymap, ShellError> {
        if let Some(context) = tree_context {
            if let Some(menu_name) = &self.active_menu {
                let menu = self
                    .menus
                    .get(menu_name)
                    .ok_or_else(|| ShellError::UnknownMenuName(menu_name.to_owned()))?;
                Ok(menu.filter(&context))
            } else {
                let mode_name = self.mode_stack.last().ok_or(ShellError::NoMode)?;
                let mode = self
                    .modes
                    .get(mode_name)
                    .ok_or_else(|| ShellError::UnknownModeName(mode_name.to_owned()))?;
                Ok(mode.filter(&context))
            }
        } else {
            Ok(FilteredKeymap::Text)
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
    }
}
