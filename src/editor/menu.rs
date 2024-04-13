use super::keymap::{Candidate, Keymap};
use super::stack::Prog;
use crate::frontends::Key;
use crate::language::Storage;
use crate::tree::Node;
use crate::util::{bug_assert, SynlessBug};

const SELECTION_LANGUAGE_NAME: &str = "SelectionMenu";

pub type MenuName = String;

pub enum MenuSelectionCmd {
    Up,
    Down,
    Backspace,
    Insert(char),
}

pub struct Menu {
    name: MenuName,
    keymap: Keymap,
    selection: Option<MenuSelection>,
}

struct MenuSelection {
    custom_candidate: Option<Candidate>,
    candidates: Vec<Candidate>,
    input: String,
    index: usize,
    default_index: usize,
}

impl MenuSelection {
    fn new(keymap: &Keymap) -> Option<MenuSelection> {
        let custom_candidate = keymap.custom_candidate();
        let candidates = keymap.candidates().collect::<Vec<_>>();
        if candidates.is_empty() && custom_candidate.is_none() {
            return None;
        }
        let default_index = if custom_candidate.is_some() { 1 } else { 0 };
        Some(MenuSelection {
            custom_candidate,
            candidates,
            input: String::new(),
            index: default_index,
            default_index,
        })
    }

    fn execute(&mut self, cmd: MenuSelectionCmd) {
        use MenuSelectionCmd::{Backspace, Down, Insert, Up};

        match cmd {
            Up => self.index = self.index.saturating_sub(1),
            Down => self.index += 1,
            Backspace => {
                self.input.pop();
                if let Some(Candidate::Custom { input }) = &mut self.custom_candidate {
                    input.pop();
                }
                self.index = self.default_index;
            }
            Insert(ch) => {
                self.input.push(ch);
                if let Some(Candidate::Custom { input }) = &mut self.custom_candidate {
                    input.push(ch);
                }
                self.index = self.default_index;
            }
        }
    }

    // TODO: Implement fuzzy search
    fn filtered_candidates(&self) -> Vec<&Candidate> {
        let mut filtered = Vec::new();
        if let Some(candidate) = &self.custom_candidate {
            filtered.push(candidate);
        }
        for candidate in &self.candidates {
            if candidate.display_str().contains(&self.input) {
                filtered.push(candidate);
            }
        }
        filtered
    }

    fn make_selection_doc(&self, s: &mut Storage) -> Node {
        use Candidate::{Custom, Literal, NonLiteral};

        // Lookup SelectionMenu language and constructs
        let lang = s
            .language(&SELECTION_LANGUAGE_NAME)
            .bug_msg("Missing SelectionMenu lang");
        let c_root = lang.root_construct(s);
        let c_input = lang.construct(s, "Input").bug();
        let c_selected = lang.construct(s, "Selected").bug();
        let c_custom = lang.construct(s, "Custom").bug();
        let c_literal = lang.construct(s, "Literal").bug();
        let c_non_literal = lang.construct(s, "NonLiteral").bug();

        // Construct root node
        let root = Node::new(s, c_root);

        // Add input entry
        let input_node = Node::with_text(s, c_input, self.input.clone()).bug();
        bug_assert!(root.insert_last_child(s, input_node));

        // Add candidate entries, highlighting the one at self.index
        let candidates = self.filtered_candidates();
        let index = self.index.min(candidates.len().saturating_sub(1));
        for (i, candidate) in candidates.iter().enumerate() {
            let construct = match candidate {
                Custom { .. } => c_custom,
                Literal { .. } => c_literal,
                NonLiteral { .. } => c_non_literal,
            };
            let mut node = Node::with_text(s, construct, candidate.display_str().to_owned()).bug();
            if i == index {
                node = Node::with_children(s, c_selected, [node]).bug();
            }
            bug_assert!(root.insert_last_child(s, node));
        }

        root
    }
}

impl Menu {
    pub fn new(name: MenuName, keymap: Keymap) -> Menu {
        Menu {
            name,
            selection: MenuSelection::new(&keymap),
            keymap,
        }
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

    pub fn lookup(&self, key: Key) -> Option<Prog> {
        self.keymap.lookup(key, self.selected_candidate())
    }

    pub fn make_selection_doc(&self, s: &mut Storage) -> Option<Node> {
        self.selection
            .as_ref()
            .map(|selection| selection.make_selection_doc(s))
    }

    pub fn make_keyhint_doc(&self, s: &mut Storage) -> Node {
        self.keymap.make_keyhint_doc(s, self.selected_candidate())
    }

    fn selected_candidate(&self) -> Option<&Candidate> {
        if let Some(selection) = &self.selection {
            let candidates = selection.filtered_candidates();
            let index = selection.index.min(candidates.len().saturating_sub(1));
            candidates.get(index).copied()
        } else {
            None
        }
    }
}
