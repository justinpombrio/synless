//! An editable language.

use std::collections::HashMap;
use std::fmt;
use std::iter::Iterator;

use crate::construct::{Construct, ConstructName, Sort, BUILTIN_CONSTRUCTS};
use utility::GrowOnlyMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LanguageName(String);

pub struct Language {
    name: LanguageName,
    constructs: HashMap<ConstructName, Construct>,
    sorts: HashMap<Sort, Vec<ConstructName>>,
    keymap: HashMap<char, ConstructName>,
}

pub type LanguageSet = GrowOnlyMap<LanguageName, Language>;

impl Language {
    pub fn new(name: LanguageName) -> Language {
        Language {
            name,
            sorts: HashMap::new(),
            constructs: HashMap::new(),
            keymap: HashMap::new(),
        }
    }

    pub fn name(&self) -> &LanguageName {
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

    pub fn lookup_construct(&self, construct_name: &ConstructName) -> &Construct {
        match self.constructs.get(construct_name) {
            Some(con) => con,
            None => match BUILTIN_CONSTRUCTS.get(construct_name) {
                None => panic!(
                    "Could not find construct named {:?} in language.",
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

impl fmt::Display for LanguageName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for LanguageName {
    fn from(s: String) -> LanguageName {
        LanguageName(s)
    }
}

impl<'a> From<&'a str> for LanguageName {
    fn from(s: &'a str) -> LanguageName {
        LanguageName(s.to_string())
    }
}

impl From<LanguageName> for String {
    fn from(m: LanguageName) -> String {
        m.0
    }
}

impl<'a> From<&'a LanguageName> for String {
    fn from(m: &'a LanguageName) -> String {
        m.0.to_owned()
    }
}
