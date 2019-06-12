//! An editable language.

use std::collections::HashMap;
use std::iter::Iterator;

use crate::construct::{Construct, ConstructName, Sort, BUILTIN_CONSTRUCTS};
use utility::GrowOnlyMap;

pub type LanguageName = String;

pub struct Language {
    name: LanguageName,
    constructs: HashMap<ConstructName, Construct>,
    sorts: HashMap<Sort, Vec<ConstructName>>,
    keymap: HashMap<char, ConstructName>,
}

pub type LanguageSet = GrowOnlyMap<String, Language>;

impl Language {
    pub fn new(name: &str) -> Language {
        Language {
            name: name.to_string(),
            sorts: HashMap::new(),
            constructs: HashMap::new(),
            keymap: HashMap::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add(&mut self, construct: Construct) {
        // Insert sort
        if !self.sorts.contains_key(&construct.sort) {
            self.sorts.insert(construct.sort.clone(), vec![]);
        }
        self.sorts
            .get_mut(&construct.sort)
            .unwrap()
            .push(construct.name.clone());
        // Insert key
        if let Some(key) = construct.key {
            self.keymap.insert(key, construct.name.clone());
        }
        // Insert construct
        self.constructs.insert(construct.name.clone(), construct);
    }

    pub fn lookup_key(&self, key: char) -> Option<&ConstructName> {
        self.keymap.get(&key)
    }

    pub fn lookup_construct(&self, construct_name: &str) -> &Construct {
        match self.constructs.get(construct_name) {
            Some(con) => con,
            None => match BUILTIN_CONSTRUCTS.get(construct_name) {
                None => panic!(
                    "Could not find construct named {} in language.",
                    construct_name
                ),
                Some(con) => con,
            },
        }
    }

    pub fn constructs(&self) -> impl Iterator<Item = &Construct> {
        self.constructs.values()
    }

    pub fn keymap(&self) -> &HashMap<char, ConstructName> {
        &self.keymap
    }
}
