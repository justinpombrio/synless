use crate::frontends::Key;
use crate::util::OrderedMap;
use crate::util::SynlessBug;
use std::borrow::Borrow;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Prog {}

/// Key bindings.
///
/// All methods that add bindings will overwrite existing bindings.
pub struct Keymap {
    /// If the user types `Key`, execute `progs[usize]`.
    key_map: OrderedMap<Key, usize>,
    /// If the user types `Key` while `String` is selected, execute `progs[usize]`.
    string_map: OrderedMap<String, OrderedMap<Key, usize>>,
    progs: Vec<KeyProg>,
}

/// A named program that can be used in a key binding.
/// Represents the regular program that:
///
/// 1. If `exit_menu`, exits the menu.
/// 2. If `push_string`, pushes the selected keymap string onto the stack.
/// 3. Executes `prog`.
struct KeyProg {
    name: String,
    exit_menu: bool,
    push_string: bool,
    prog: Prog,
}

impl KeyProg {
    fn to_program(&self, _string: Option<String>) -> Prog {
        todo!()
    }
}

impl Keymap {
    pub fn new() -> Keymap {
        Keymap {
            key_map: OrderedMap::new(),
            string_map: OrderedMap::new(),
            progs: Vec::new(),
        }
    }

    /// If the user types `key`, execute `prog`, potentially after exiting the menu.
    /// Use `name` when displaying this binding.
    pub fn bind_key(&mut self, key: Key, name: String, prog: Prog, exit_menu: bool) {
        let index = self.add_prog(KeyProg {
            name,
            exit_menu,
            push_string: false,
            prog,
        });
        self.key_map.insert(key, index);
    }

    /// If the user types `key` while `string` is selected, execute `prog`,
    /// potentially after exiting the menu.
    /// Use `name` when displaying this binding.
    pub fn bind_string(
        &mut self,
        string: String,
        key: Key,
        name: String,
        prog: Prog,
        exit_menu: bool,
    ) {
        let index = self.add_prog(KeyProg {
            name,
            exit_menu,
            push_string: false,
            prog,
        });
        self.bind_string_to_prog_index(string, key, index);
    }

    /// If the user types `key` while one of the `strings` is selected,
    /// push that string onto the stack and then execute `prog`,
    /// potentially after exiting the menu.
    /// Use `name` when displaying this binding.
    pub fn bind_string_set(
        &mut self,
        strings: Vec<String>,
        key: Key,
        name: String,
        prog: Prog,
        exit_menu: bool,
    ) {
        let index = self.add_prog(KeyProg {
            name,
            exit_menu,
            push_string: true,
            prog,
        });
        for string in strings {
            self.bind_string_to_prog_index(string, key, index);
        }
    }

    /// Returns the program to execute if `key` is pressed while `string` is selected.
    pub fn lookup_prog(&self, key: Key, string: Option<&str>) -> Option<Prog> {
        let index = if let Some(index) = self.key_map.get(&key) {
            *index
        } else {
            *self.string_map.get(string?)?.get(&key)?
        };
        Some(self.progs[index].to_program(string.map(|s| s.to_owned())))
    }

    /// Whether this keymap contains any candidate strings.
    pub fn has_strings(&self) -> bool {
        !self.string_map.is_empty()
    }

    /// Iterates over all of the candidate strings.
    pub fn strings(&self) -> impl Iterator<Item = &str> {
        self.string_map.keys().map(|s: &String| s.borrow())
    }

    // TODO: Implement fuzzy search
    /// Iterates over the candidate strings that match the given pattern.
    pub fn filtered_strings<'s>(&'s self, pattern: &'s str) -> impl Iterator<Item = &'s str> {
        self.string_map.keys().filter_map(move |s: &String| {
            if s.contains(pattern) {
                Some(s.borrow())
            } else {
                None
            }
        })
    }

    /// Iterates over all `(key, name)` pairs that are available, given that `string` is selected.
    pub fn available_bindings(&self, string: Option<&str>) -> impl Iterator<Item = (Key, &str)> {
        let key_bindings = self.get_map_bindings(&self.key_map);
        let string_bindings = string
            .and_then(|s| self.string_map.get(s))
            .into_iter()
            .flat_map(|string_keymap| self.get_map_bindings(string_keymap));
        key_bindings.chain(string_bindings)
    }

    fn get_map_bindings<'a>(
        &'a self,
        map: &'a OrderedMap<Key, usize>,
    ) -> impl Iterator<Item = (Key, &'a str)> {
        map.keys()
            .map(|key| (*key, self.progs[map[key]].name.as_str()))
    }

    fn bind_string_to_prog_index(&mut self, string: String, key: Key, prog_index: usize) {
        if self.string_map.contains_key(&string) {
            self.string_map[&string].insert(key, prog_index);
        } else {
            let mut key_map = OrderedMap::<Key, usize>::new();
            key_map.insert(key, prog_index);
            self.string_map.insert(string, key_map);
        }
    }

    fn add_prog(&mut self, prog: KeyProg) -> usize {
        let index = self.progs.len();
        self.progs.push(prog);
        index
    }
}
