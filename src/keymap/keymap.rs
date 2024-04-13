#![allow(clippy::module_inception)]

use crate::editor::{Op, Prog, Value};
use crate::frontends::Key;
use crate::language::Storage;
use crate::tree::Node;
use crate::util::{bug, bug_assert, OrderedMap, SynlessBug};
use std::borrow::Borrow;
use std::collections::HashMap;

const KEYHINTS_LANGUAGE_NAME: &str = "Keyhints";

/*************
 * Candidate *
 *************/

// TODO: doc
#[derive(Debug, Clone)]
pub enum Candidate {
    /// A candidate created from [`bind_key_for_candidate`].
    NonLiteral { display: String },
    /// A candidate created from [`add_candidate`].
    Literal { display: String, value: Value },
    /// A new candidate created from the custom string the user typed.
    Custom { input: String },
}

impl Candidate {
    fn new_custom() -> Candidate {
        Candidate::Custom {
            input: String::new(),
        }
    }

    fn new_literal(display: &str, value: &Value) -> Candidate {
        Candidate::Literal {
            display: display.to_owned(),
            value: value.to_owned(),
        }
    }

    fn new_non_literal(display: &str) -> Candidate {
        Candidate::NonLiteral {
            display: display.to_owned(),
        }
    }

    pub fn display_str(&self) -> &str {
        use Candidate::{Custom, Literal, NonLiteral};

        match self {
            NonLiteral { display } => display,
            Literal { display, .. } => display,
            Custom { input } => input,
        }
    }

    fn value(&self) -> Option<Value> {
        use Candidate::{Custom, Literal, NonLiteral};

        match self {
            NonLiteral { .. } => None,
            Literal { value, .. } => Some(value.clone()),
            Custom { input } => Some(Value::String(input.clone())),
        }
    }
}

/***********
 * KeyProg *
 ***********/

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
    prog: Prog,
}

impl KeyProg {
    fn to_program(&self, candidate: Option<&Candidate>) -> Prog {
        let mut prog = self.prog.to_owned();
        if let Some(value) = candidate.and_then(|candidate| candidate.value()) {
            prog.insert_first(Op::Literal(value));
        }
        if self.exit_menu {
            prog.insert_first(Op::ExitMenu);
        } else {
            prog.insert_last(Op::Block);
        }
        prog
    }
}

/**********
 * KeyMap *
 **********/

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

impl Keymap {
    /****************
     * Constructors *
     ****************/

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

    /****************
     * Binding Keys *
     ****************/

    /// If the user types `key`, execute `prog`, potentially after exiting the menu.
    /// Use `name` when displaying this binding.
    pub fn bind_key(&mut self, key: Key, name: String, prog: Prog, exit_menu: bool) {
        self.general_bindings.insert(
            key,
            KeyProg {
                name,
                exit_menu,
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
                prog,
            },
        );
    }

    /*************
     * Accessors *
     *************/

    pub fn custom_candidate(&self) -> Option<Candidate> {
        (!self.custom_bindings.is_empty()).then(Candidate::new_custom)
    }

    // all candidates except the custom candidate
    pub fn candidates(&self) -> impl Iterator<Item = Candidate> + '_ {
        let literal_iter = self
            .literals
            .iter()
            .map(|(display, value)| Candidate::new_literal(display, value));
        let non_literal_iter = self
            .non_literal_bindings
            .keys()
            .map(|display| Candidate::new_non_literal(display));

        literal_iter.chain(non_literal_iter)
    }

    /// Returns the program to execute if `key` is pressed while `candidate` is selected.
    pub fn lookup(&self, key: Key, mut candidate: Option<&Candidate>) -> Option<Prog> {
        for (bound_key, keyprog, use_candidate) in self.available_keys_impl(candidate) {
            if bound_key == key {
                if !use_candidate {
                    candidate = None;
                }
                return Some(keyprog.to_program(candidate));
            }
        }
        None
    }

    /// Iterates over all `(key, name)` pairs that are available, given that `candidate` is selected.
    pub fn available_keys(
        &self,
        candidate: Option<&Candidate>,
    ) -> impl Iterator<Item = (Key, &str)> + '_ {
        self.available_keys_impl(candidate)
            .map(|(key, keyprog, _)| (key, keyprog.name.as_ref()))
    }

    // General key bindings are _lowest_ priority.
    // Returns iterator of (key, prog_to_run_if_pressed, use_candidate)
    fn available_keys_impl(
        &self,
        candidate: Option<&Candidate>,
    ) -> impl Iterator<Item = (Key, &KeyProg, bool)> + '_ {
        use Candidate::{Custom, Literal, NonLiteral};

        let ordered_map_opt = candidate.map(|candidate| match candidate {
            NonLiteral { display } => &self.non_literal_bindings[display],
            Literal { .. } => &self.literal_bindings,
            Custom { .. } => &self.custom_bindings,
        });
        let candidate_iter = ordered_map_opt
            .into_iter()
            .flat_map(|map| map.into_iter().map(|(key, keyprog)| (*key, keyprog, true)));

        let general_iter = self
            .general_bindings
            .iter()
            .map(|(key, keyprog)| (*key, keyprog, false));

        candidate_iter.chain(general_iter)
    }

    /***************
     * KeyHint Doc *
     ***************/

    pub fn make_keyhint_doc(&self, s: &mut Storage, candidate: Option<&Candidate>) -> Node {
        // Lookup SelectionMenu language and constructs
        let lang = s
            .language(KEYHINTS_LANGUAGE_NAME)
            .bug_msg("Missing Keyhints lang");
        let c_root = lang.root_construct(s);
        let c_entry = lang.construct(s, "Entry").bug();
        let c_key = lang.construct(s, "Key").bug();
        let c_hint = lang.construct(s, "Hint").bug();

        // Construct root node
        let root = Node::new(s, c_root);

        // Add (key, hint) entries
        for (key, hint) in self.available_keys(candidate) {
            let key_node = Node::with_text(s, c_key, key.to_string()).bug();
            let hint_node = Node::with_text(s, c_hint, hint.to_owned()).bug();
            let entry_node = Node::with_children(s, c_entry, [key_node, hint_node]).bug();
            bug_assert!(root.insert_last_child(s, entry_node));
        }

        root
    }
}
