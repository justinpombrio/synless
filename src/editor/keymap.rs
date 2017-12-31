use std::collections::HashMap;

use terminal::Key;
use editor::command::Command;
use doc::Mode::{self, TreeMode, TextMode};
use language::Language;


/// A set of keyboard bindings.
pub struct KeyMap {
    tree_map: HashMap<Key, Action>,
    text_map: HashMap<Key, Action>,
    groups:   HashMap<String, KeyGroup>
}

/// A group of keyboard bindings: several key bindings that can be
/// accessed by first pressing a shared common key.
struct KeyGroup {
    keymap: KeyMap
}

/// An action to perform on a key press.
#[derive(Clone)]
pub enum Action {
    /// Perform the builtin command.
    Command(Command),
    /// Enter a subgroup of keyboard bindings.
    KeyGroup(String)
}

impl KeyMap {
    pub fn new() -> KeyMap {
        KeyMap{
            tree_map: HashMap::new(),
            text_map: HashMap::new(),
            groups:   HashMap::new()
        }
    }

    /// Lookup what should happen when a key is pressed.
    pub fn lookup(&self, key: Key, mode: Mode) -> Option<Action> {
        if let (TextMode, Key::Char(key)) = (mode, key) {
            return Some(Action::Command(Command::InsertChar(key)))
        }
        let map = match mode {
            TreeMode => &self.tree_map,
            TextMode => &self.text_map
        };
        match map.get(&key) {
            None => None,
            Some(action) => Some(action.clone())
        }
    }

    /// Lookup the KeyMap for a `KeyGroup(name)` action.
    pub fn lookup_keygroup(&self, name: &str) -> &KeyMap {
        match self.groups.get(name) {
            Some(keygroup) => &keygroup.keymap,
            None => panic!("Key group not found.")
        }
    }

    /// Add a new text mode key binding, that performs a builtin command.
    pub fn add_text_cmd(&mut self, key: Key, cmd: Command) {
        self.text_map.insert(key, Action::Command(cmd));
    }

    /// Add a new tree mode key binding, that performs a builtin command.
    pub fn add_tree_cmd(&mut self, key: Key, cmd: Command) {
        self.tree_map.insert(key, Action::Command(cmd));
    }

    /// Add a grouped set of key bindings for text mode.
    pub fn add_text_keygroup(&mut self, key: Key, name: &str, keymap: KeyMap) {
        let group = KeyGroup{
            keymap: keymap
        };
        let action = Action::KeyGroup(name.to_string());

        self.groups.insert(name.to_string(), group);
        self.text_map.insert(key, action);
    }

    /// Add a grouped set of key bindings for tree mode.
    pub fn add_tree_keygroup(&mut self, key: Key, name: &str, keymap: KeyMap) {
        let group = KeyGroup{
            keymap: keymap
        };
        let action = Action::KeyGroup(name.to_string());

        self.groups.insert(name.to_string(), group);
        self.tree_map.insert(key, action);
    }

    /// A keymap for `Language::example_language()`.
    pub fn example_keymap(lang: &Language) -> KeyMap {
        let mut map = KeyMap::new();

        // Tree Navigation
        map.add_tree_cmd(Key::Left,      Command::Left);
        map.add_tree_cmd(Key::Right,     Command::Right);
        map.add_tree_cmd(Key::Up,        Command::Up);
        map.add_tree_cmd(Key::Down,      Command::Down);
        map.add_tree_cmd(Key::Char('h'), Command::Left);
        map.add_tree_cmd(Key::Char('j'), Command::Down);
        map.add_tree_cmd(Key::Char('k'), Command::Up);
        map.add_tree_cmd(Key::Char('l'), Command::Right);
        // Text Navigation
        map.add_text_cmd(Key::Left,  Command::LeftChar);
        map.add_text_cmd(Key::Right, Command::RightChar);
        // Modes
        map.add_tree_cmd(Key::Enter, Command::EnterText);
        map.add_text_cmd(Key::Enter, Command::ExitText);
        // Tree Editing
        map.add_tree_cmd(Key::Char('a'), Command::AddChild);
        map.add_tree_cmd(Key::Backspace, Command::DeleteTree);
        // Text Editing
        map.add_text_cmd(Key::Backspace, Command::DeleteChar);

        let mut lang_keymap = KeyMap::new();
        for (&key, name) in &lang.keymap {
            let cmd = Command::ReplaceTree(name.to_string());
            lang_keymap.add_tree_cmd(Key::Char(key), cmd);
        }
        map.add_tree_keygroup(Key::Char('i'), "insert", lang_keymap);
        
        map
    }
}
