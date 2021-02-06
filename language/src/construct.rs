use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SortId(pub(crate) u32);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Sort {
    Any,
    Named(String),
}

/// A kind of node that can appear in a document.
#[derive(Debug)]
pub struct Construct {
    pub name: String,
    pub sort_id: SortId,
    pub arity: Arity,
    pub key: Option<char>,
}

/// The sorts of children that a node is allowed to contain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Arity {
    /// Designates a pure text node.
    Texty,
    /// Designates a node containing a fixed number of tree children.
    /// `Vec<Sort>` contains the `Sort`s of each of its children respectively.
    Fixed(Vec<Sort>),
    /// Designates a node containing any number of tree children,
    /// all of the same `Sort`.
    Listy(Sort),
}

/// Like `Arity`, but without any data in the variants.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArityType {
    Texty,
    Fixed,
    Listy,
}

impl Sort {
    /// Construct a new "Any" sort.
    pub fn any() -> Sort {
        Sort::Any
    }

    pub fn named(sort_name: String) -> Sort {
        Sort::Named(sort_name)
    }

    /// Return true if a hole with this sort can accept a node with the given sort.
    pub fn accepts(&self, candidate: &Sort) -> bool {
        match (self, candidate) {
            (Sort::Any, _) => true,
            (_, Sort::Any) => true,
            (Sort::Named(x), Sort::Named(y)) => x == y,
        }
    }
}

impl Arity {
    pub fn is_texty(&self) -> bool {
        matches!(self, Arity::Texty)
    }

    pub fn is_fixed(&self) -> bool {
        matches!(self, Arity::Fixed(_))
    }

    pub fn is_listy(&self) -> bool {
        matches!(self, Arity::Listy(_))
    }

    /// Get the `Sort` of the `i`th child. For listy nodes, get the `Sort` required of all tree
    /// children, ignoring `i`.
    ///
    /// # Panics
    ///
    /// Panics if nodes of this arity cannot have an `i`th child.
    pub fn child_sort(&self, i: usize) -> &Sort {
        match self {
            Arity::Listy(sort) => sort,
            Arity::Fixed(sorts) => sorts.get(i).unwrap_or_else(|| {
                panic!("child_sort - fixed node has only {} children", sorts.len())
            }),
            _ => panic!("child_sort - node has no children"),
        }
    }

    pub fn arity_type(&self) -> ArityType {
        match self {
            Arity::Texty => ArityType::Texty,
            Arity::Fixed(_) => ArityType::Fixed,
            Arity::Listy(_) => ArityType::Listy,
        }
    }
}
