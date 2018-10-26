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
    pub fn example_language() -> Language {
        let mut language = Language::new("TestLang");

        let arity = Arity::Forest(vec!("Expr".to_string(),
                                       "Expr".to_string()),
                                  None);
        let construct = Construct::new("plus", "Expr", arity, 'p');
        language.add(construct);
/*
        let syn = repeat(Repeat{
            empty:  empty(),
            lone:   star(),
            first:  star() + punct(", "),
            middle: star() + punct(", "),
            last:   star()
        }) | repeat(Repeat{
            empty:  empty(),
            lone:   star(),
            first:  flush(star() + punct(",")),
            middle: flush(star() + punct(",")),
            last:   star()
        });
        lang.add('a', Construct::new("args", Arity::extendable(0), syn));

        let syn = repeat(Repeat{
            empty:  punct("[]"),
            lone:   punct("[") + star() + punct("]"),
            first:  punct("[") + star() + punct(", "),
            middle: star() + punct(", "),
            last:   star() + punct("]")
        })| repeat(Repeat{
            empty:  punct("[]"),
            lone:   punct("[") + star() + punct("]"),
            first:  flush(star() + punct(",")),
            middle: flush(star() + punct(",")),
            last:   star() + punct("]")
        })| repeat(Repeat{
            empty:  punct("[]"),
            lone:   punct("[") + star() + punct("]"),
            first:  punct("[")
                + (star() + punct(", ") | flush(star() + punct(","))),
            middle: star() + punct(", ") | flush(star() + punct(",")),
            last:   star() + punct("]")
        });
        lang.add('l', Construct::new("list", Arity::extendable(0), syn));

        let syn =
            word("func ") + child(0)
            + punct("(") + child(1) + punct(") { ") + child(2) + punct(" }")
            | flush(word("func ") + child(0) + punct("(") + child(1) + punct(") {"))
            + flush(word("  ") + child(2))
            + punct("}")
            | flush(word("func ") + child(0) + punct("("))
            + flush(word("  ") + child(1) + punct(")"))
            + flush(punct("{"))
            + flush(word("  ") + child(2))
            + punct("}");
        lang.add('f', Construct::new("func", Arity::fixed(3), syn));

        let syn = if_empty_text(txt() + punct("·"), txt());
        lang.add('i', Construct::new("iden", Arity::text(), syn));

        let syn = punct("'") + txt() + punct("'");
        lang.add('s', Construct::new("strn", Arity::text(), syn));
         */

        language
    }

}
