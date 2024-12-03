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
    /// Designates a pure text node. Optionally constrained to match a regex.
    Texty(Option<String>),
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
    pub notations: Vec<NotationSetSpec>,
    pub default_display_notation: String,
    pub default_source_notation: Option<String>,
    /// Load files with these extensions using this language. Must include the `.`.
    pub file_extensions: Vec<String>,
    pub hole_syntax: Option<HoleSyntax>,
}

/// The syntax to use when saving and loading holes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HoleSyntax {
    /// What to save each hole as in a source file. It should ideally cause a syntax error when
    /// parsed with the language's standard parser.
    pub invalid: String,
    /// Something syntactically valid to convert holes into, before Synless parses it with a
    /// standard parser for the language.
    pub valid: String,
    /// After a `HoleSyntax.valid` is parsed into a texty node, this is the contents of that node.
    /// It will then be replaced by a hole, completing the cycle.
    pub text: String,
}
