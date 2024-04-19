#![allow(clippy::module_inception)]

use crate::frontends::Key;
use crate::language::Storage;
use crate::tree::Node;
use crate::util::{bug, bug_assert, error, OrderedMap, SynlessBug};
use std::borrow::Borrow;
use std::collections::HashMap;

const KEYHINTS_LANGUAGE_NAME: &str = "keyhints";

/*************
 * Candidate *
 *************/

/// An entry in a [`Keymap`]'s candidate list.
#[derive(Debug, Clone)]
pub enum Candidate {
    /// A candidate created from [`bind_key_for_special_candidate`].
    Special { display: String },
    /// A candidate created from [`add_regular_candidate`].
    Regular {
        display: String,
        value: rhai::Dynamic,
    },
    /// A new candidate created from the custom string the user typed.
    Custom { input: String },
}

impl Candidate {
    pub fn new_custom() -> Candidate {
        Candidate::Custom {
            input: String::new(),
        }
    }

    fn new_regular(display: &str, value: &rhai::Dynamic) -> Candidate {
        Candidate::Regular {
            display: display.to_owned(),
            value: value.to_owned(),
        }
    }

    fn new_special(display: &str) -> Candidate {
        Candidate::Special {
            display: display.to_owned(),
        }
    }

    pub fn display_str(&self) -> &str {
        use Candidate::{Custom, Regular, Special};

        match self {
            Special { display } => display,
            Regular { display, .. } => display,
            Custom { input } => input,
        }
    }

    fn value(&self) -> Option<rhai::Dynamic> {
        use Candidate::{Custom, Regular, Special};

        match self {
            Special { .. } => None,
            Regular { value, .. } => Some(value.clone()),
            Custom { input } => Some(rhai::Dynamic::from(input.to_owned())),
        }
    }
}

/***********
 * KeyProg *
 ***********/

/// A program that can be used in a key binding.
/// Represents the regular program that:
///
/// 1. If `close_menu`, exits the menu.
/// 2. If `Some(val) = candidate.value()`, pushes `val` onto the stack.
/// 3. Executes `prog`.
#[derive(Debug, Clone)]
struct KeyProgSpec {
    hint: String,
    close_menu: bool,
    prog: rhai::FnPtr,
}

#[derive(Debug, Clone)]
pub struct KeyProg {
    close_menu: bool,
    prog: rhai::FnPtr,
    value: Option<rhai::Dynamic>,
}

impl KeyProgSpec {
    // If this KeyProgSpec is from a general binding, `candidate` should be None.
    fn to_key_prog(&self, candidate: Option<&Candidate>) -> KeyProg {
        let value = candidate.and_then(|candidate| candidate.value());
        KeyProg {
            close_menu: self.close_menu,
            prog: self.prog.clone(),
            value,
        }
    }
}

impl rhai::CustomType for KeyProg {
    fn build(mut builder: rhai::TypeBuilder<Self>) {
        builder
            .with_name("KeyProg")
            .with_get_set(
                "close_menu",
                |kp: &mut KeyProg| -> bool { kp.close_menu },
                |kp: &mut KeyProg, close_menu: bool| kp.close_menu = close_menu,
            )
            .with_get_set(
                "prog",
                |kp: &mut KeyProg| -> rhai::FnPtr { kp.prog.clone() },
                |kp: &mut KeyProg, prog: rhai::FnPtr| kp.prog = prog,
            )
            .with_get_set(
                "value",
                |kp: &mut KeyProg| -> Option<rhai::Dynamic> { kp.value.clone() },
                |kp: &mut KeyProg, value: Option<rhai::Dynamic>| kp.value = value,
            );
    }
}

/**********
 * KeyMap *
 **********/

/// A keymap consists of one or two panes in the window. All keymaps have a "key hints" pane that
/// shows which keys can be pressed, with short hints describing what each key will do. Some
/// keymaps have an additional "selection" pane, where the user can select between candidate values
/// by typing a search pattern and using arrow keys.
///
/// To illustrate the types of candidates and bindings, we will describe how to implement a file
/// selection menu. It would look like:
///
/// ```text
///     +------------------+
///     | prompt>          |
///     | * [new file]     |
///  -> | * baz.rs         |
///     | * foobar.rs      |
///     | * ..             |
///     +------------------+
///     | enter: open      |
///     | ctrl-d: delete   |
///     | esc: exit menu   |
///     +------------------+
/// ```
///
/// This menu contains:
///
/// - A prompt where the user can enter text. It can be used to filter the candidates or to
/// create a new file with a custom name.
///
/// - A list of candidates. This includes the name of each file in the directory, a special
/// candidate `..` that opens a new selection menu for the parent directory, and a custom candidate
/// that creates a new file with whatever name was entered at the prompt. There is always one
/// selected candidate, shown here with `->`.
///
/// - A list of key hints, showing which keys can be pressed and what they will do. These change
/// depending on which candidate is currently selected. For example, normal files can be "opened" or
/// "deleted", `..` can be "opened", and the custom candidate can be "created".
///
/// If the user types "foo" and presses the up arrow, the candidates list will be filtered, the
/// selection will move to the custom "[new file] foo" candidate, and the key hints will be updated
/// to reflect that selection:
///
/// ```text
///     +------------------+
///     | prompt> foo      |
///  -> | * [new file] foo |
///     | * foobar.rs      |
///     +------------------+
///     |  enter: create   |
///     |  esc: exit menu  |
///     +------------------+
/// ```
///
/// Many keymaps don't need candidate selection. These will only contain _general bindings_, added
/// with the method [`add_binding()`]. This method binds a key to a function that takes no
/// arguments. If you only add general bindings, then no prompt or candidate list will be shown.
///
/// For keymaps with candidate selection, there are three kinds of candidates:
///
/// - _The custom candidate._ The method [`bind_key_for_custom_candidate()`] binds a key to a
/// function that takes the user's input string as an argument. The custom candidate is only shown
/// in the list if there is at least one binding for it. In the file example, the entry prefixed
/// with `[new file]` is a custom candidate.
///
/// - _Regular candidates._ The method [`add_regular_candidate()`] adds a regular candidate to the
/// candidate list. This candidate has both a display string and a _value_. The method
/// [`bind_key_for_regular_candidates()`] binds a key to a function that takes the selected
/// candidate's value as an argument. Each such binding applies to _all_ regular candidates. In
/// the file example, the file names "baz.rs" and "foobar.rs" are regular candidates and "enter" is
/// bound to "open file by name" for both of them.
///
/// - _Special candidates._ The method [`bind_key_for_special_candidate()`] adds a special
/// candidate to the candidate list, and gives it a binding from a key to a function that takes no
/// arguments. You can call it more than once for the same special candidate to give it multiple
/// bindings. In the file example, `..` is a special candidate, for which "enter" is bound to "open
/// menu for parent directory".
///
/// You can have both candidate selection and general bindings in one keymap. In the file example,
/// "esc" is bound to "exit menu" with a general binding. The key hints pane shows the key bindings
/// for the selected candidate, plus all general key bindings.
///
/// ### Conflict Resolution
///
/// - If you add multiple general bindings for the same key, the latest binding overrides the
/// previous ones.
///
/// - If you add multiple candidate-specific bindings for the same key and candidate, the latest binding
/// overrides the previous ones.
///
/// - If you add a general binding and candidate-specific binding with the same key, the
/// candidate-specific binding takes priority.
#[derive(Debug, Clone, Default)]
pub struct Keymap {
    /// If the user types `Key`, execute `KeyProgSpec`.
    general_bindings: OrderedMap<Key, KeyProgSpec>,
    /// If the user types `Key` while `String` is selected, execute `KeyProgSpec`.
    special_bindings: OrderedMap<String, OrderedMap<Key, KeyProgSpec>>,
    /// If the user types `Key` while any of `regular_candidates` is selected, invoke `KeyProgSpec`
    /// with the regular candidate's `rhai::Dynamic`.
    regular_bindings: OrderedMap<Key, KeyProgSpec>,
    /// The set of regular candidates. Each has a display label and a value.
    // TODO: Regular candidate insertion is quadratic. Make an efficient OrderedSet instead.
    regular_candidates: Vec<(String, rhai::Dynamic)>,
    /// If the user types `Key` while a custom candidate is selected, invoke `KeyProgSpec` with the
    /// user's input string.
    custom_bindings: OrderedMap<Key, KeyProgSpec>,
}

impl Keymap {
    /****************
     * Constructors *
     ****************/

    pub fn new() -> Keymap {
        Keymap::default()
    }

    /// Take the union of the two keymaps, with bindings in `other` overriding those in `self`.
    pub fn append(&mut self, other: Keymap) {
        // general_bindings
        self.general_bindings.append(other.general_bindings);

        // special_bindings
        for (special, map) in other.special_bindings {
            if let Some(self_map) = self.special_bindings.get_mut(&special) {
                self_map.append(map);
            } else {
                self.special_bindings.insert(special, map);
            }
        }

        // regular_bindings
        self.regular_bindings.append(other.regular_bindings);

        // regular candidatess
        for (display, value) in other.regular_candidates {
            self.add_regular_candidate(display, value);
        }

        // custom_bindings
        self.custom_bindings.append(other.custom_bindings);
    }

    /****************
     * Binding Keys *
     ****************/

    /// Add a general binding: if the user types `key`, execute `prog`, potentially after exiting
    /// the menu.  Use `hint` when displaying this binding.
    pub fn bind_key(&mut self, key: Key, hint: String, prog: rhai::FnPtr, close_menu: bool) {
        self.general_bindings.insert(
            key,
            KeyProgSpec {
                hint,
                close_menu,
                prog,
            },
        );
    }

    /// Add a special binding: if the user types `key` while `candidate` is selected, execute
    /// `prog`, potentially after exiting the menu. Use `hint` when displaying this binding.
    pub fn bind_key_for_special_candidate(
        &mut self,
        key: Key,
        candidate: String,
        hint: String,
        prog: rhai::FnPtr,
        close_menu: bool,
    ) {
        let key_prog = KeyProgSpec {
            hint,
            close_menu,
            prog,
        };
        if !self.special_bindings.contains_key(&candidate) {
            self.special_bindings
                .insert(candidate.to_owned(), OrderedMap::new());
        }
        self.special_bindings[&candidate].insert(key, key_prog);
    }

    /// Add a regular candidate to the list of candidates (used together with
    /// [`bind_key_for_regular_candidates`]).
    pub fn add_regular_candidate(&mut self, display: String, value: rhai::Dynamic) {
        for (existing_display, existing_value) in &mut self.regular_candidates {
            if existing_display == &display {
                *existing_value = value;
                return;
            }
        }
        self.regular_candidates.push((display, value));
    }

    /// If the user types `Key` while one of the regular candidates is selected, pass that
    /// candidate's value to `prog`, potentially after exiting the menu. Use `hint` when displaying
    /// this binding.
    pub fn bind_key_for_regular_candidates(
        &mut self,
        key: Key,
        hint: String,
        prog: rhai::FnPtr,
        close_menu: bool,
    ) {
        self.regular_bindings.insert(
            key,
            KeyProgSpec {
                hint,
                close_menu,
                prog,
            },
        );
    }

    /// If the user types `key` while the custom candidate is selected, pass the user's input
    /// string to `prog`, potentially after exiting the menu. Use `hint` when displaying this
    /// binding.
    pub fn bind_key_for_custom_candidate(
        &mut self,
        key: Key,
        hint: String,
        prog: rhai::FnPtr,
        close_menu: bool,
    ) {
        self.custom_bindings.insert(
            key,
            KeyProgSpec {
                hint,
                close_menu,
                prog,
            },
        );
    }

    /*************
     * Accessors *
     *************/

    pub fn has_custom_candidate(&self) -> bool {
        !self.custom_bindings.is_empty()
    }

    /// Constructs a sequence of all regular and special candidates (does not include the custom
    /// candidate).
    pub fn candidates(&self) -> impl Iterator<Item = Candidate> + '_ {
        let regular_iter = self
            .regular_candidates
            .iter()
            .map(|(display, value)| Candidate::new_regular(display, value));
        let special_iter = self
            .special_bindings
            .keys()
            .map(|display| Candidate::new_special(display));

        regular_iter.chain(special_iter)
    }

    /// Returns the program to execute if `key` is pressed while `candidate` is selected.
    pub fn lookup(&self, key: Key, mut candidate: Option<&Candidate>) -> Option<KeyProg> {
        for (bound_key, keyprog, use_candidate) in self.available_keys_impl(candidate) {
            if bound_key == key {
                if !use_candidate {
                    candidate = None;
                }
                return Some(keyprog.to_key_prog(candidate));
            }
        }
        None
    }

    /// Iterates over all `(key, hint)` pairs that are available, given that `candidate` is selected.
    pub fn available_keys(
        &self,
        candidate: Option<&Candidate>,
    ) -> impl Iterator<Item = (Key, &str)> + '_ {
        self.available_keys_impl(candidate)
            .map(|(key, keyprog, _)| (key, keyprog.hint.as_ref()))
    }

    // Returns iterator of (key, prog_to_run_if_pressed, use_candidate)
    fn available_keys_impl(
        &self,
        candidate: Option<&Candidate>,
    ) -> impl Iterator<Item = (Key, &KeyProgSpec, bool)> + '_ {
        use Candidate::{Custom, Regular, Special};

        let ordered_map_opt = candidate.map(|candidate| match candidate {
            Special { display } => &self.special_bindings[display],
            Regular { .. } => &self.regular_bindings,
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
            .bug_msg("Missing keyhints lang");
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

impl rhai::CustomType for Keymap {
    fn build(mut builder: rhai::TypeBuilder<Self>) {
        use std::str::FromStr;

        // TODO add the other bind methods
        builder
            .with_name("Keymap")
            .with_fn("new_keymap", Keymap::new)
            .with_fn(
                "bind_key",
                |keymap: &mut Keymap,
                 key_str: &str,
                 hint: String,
                 prog: rhai::FnPtr,
                 close_menu: bool|
                 -> Result<(), Box<rhai::EvalAltResult>> {
                    let key =
                        Key::from_str(key_str).map_err(|err| error!(Keymap, "{err}: {key_str}"))?;
                    keymap.bind_key(key, hint, prog, close_menu);
                    Ok(())
                },
            )
            .with_fn(
                "bind_key_for_special_candidate",
                |keymap: &mut Keymap,
                 key_str: &str,
                 candidate: String,
                 hint: String,
                 prog: rhai::FnPtr,
                 close_menu: bool|
                 -> Result<(), Box<rhai::EvalAltResult>> {
                    let key =
                        Key::from_str(key_str).map_err(|err| error!(Keymap, "{err}: {key_str}"))?;
                    keymap.bind_key_for_special_candidate(key, candidate, hint, prog, close_menu);
                    Ok(())
                },
            )
            .with_fn(
                "add_regular_candidate",
                |keymap: &mut Keymap, display: String, value: rhai::Dynamic| {
                    keymap.add_regular_candidate(display, value)
                },
            )
            .with_fn(
                "add_regular_candidate",
                |keymap: &mut Keymap, value: rhai::Dynamic| {
                    keymap.add_regular_candidate(value.to_string(), value)
                },
            )
            .with_fn(
                "bind_key_for_regular_candidates",
                |keymap: &mut Keymap,
                 key_str: &str,
                 hint: String,
                 prog: rhai::FnPtr,
                 close_menu: bool|
                 -> Result<(), Box<rhai::EvalAltResult>> {
                    let key =
                        Key::from_str(key_str).map_err(|err| error!(Keymap, "{err}: {key_str}"))?;
                    keymap.bind_key_for_regular_candidates(key, hint, prog, close_menu);
                    Ok(())
                },
            )
            .with_fn(
                "bind_key_for_custom_candidate",
                |keymap: &mut Keymap,
                 key_str: &str,
                 hint: String,
                 prog: rhai::FnPtr,
                 close_menu: bool|
                 -> Result<(), Box<rhai::EvalAltResult>> {
                    let key =
                        Key::from_str(key_str).map_err(|err| error!(Keymap, "{err}: {key_str}"))?;
                    keymap.bind_key_for_custom_candidate(key, hint, prog, close_menu);
                    Ok(())
                },
            );
    }
}
