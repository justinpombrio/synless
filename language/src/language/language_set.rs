//! An editable language.

use super::construct::{Arity, Construct, Sort, SortId};
use partial_pretty_printer::Notation;
use std::collections::HashMap;
use std::default::Default;
use std::iter::Iterator;
use utility::spanic;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstructId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LanguageId(u32);

pub struct LanguageSet {
    /// LanguageId -> Language
    languages: Vec<Language>,
    languages_by_name: HashMap<String, LanguageId>,
}

pub struct Grammar {
    /// SortId -> Sort
    sorts: Vec<Sort>,
    /// ConstructId -> Construct
    constructs: Vec<Construct>,
    /// SortId -> Vec<ConstructId>
    constructs_of_sort: Vec<Vec<ConstructId>>,
    keymap: HashMap<char, ConstructId>,
}

pub struct Language {
    name: String,
    grammar: &'static Grammar,
    current_notation_set: &'static NotationSet,
    alternative_notation_sets: HashMap<String, &'static NotationSet>,
}

pub struct NotationConfig {
    name: String,
    notations: Vec<(String, Notation)>,
}

pub struct NotationSet {
    name: String,
    /// Construct id -> Notation
    notations: Vec<Notation>,
}

impl Default for LanguageSet {
    fn default() -> LanguageSet {
        LanguageSet::new()
    }
}

impl LanguageSet {
    pub fn new() -> LanguageSet {
        LanguageSet {
            languages: vec![],
            languages_by_name: HashMap::new(),
        }
    }

    pub fn add_language(&mut self, language: Language) -> LanguageId {
        let language_id = LanguageId(self.languages.len() as u32);
        self.languages_by_name
            .insert(language.name.clone(), language_id);
        self.languages.push(language);
        language_id
    }
}

impl Language {
    pub fn new(name: String, grammar: Grammar, default_notation_set: NotationConfig) -> Language {
        let notation_set = NotationSet::new(
            default_notation_set.name,
            &grammar,
            default_notation_set.notations,
        );
        let notation_set: &'static NotationSet = Box::leak(Box::new(notation_set));
        let mut alternative_notation_sets = HashMap::new();
        alternative_notation_sets.insert(notation_set.name.clone(), notation_set);
        Language {
            name,
            grammar: Box::leak(Box::new(grammar)),
            current_notation_set: notation_set,
            alternative_notation_sets,
        }
    }

    pub fn add_notation_set(&mut self, notation_set: NotationConfig) {
        let notation_set =
            NotationSet::new(notation_set.name, &self.grammar, notation_set.notations);
        let notation_set: &'static NotationSet = Box::leak(Box::new(notation_set));
        self.alternative_notation_sets
            .insert(notation_set.name.to_owned(), notation_set);
    }

    pub fn grammar(&self) -> &'static Grammar {
        self.grammar
    }

    pub fn current_notation_set(&self) -> &'static NotationSet {
        self.current_notation_set
    }

    pub fn alternative_notation_sets(&self) -> impl Iterator<Item = &NotationSet> {
        self.alternative_notation_sets.values().copied()
    }

    pub fn switch_notation_set(&mut self, notation_set_name: &str) {
        self.current_notation_set = self.alternative_notation_sets[notation_set_name]
    }
}

impl NotationSet {
    pub fn new(name: String, grammar: &Grammar, notations: Vec<(String, Notation)>) -> NotationSet {
        let mut notations_map = notations.into_iter().collect::<HashMap<_, _>>();
        let notations = grammar
            .constructs
            .iter()
            .map(|con| notations_map.remove(&con.name).unwrap())
            .collect::<Vec<_>>();
        NotationSet { name, notations }
    }

    pub fn lookup(&self, construct_id: ConstructId) -> &Notation {
        &self.notations[construct_id.0 as usize]
    }
}

impl Default for Grammar {
    fn default() -> Grammar {
        Grammar::new()
    }
}

impl Grammar {
    pub fn new() -> Grammar {
        Grammar {
            sorts: vec![],
            constructs: vec![],
            constructs_of_sort: vec![],
            keymap: HashMap::new(),
        }
    }

    pub fn lookup_key(&self, key: char) -> Option<&Construct> {
        Some(&self.constructs[self.keymap.get(&key)?.0 as usize])
    }

    pub fn lookup_construct(&self, construct_id: ConstructId) -> &Construct {
        &self.constructs[construct_id.0 as usize]
    }

    pub fn keymap(&self) -> impl Iterator<Item = (char, &str)> {
        self.keymap
            .iter()
            .map(move |(ch, con)| (*ch, self.lookup_construct(*con).name.as_ref()))
    }

    pub fn constructs(&self) -> impl Iterator<Item = &Construct> {
        self.constructs.iter()
    }

    fn add_sort(&mut self, sort: Sort) -> SortId {
        if let Some(sort_id) = self.sorts.iter().position(|s| s == &sort) {
            SortId(sort_id as u32)
        } else {
            let sort_id = SortId(self.sorts.len() as u32);
            self.sorts.push(sort);
            self.constructs_of_sort.push(vec![]);
            sort_id
        }
    }

    pub fn add_construct(&mut self, name: String, sort: Sort, arity: Arity, key: Option<char>) {
        // Add the sort
        let sort_id = self.add_sort(sort);

        // Add the construct
        let construct = Construct {
            name,
            sort_id,
            arity,
            key,
        };
        let construct_id = ConstructId(self.constructs.len() as u32);
        self.constructs.push(construct);

        // Extend the keymap
        if let Some(key) = key {
            let duplicate = self.keymap.insert(key, construct_id);
            if duplicate.is_some() {
                spanic!("Duplicate key '{}'", key);
            }
        }

        // Extend the construct list for the sort
        let cons_list = &mut self.constructs_of_sort[sort_id.0 as usize];
        if !cons_list.contains(&construct_id) {
            cons_list.push(construct_id);
        }
    }
}
