// TODO: use or remove commented code

//! An editable language.

use std::collections::HashMap;
use super::construct::{ConstructName, TypeName, Construct};

/// The syntax and whatnot for a language.
/// Right now it just has a mapping from key to syntactic construct.
/// Eventually, the language, which describes syntactic constructs,
/// should be separated from the notation that describes how to
/// display it.
pub struct Language {
    pub(crate) constructs: HashMap<ConstructName, Construct>,
    pub(crate) types:      HashMap<TypeName, Vec<ConstructName>>,
    pub(crate) keymap:     HashMap<char, ConstructName>
}

impl Language {

    pub fn new() -> Language {
        Language {
            constructs: HashMap::new(),
            types:      HashMap::new(),
            keymap:     HashMap::new()
        }
    }

    pub fn add(&mut self, key: char, construct: Construct) {
        self.keymap.insert(key, construct.name.clone());
        if !self.types.contains_key(&construct.typ) {
            self.types.insert(construct.typ.clone(), vec!());
        }
        self.types.get_mut(&construct.typ).unwrap().push(construct.name.clone());
        self.constructs.insert(construct.name.clone(), construct);
    }

    pub fn lookup(&self, key: char) -> Option<&Construct> {
        match self.keymap.get(&key) {
            Some(name) => Some(self.lookup_name(name)),
            None => None
        }
    }

    pub fn lookup_name(&self, construct: &str) -> &Construct {
        match self.constructs.get(construct) {
            Some(con) => con,
            None => panic!("Could not find construct named {} in language.",
                           construct)
        }
    }
}

/*
fn punct(s: &str) -> Syntax {
    literal(s, Style::color(Color::Yellow))
}

fn word(s: &str) -> Syntax {
    literal(s, Style::color(Color::Green))
}

fn txt() -> Syntax {
    text(Style::new(Color::Blue, Emph::underlined(), Shade::black(), false))
}

    /// An example language for testing.
    pub fn example_language() -> Language {
        let mut lang = Language::new();

        let syn =
            child(0) + punct(" + ") + child(1)
            | flush(child(0)) + punct("+ ") + child(1);
        lang.add('p', Construct::new("plus", Arity::fixed(2), syn));

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

        lang
    }
}

#[cfg(test)]
pub(crate) fn make_example_tree<'l>(lang: &'l Language, tweak: bool) -> Tree<'l> {
    let con_func = lang.lookup_name("func");
    let con_id   = lang.lookup_name("iden");
    let con_str  = lang.lookup_name("strn");
    let con_arg  = lang.lookup_name("args");
    let con_plus = lang.lookup_name("plus");

    let foo = Tree::new_text(con_id, "foo");
    let abc = Tree::new_text(con_id, "abc");
    let def = Tree::new_text(con_id, "def");
    let args = Tree::new_forest(con_arg, vec!(abc, def));
    let abcdef1 = Tree::new_text(con_str, "abcdef");
    let abcdef2 = if tweak {
        Tree::new_text(con_str, "abc")
    } else {
        Tree::new_text(con_str, "abcdef")
    };
    let body = Tree::new_forest(con_plus, vec!(abcdef1, abcdef2));
    Tree::new_forest(con_func, vec!(foo, args, body))
}
*/
