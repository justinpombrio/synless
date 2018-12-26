// TODO: fix example
// TODO: use or remove commented code

use std::fmt;
use lazy_static::lazy_static;

pub type ConstructName = String;
pub type Sort = String; // "Any" is special

/// A syntactic construct.
#[derive(Debug, PartialEq, Eq)]
pub struct Construct {
    pub name:  ConstructName,
    pub sort:  Sort,
    pub arity: Arity,
    pub key:   char
}

impl Construct {
    pub fn new(name: &str, sort: &str, arity: Arity, key: char) -> Construct {
        Construct{
            name: name.to_string(),
            sort: sort.to_string(),
            arity: arity,
            key: key
        }
    }
}

/// The sorts of children that a node is allowed to contain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Arity {
    /// Designates a pure text node.
    Text,
    /// Designates a node containing mixed text and trees.
    /// `Sort` is the sort of trees it may contain.
    Mixed(Sort),
    /// Designates a node containing a fixed number of tree children.
    /// `Vec<Sort>` contains the `Sort`s of each of its children respectively.
    Fixed(Vec<Sort>),
    /// Designates a node containing any number of tree children,
    /// all of the same `Sort`.
    Flexible(Sort)
}

impl fmt::Display for Arity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

lazy_static! {
    /// A hole in the document, for when your program is incomplete.
    pub static ref HOLE: Construct =
        Construct::new("Hole", "Any", Arity::Fixed(vec!()), '?');
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
