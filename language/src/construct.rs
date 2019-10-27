// TODO: fix example
// TODO: use or remove commented code

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fmt;

pub type ConstructName = String;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Sort(String); // "Any" is special

impl Sort {
    /// Construct a new "Any" sort.
    pub fn any() -> Sort {
        "Any".into()
    }

    /// Return true if a hole with this sort can accept a child with the given sort.
    pub fn accepts(&self, child: &Sort) -> bool {
        &self.0 == "Any" || &child.0 == "Any" || self == child
    }
}

impl From<String> for Sort {
    fn from(s: String) -> Sort {
        Sort(s)
    }
}

impl<'a> From<&'a str> for Sort {
    fn from(s: &'a str) -> Sort {
        Sort(s.to_owned())
    }
}

/// A syntactic construct.
#[derive(Debug, PartialEq, Eq)]
pub struct Construct {
    pub name: ConstructName,
    pub sort: Sort,
    pub arity: Arity,
    pub key: Option<char>,
}

impl Construct {
    pub fn new<T>(name: &str, sort: T, arity: Arity, key: Option<char>) -> Construct
    where
        T: Into<Sort>,
    {
        Construct {
            name: name.to_string(),
            sort: sort.into(),
            arity,
            key,
        }
    }

    pub fn hole() -> &'static Construct {
        BUILTIN_CONSTRUCTS
            .get("hole")
            .expect("no builtin 'hole' construct found")
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
    Flexible(Sort),
}

impl Arity {
    pub fn is_text(&self) -> bool {
        match self {
            Arity::Text => true,
            _ => false,
        }
    }

    pub fn is_mixed(&self) -> bool {
        match self {
            Arity::Mixed(_) => true,
            _ => false,
        }
    }

    pub fn is_fixed(&self) -> bool {
        match self {
            Arity::Fixed(_) => true,
            _ => false,
        }
    }

    pub fn is_flexible(&self) -> bool {
        match self {
            Arity::Flexible(_) => true,
            _ => false,
        }
    }

    /// Get the `Sort` of the `i`th child. For flexible-arity and mixed-arity nodes, get the `Sort`
    /// required of all tree children, ignoring `i`.
    ///
    /// # Panics
    ///
    /// Panics if nodes of this arity cannot have an `i`th child.
    pub fn child_sort(&self, i: usize) -> &Sort {
        match self {
            Arity::Flexible(sort) | Arity::Mixed(sort) => sort, // all tree children have the same Sort
            Arity::Fixed(sorts) => sorts.get(i).unwrap_or_else(|| {
                panic!("child_sort - fixed node has only {} children", sorts.len())
            }),
            _ => panic!("child_sort - node has no children"),
        }
    }

    /// Get the `Sort` of all children of this flexible or mixed node.
    ///
    /// # Panics
    ///
    /// Panics if the arity is not flexible or mixed.
    pub fn uniform_child_sort(&self) -> &Sort {
        match self {
            Arity::Flexible(sort) | Arity::Mixed(sort) => sort, // all tree children have the same Sort
            _ => panic!("uniform_child_sort - node is not flexible or mixed"),
        }
    }
}

impl fmt::Display for Arity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Like `Arity`, but without any data in the variants.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArityType {
    Text,
    Fixed,
    Flexible,
    Mixed,
}

impl From<Arity> for ArityType {
    fn from(arity: Arity) -> ArityType {
        match arity {
            Arity::Flexible(..) => ArityType::Flexible,
            Arity::Fixed(..) => ArityType::Fixed,
            Arity::Text => ArityType::Text,
            Arity::Mixed(..) => ArityType::Mixed,
        }
    }
}

lazy_static! {
    /// Built-in constructs that can appear in any document.
    pub static ref BUILTIN_CONSTRUCTS: HashMap<ConstructName, Construct> = vec![
        (
            "hole".to_owned(),
            Construct::new("hole", Sort::any(), Arity::Fixed(vec!()), Some('?'))
        ),
        (
            "root".to_owned(),
            Construct::new("root", "root", Arity::Fixed(vec![Sort::any()]), None)
        )
    ].into_iter().collect();
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
