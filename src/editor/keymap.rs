use super::stack::{Op, Prog, Value};
use crate::frontends::Key;
use crate::util::{bug, OrderedMap, SynlessBug};
use std::borrow::Borrow;
use std::collections::HashMap;

/// Key bindings.
///
/// All methods that add bindings will overwrite existing bindings.
#[derive(Debug, Clone)]
pub struct Keymap {
    /// If the user types `Key`, execute `KeyProg`.
    general_bindings: OrderedMap<Key, KeyProg>,
    /// If the user types `Key` while `String` is selected, execute `KeyProg`.
    non_literal_bindings: OrderedMap<String, OrderedMap<Key, KeyProg>>,
    /// If the user types `Key` while any of `literals` is selected, push the literal candidate's
    /// `Value` and then execute `KeyProg`.
    literal_bindings: OrderedMap<Key, KeyProg>,
    /// The set of literal candidates. Each has a display name and a value to push.
    // TODO: Literal insertion is quadratic. Make an efficient OrderedSet instead.
    literals: Vec<(String, Value)>,
    /// If the user types `Key` while a custom candidate is selected, push its string and execute
    /// `KeyProg`.
    custom_bindings: OrderedMap<Key, KeyProg>,
}

// TODO: doc
#[derive(Debug, Clone, Copy)]
pub enum Candidate<'a> {
    /// A candidate created from [`bind_key_for_candidate`].
    NonLiteral { display: &'a str },
    /// A candidate created from [`add_candidate`].
    Literal { display: &'a str, value: &'a Value },
    /// A new candidate created from the custom string the user typed.
    Custom { input: &'a str },
}

/// A named program that can be used in a key binding.
/// Represents the regular program that:
///
/// 1. If `exit_menu`, exits the menu.
/// 2. If `push_candidate_value`, pushes the selected candidate onto the stack.
/// 3. Executes `prog`.
#[derive(Debug, Clone)]
struct KeyProg {
    name: String,
    exit_menu: bool,
    push_candidate_value: bool,
    prog: Prog,
}

impl KeyProg {
    fn to_program(&self, candidate: Option<Candidate>) -> Prog {
        let mut prog = self.prog.to_owned();
        if self.push_candidate_value {
            // push_candidate_value is only set for literal&custom, so there must be a candidate
            let candidate = candidate.bug_msg("Keymap: push w/o candidate");
            let candidate_value = match candidate {
                Candidate::Literal { value, .. } => value.to_owned(),
                Candidate::Custom { input } => Value::String(input.to_owned()),
                // push_candidate_value is never set for a non_literal binding
                Candidate::NonLiteral { .. } => bug!("Keymap: NonLiteral w/ push"),
            };
            prog.insert_first(Op::Literal(candidate_value));
        }
        if self.exit_menu {
            prog.insert_first(Op::ExitMenu);
        } else {
            prog.insert_last(Op::Block);
        }
        prog
    }
}

impl Keymap {
    pub fn new() -> Keymap {
        Keymap {
            general_bindings: OrderedMap::new(),
            non_literal_bindings: OrderedMap::new(),
            literal_bindings: OrderedMap::new(),
            literals: Vec::new(),
            custom_bindings: OrderedMap::new(),
        }
    }

    /// Take the union of the two keymaps, with bindings in `other` overriding those in `self`.
    pub fn append(&mut self, other: Keymap) {
        // general_bindings
        self.general_bindings.append(other.general_bindings);

        // non_literal_bindings
        for (non_literal, map) in other.non_literal_bindings {
            if let Some(self_map) = self.non_literal_bindings.get_mut(&non_literal) {
                self_map.append(map);
            } else {
                self.non_literal_bindings.insert(non_literal, map);
            }
        }

        // literal_bindings
        self.literal_bindings.append(other.literal_bindings);

        // literals
        for (display, value) in other.literals {
            self.add_literal_candidate(display, value);
        }

        // custom_bindings
        self.custom_bindings.append(other.custom_bindings);
    }

    /// If the user types `key`, execute `prog`, potentially after exiting the menu.
    /// Use `name` when displaying this binding.
    pub fn bind_key(&mut self, key: Key, name: String, prog: Prog, exit_menu: bool) {
        self.general_bindings.insert(
            key,
            KeyProg {
                name,
                exit_menu,
                push_candidate_value: false,
                prog,
            },
        );
    }

    /// If the user types `key` while `candidate` is selected, execute `prog`,
    /// potentially after exiting the menu. Use `name` when displaying this binding.
    pub fn bind_key_for_candidate(
        &mut self,
        key: Key,
        candidate: String,
        name: String,
        prog: Prog,
        exit_menu: bool,
    ) {
        let key_prog = KeyProg {
            name,
            exit_menu,
            push_candidate_value: false,
            prog,
        };
        if !self.non_literal_bindings.contains_key(&candidate) {
            self.non_literal_bindings
                .insert(candidate.to_owned(), OrderedMap::new());
        }
        self.non_literal_bindings[&candidate].insert(key, key_prog);
    }

    /// Add a candidate to the list of literal candidates (used together with
    /// [`bind_key_for_literal_candidates`]).
    pub fn add_literal_candidate(&mut self, display: String, value: Value) {
        for (existing_display, existing_value) in &mut self.literals {
            if existing_display == &display {
                *existing_value = value;
                return;
            }
        }
        self.literals.push((display, value));
    }

    /// If the user types `Key` while one of the literal candidates is selected, push that
    /// candidate and then execute `prog`, potentially after exiting the menu. Use `name` when
    /// displaying this binding.
    pub fn bind_key_for_literal_candidates(
        &mut self,
        key: Key,
        name: String,
        prog: Prog,
        exit_menu: bool,
    ) {
        self.literal_bindings.insert(
            key,
            KeyProg {
                name,
                exit_menu,
                push_candidate_value: true,
                prog,
            },
        );
    }

    /// If the user types `key` while a custom candidate is selected, push its string and execute
    /// `prog`, potentially after exiting the menu. Use `name` when displaying this binding.
    pub fn bind_key_for_custom_candidates(
        &mut self,
        key: Key,
        name: String,
        prog: Prog,
        exit_menu: bool,
    ) {
        self.custom_bindings.insert(
            key,
            KeyProg {
                name,
                exit_menu,
                push_candidate_value: true,
                prog,
            },
        );
    }

    /// Whether this set of keymaps contains any candidates. (First keymap takes priority.)
    pub fn has_candidates(&self) -> bool {
        self.filtered_candidates("").next().is_some()
    }

    // TODO: Implement fuzzy search
    /// Iterates over all candidates that match the pattern, in the order they should be
    /// displayed.
    pub fn filtered_candidates<'a>(
        &'a self,
        pattern: &'a str,
    ) -> impl Iterator<Item = Candidate<'a>> {
        let custom_iter = {
            let has_custom = !self.custom_bindings.is_empty();
            let custom = if has_custom {
                Some(Candidate::Custom { input: pattern })
            } else {
                None
            };
            custom.into_iter()
        };

        let literal_iter = self.literals.iter().filter_map(move |(display, value)| {
            if display.contains(pattern) {
                Some(Candidate::Literal { display, value })
            } else {
                None
            }
        });

        let non_literal_iter = self.non_literal_bindings.keys().filter_map(move |display| {
            if display.contains(pattern) {
                Some(Candidate::NonLiteral { display })
            } else {
                None
            }
        });

        custom_iter.chain(literal_iter).chain(non_literal_iter)
    }

    /// Returns the program to execute if `key` is pressed while `candidate` is selected.
    pub fn lookup(&self, key: Key, candidate: Option<Candidate>) -> Option<Prog> {
        for (bound_key, keyprog) in self.available_keys_impl(candidate) {
            if bound_key == key {
                return Some(keyprog.to_program(candidate));
            }
        }
        None
    }

    /// Iterates over all `(key, name)` pairs that are available, given that `candidate` is selected.
    pub fn available_keys<'a>(
        &'a self,
        candidate: Option<Candidate<'a>>,
    ) -> impl Iterator<Item = (Key, &'a str)> {
        self.available_keys_impl(candidate)
            .map(|(key, keyprog)| (key, keyprog.name.as_ref()))
    }

    // General key bindings are _lowest_ priority.
    fn available_keys_impl<'a>(
        &'a self,
        candidate: Option<Candidate<'a>>,
    ) -> impl Iterator<Item = (Key, &'a KeyProg)> {
        let candidate_iter = candidate.into_iter().flat_map(|candidate| {
            let ordered_map = match candidate {
                Candidate::NonLiteral { display } => &self.non_literal_bindings[display],
                Candidate::Literal { .. } => &self.literal_bindings,
                Candidate::Custom { .. } => &self.custom_bindings,
            };
            ordered_map
                .into_iter()
                .map(|(key, keyprog)| (*key, keyprog))
        });

        let general_iter = self
            .general_bindings
            .iter()
            .map(|(key, keyprog)| (*key, keyprog));

        candidate_iter.chain(general_iter)
    }
}
