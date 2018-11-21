// TODO: use or remove commented code

//! An editable language.

use std::collections::HashMap;
use std::iter::Iterator;

use crate::construct::{ConstructName, Sort, Construct};

pub type LanguageName = String;

// TODO: rename to Grammar
/// The notation and whatnot for a language.
pub struct Language {
    name:       LanguageName,
    constructs: HashMap<ConstructName, Construct>,
    sorts:      HashMap<Sort, Vec<ConstructName>>,
    keymap:     HashMap<char, ConstructName>
}

impl Language {

    pub fn new(name: &str) -> Language {
        Language {
            name:       name.to_string(),
            sorts:      HashMap::new(),
            constructs: HashMap::new(),
            keymap:     HashMap::new()
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add(&mut self, construct: Construct) {
        // Insert sort
        if !self.sorts.contains_key(&construct.sort) {
            self.sorts.insert(construct.sort.clone(), vec!());
        }
        self.sorts.get_mut(&construct.sort).unwrap().push(construct.name.clone());
        // Insert key
        self.keymap.insert(construct.key, construct.name.clone());
        // Insert construct
        self.constructs.insert(construct.name.clone(), construct);
    }

    pub fn lookup_key(&self, key: char) -> Option<&Construct> {
        match self.keymap.get(&key) {
            Some(name) => Some(self.lookup_construct(name)),
            None => None
        }
    }

    pub fn lookup_construct(&self, construct_name: &str) -> &Construct {
        match self.constructs.get(construct_name) {
            Some(con) => con,
            None => panic!("Could not find construct named {} in language.",
                           construct_name)
        }
    }

    pub fn constructs(&self) -> impl Iterator<Item=&Construct> {
        self.constructs.values()
    }
}

//#[cfg(test)]
//use self::example::*;

#[cfg(test)]
mod example {
    use crate::Arity;
    use super::*;

    /// An example language for testing.
    // TODO: nvm, it's used to test `pretty`.
    pub fn example_language() -> Language {
        let mut language = Language::new("TestLang");

        let arity = Arity::Forest(vec!("Expr".to_string(),
                                       "Expr".to_string()),
                                  None);
        let construct = Construct::new("plus", "Expr", arity, 'p');
        language.add(construct);

        language
    }

}
