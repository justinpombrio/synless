// TODO: fix example
// TODO: use or remove commented code

use lazy_static::lazy_static;

pub type ConstructName = String;
pub type Sort = String; // "Any" is special

/// A syntactic construct.
///
/// For example,
/// `Construct::new("plus", ForestArity{arity: 2, flexible: false}, some_notation_for_plus)`
/// might represent binary addition.
#[derive(Debug)]
pub struct Construct {
    pub name: ConstructName,
    pub sort: Sort,
    pub arity: Arity,
    pub key: char,
}

impl Construct {
    pub fn new(name: &str, sort: &str, arity: Arity, key: char) -> Construct {
        Construct {
            name: name.to_string(),
            sort: sort.to_string(),
            arity: arity,
            key: key,
        }
    }
}

#[derive(Debug)]
pub enum Arity {
    Text,
    Mixed(Sort),
    Forest(Vec<Sort>, Option<Sort>), // if Some, rest of children have this sort
}

lazy_static! {
    /// A hole in the document, for when your program is incomplete.
    pub static ref HOLE: Construct =
        Construct::new("?", "Any", Arity::Forest(vec!(), None), '?');
}

/*
#[cfg(test)]
lazy_static! {
    pub static ref TEST_FOREST: Construct = {
        let syntax = literal("TEST_FOREST", Style::plain());
        Construct::new("TEST_FOREST",
                       Arity::Forest{ arity: 0, flexible: false },
                       syntax)
    };

    pub static ref TEST_TEXT: Construct = {
        let syntax = literal("TEST_TEXT", Style::plain());
        Construct::new("TEST_TEXT", Arity::Text, syntax)
    };

    pub static ref TEST_MIXED: Construct = {
        let syntax = literal("TEST_MIXED", Style::plain());
        Construct::new("TEST_MIXED", Arity::Mixed
    };
}
*/
