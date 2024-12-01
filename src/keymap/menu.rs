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
    selection: Option<MenuSelection>,
}

/// The state of a menu's candidate selection.
struct MenuSelection {
    custom_candidate: Option<Candidate>,
    candidates: Vec<Candidate>,
    filtered_candidates: Vec<Candidate>,
    input: String,
    index: usize,
    default_to_custom_candidate: bool,
}

impl MenuSelection {
    fn new(keymap: &Keymap, default_to_custom_candidate: bool) -> MenuSelection {
        let custom_candidate = keymap.has_custom_candidate().then(Candidate::new_custom);
        let candidates = keymap.candidates().collect::<Vec<_>>();
        let mut menu = MenuSelection {
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

    fn execute(&mut self, cmd: MenuSelectionCmd) {
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

impl Menu {
    pub fn new(
        name: MenuName,
        description: String,
        keymap: Keymap,
        is_candidate_menu: bool,
        default_to_custom_candidate: bool,
    ) -> Menu {
        let selection = if is_candidate_menu {
            Some(MenuSelection::new(&keymap, default_to_custom_candidate))
        } else {
            None
        };
        Menu {
            name,
            description,
            keymap,
            selection,
        }
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    #[must_use]
    pub fn execute(&mut self, cmd: MenuSelectionCmd) -> bool {
        if let Some(selection) = &mut self.selection {
            selection.execute(cmd);
            true
        } else {
            false
        }
    }

    pub fn lookup(&self, key: Key) -> Option<KeyProg> {
        self.keymap.lookup(key, self.selected_candidate())
    }

    pub fn make_candidate_selection_doc(&self, s: &mut Storage) -> Option<Node> {
        self.selection
            .as_ref()
            .map(|selection| selection.make_candidate_selection_doc(s))
    }

    pub fn make_keyhint_doc(&self, s: &mut Storage) -> Node {
        self.keymap.make_keyhint_doc(s, self.selected_candidate())
    }

    fn selected_candidate(&self) -> Option<&Candidate> {
        self.selection
            .as_ref()
            .and_then(|selection| selection.selected_candidate())
    }
}
