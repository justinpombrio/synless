use crate::style::Notation;
use serde::{Deserialize, Serialize};

/// A kind of node that can appear in a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstructSpec {
    pub name: String,
    pub arity: AritySpec,
    #[serde(default)]
    pub is_comment_or_ws: bool,
    // TODO: https://github.com/justinpombrio/synless/issues/88
    #[serde(default)]
    pub key: Option<char>,
}

/// A set of constructs. Can both include and be included by other sorts.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(deny_unknown_fields)]
pub struct SortSpec(pub Vec<String>);

/// The sorts of children that a node is allowed to contain.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum AritySpec {
    /// Designates a pure text node.
    Texty,
    /// Designates a node containing a fixed number of tree children.
    /// `Vec<ConstructSet>` contains the sort of each of its children respectively.
    Fixed(Vec<SortSpec>),
    /// Designates a node containing any number of tree children,
    /// all of the same sort.
    Listy(SortSpec),
}

/// Describes the structure of a language, e.g. which constructs can appear
/// in which positions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrammarSpec {
    pub constructs: Vec<ConstructSpec>,
    pub sorts: Vec<(String, SortSpec)>,
    pub root_construct: String,
}

/// Describes how to display every construct in a language.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NotationSetSpec {
    /// A unqiue name for this set of notations
    pub name: String,
    /// Maps `Construct.name` to that construct's notation.
    pub notations: Vec<(String, Notation)>,
}

/// A single notation, with a grammar describing its structure and a notation describing how to
/// display it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LanguageSpec {
    pub name: String,
    pub grammar: GrammarSpec,
    pub default_display_notation: NotationSetSpec,
    /// Load files with these extensions using this language. Must include the `.`.
    pub file_extensions: Vec<String>,
}
