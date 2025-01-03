use super::keymap::{Candidate, KeyProg, Keymap};
use crate::frontends::Key;
use crate::language::Storage;
use crate::tree::Node;
use crate::util::{bug_assert, fuzzy_search, SynlessBug};

const SELECTION_LANGUAGE_NAME: &str = "selection_menu";

pub type MenuName = String;

/// A command that manipulates the menu's candidate selection.
pub enum MenuSelectionCmd {
    Up,
    Down,
    Backspace,
    Insert(char),
}

/// An open menu. Keeps track of the state of its candidate selection.
pub struct Menu {
    name: MenuName,
    description: String,
    keymap: Keymap,
    state: MenuState,
}

#[derive(Debug, Clone)]
pub enum MenuKind {
    Char,
    Candidate { default_to_custom_candidate: bool },
    InputString,
}

enum MenuState {
    Char,
    Candidate(CandidateMenu),
    InputString(InputStringMenu),
}

struct InputStringMenu {
    input: String,
}

struct CandidateMenu {
    custom_candidate: Option<Candidate>,
    candidates: Vec<Candidate>,
    filtered_candidates: Vec<Candidate>,
    input: String,
    index: usize,
    default_to_custom_candidate: bool,
}

impl Menu {
    pub fn new(name: MenuName, description: String, keymap: Keymap, kind: MenuKind) -> Menu {
        let state = match kind {
            MenuKind::Char => MenuState::Char,
            MenuKind::Candidate {
                default_to_custom_candidate,
            } => MenuState::Candidate(CandidateMenu::new(&keymap, default_to_custom_candidate)),
            MenuKind::InputString => MenuState::InputString(InputStringMenu::new()),
        };
        Menu {
            name,
            description,
            keymap,
            state,
        }
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns true if this kind of menu can (ever) execute that command
    #[must_use]
    pub fn execute(&mut self, cmd: MenuSelectionCmd) -> bool {
        match &mut self.state {
            MenuState::Char => false,
            MenuState::Candidate(menu) => menu.execute(cmd),
            MenuState::InputString(menu) => menu.execute(cmd),
        }
    }

    pub fn lookup(&self, key: Key) -> Option<KeyProg> {
        self.keymap.lookup(key, self.selected_candidate().as_ref())
    }

    pub fn make_candidate_selection_doc(&self, s: &mut Storage) -> Option<Node> {
        match &self.state {
            MenuState::Char => None,
            MenuState::Candidate(menu) => Some(menu.make_candidate_selection_doc(s)),
            MenuState::InputString(menu) => Some(menu.make_candidate_selection_doc(s)),
        }
    }

    pub fn make_keyhint_doc(&self, s: &mut Storage) -> Node {
        self.keymap
            .make_keyhint_doc(s, self.selected_candidate().as_ref())
    }

    fn selected_candidate(&self) -> Option<Candidate> {
        match &self.state {
            MenuState::Char => None,
            MenuState::Candidate(menu) => menu.selected_candidate().cloned(),
            MenuState::InputString(menu) => Some(Candidate::Custom {
                input: menu.input.clone(),
            }),
        }
    }
}

impl InputStringMenu {
    fn new() -> InputStringMenu {
        InputStringMenu {
            input: String::new(),
        }
    }

    #[must_use]
    fn execute(&mut self, cmd: MenuSelectionCmd) -> bool {
        use MenuSelectionCmd::{Backspace, Down, Insert, Up};

        match cmd {
            Up => false,
            Down => false,
            Backspace => {
                self.input.pop();
                true
            }
            Insert(ch) => {
                self.input.push(ch);
                true
            }
        }
    }

    fn make_candidate_selection_doc(&self, s: &mut Storage) -> Node {
        let lang = s
            .language(SELECTION_LANGUAGE_NAME)
            .bug_msg("Missing selection menu lang");
        let c_root = lang.root_construct(s);
        let c_input = lang.construct(s, "Input").bug();

        let root = Node::new(s, c_root);
        let input_node = Node::with_text(s, c_input, self.input.clone()).bug();
        bug_assert!(root.insert_last_child(s, input_node));
        root
    }
}

impl CandidateMenu {
    fn new(keymap: &Keymap, default_to_custom_candidate: bool) -> CandidateMenu {
        let custom_candidate = keymap.has_custom_candidate().then(Candidate::new_custom);
        let candidates = keymap.candidates().collect::<Vec<_>>();
        let mut menu = CandidateMenu {
            custom_candidate,
            candidates,
            filtered_candidates: Vec::new(),
            input: String::new(),
            index: 0, // about to be updated
            default_to_custom_candidate,
        };
        menu.update_filtered_candidates();
        menu
    }

    #[must_use]
    fn execute(&mut self, cmd: MenuSelectionCmd) -> bool {
        use MenuSelectionCmd::{Backspace, Down, Insert, Up};

        match cmd {
            Up => self.index = self.index.saturating_sub(1),
            Down => {
                if self.index + 1 < self.filtered_candidates.len() {
                    self.index += 1;
                }
            }
            Backspace => {
                self.input.pop();
                if let Some(Candidate::Custom { input }) = &mut self.custom_candidate {
                    input.pop();
                }
                self.update_filtered_candidates();
            }
            Insert(ch) => {
                self.input.push(ch);
                if let Some(Candidate::Custom { input }) = &mut self.custom_candidate {
                    input.push(ch);
                }
                self.update_filtered_candidates();
            }
        }
        true
    }

    fn update_filtered_candidates(&mut self) {
        self.filtered_candidates =
            fuzzy_search(&self.input, self.candidates.clone(), |candidate| {
                candidate.display_str()
            });
        let is_exact_match = self
            .filtered_candidates
            .first()
            .map(|c| c.display_str() == self.input)
            .unwrap_or(false);
        let has_regular_candidates = !self.filtered_candidates.is_empty();
        if let Some(candidate) = &self.custom_candidate {
            self.filtered_candidates.insert(0, candidate.to_owned());
        }

        self.index = if self.custom_candidate.is_some()
            && has_regular_candidates
            && (is_exact_match || !self.default_to_custom_candidate)
        {
            1
        } else {
            0
        };
    }

    fn selected_candidate(&self) -> Option<&Candidate> {
        self.filtered_candidates.get(self.index)
    }

    fn make_candidate_selection_doc(&self, s: &mut Storage) -> Node {
        use Candidate::{Custom, Regular, Special};

        // Lookup selection menu language and constructs
        let lang = s
            .language(SELECTION_LANGUAGE_NAME)
            .bug_msg("Missing selection menu lang");
        let c_root = lang.root_construct(s);
        let c_input = lang.construct(s, "Input").bug();
        let c_selected = lang.construct(s, "Selected").bug();
        let c_custom = lang.construct(s, "Custom").bug();
        let c_regular = lang.construct(s, "Regular").bug();
        let c_special = lang.construct(s, "Special").bug();

        // Construct root node
        let root = Node::new(s, c_root);

        // Add input entry
        let input_node = Node::with_text(s, c_input, self.input.clone()).bug();
        bug_assert!(root.insert_last_child(s, input_node));

        // Add candidate entries, highlighting the one at self.index
        for (i, candidate) in self.filtered_candidates.iter().enumerate() {
            let construct = match candidate {
                Custom { .. } => c_custom,
                Regular { .. } => c_regular,
                Special { .. } => c_special,
            };
            let mut node = Node::with_text(s, construct, candidate.display_str().to_owned()).bug();
            if i == self.index {
                node = Node::with_children(s, c_selected, [node]).bug();
            }
            bug_assert!(root.insert_last_child(s, node));
        }

        root
    }
}
