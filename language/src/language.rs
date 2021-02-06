//! An editable language.

use crate::construct::{Arity, Construct, Sort, SortId};
use std::collections::HashMap;
use std::fmt;
use std::iter::Iterator;
use utility::spanic;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstructId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LanguageId(u32);

pub struct Language {
    name: String,
    // SortId -> Sort
    sorts: Vec<Sort>,
    // ConstructId -> Construct
    constructs: Vec<Construct>,
    // SortId -> Vec<ConstructId>
    constructs_of_sort: Vec<Vec<ConstructId>>,
    keymap: HashMap<char, ConstructId>,
}

pub struct LanguageSet {
    // LanguageId -> Language
    languages: Vec<&'static Language>,
    languages_by_name: HashMap<String, LanguageId>,
}

impl LanguageSet {
    fn new() -> LanguageSet {
        LanguageSet {
            languages: vec![],
            languages_by_name: HashMap::new(),
        }
    }

    fn add_language(&mut self, language: Language) -> LanguageId {
        let language_id = LanguageId(self.languages.len() as u32);
        self.languages_by_name
            .insert(language.name.clone(), language_id);
        self.languages.push(Box::leak(Box::new(language)));
        language_id
    }
}

impl Language {
    pub fn new(name: String) -> Language {
        Language {
            name,
            sorts: vec![],
            constructs: vec![],
            constructs_of_sort: vec![],
            keymap: HashMap::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
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
        let construct_id = ConstructId(self.constructs.len() as u32);

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
        let mut cons_list = &mut self.constructs_of_sort[sort_id.0 as usize];
        if !cons_list.contains(&construct_id) {
            cons_list.push(construct_id);
        }
    }
}
