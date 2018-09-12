use style::{Style, Color};
use syntax::{Syntax, literal};

use self::Arity::{ForestArity, TextArity};

/// A syntactic construct.
///
/// For example,
/// `Construct::new("plus", Arity::fixed(2), some_syntax_for_plus)`
/// might represent binary addition.
#[derive(Debug)]
pub struct Construct {
    pub name: String,
    pub arity: Arity,
    pub syntax: Syntax
}

lazy_static! {
    // TODO: remove lazy static?
    /// A hole in the document, for when your program is incomplete.
    pub static ref HOLE: Construct = {
        let syntax = literal("?", Style::color(Color::Magenta));
        Construct::new("?", Arity::fixed(0), syntax)
    };
}

#[cfg(test)]
lazy_static! {
    pub static ref TEST_FOREST: Construct = {
        let syntax = literal("TEST_FOREST", Style::plain());
        Construct::new("TEST_FOREST", Arity::extendable(0), syntax)
    };

    pub static ref TEST_TEXT: Construct = {
        let syntax = literal("TEST_TEXT", Style::plain());
        Construct::new("TEST_TEXT", Arity::text(), syntax)
    };
}

/// The arity of a syntactic construct.
///
/// A `ForestArity` is for syntactic constructs that can have trees as
/// children, or that have no children.  If the arity is *fixed*
/// (`extendable` is false), the construct must have exactly as many
/// children as its `arity` number. If the arity is *extendable*, it
/// can have more children than that: the extra children are displayed
/// via `Star` in a `Repeat` in the `Syntax`.
///
/// A `TextArity` is for syntactic constructs that contain text.
#[derive(Clone, Copy, Debug)]
pub enum Arity {
    ForestArity{
        arity: usize,
        extendable: bool
    },
    TextArity
}

impl Construct {
    pub fn new(name: &str, arity: Arity, syntax: Syntax) -> Construct {
        Construct{
            name:   name.to_string(),
            arity:  arity,
            syntax: syntax
        }
    }

    /// Is this construct extendable (and thus also foresty)?
    pub fn is_extendable(&self) -> bool {
        match self.arity {
            ForestArity{ extendable: ext, .. } => ext,
            TextArity => false
        }
    }
}

impl Arity {
    /// Construct a fixed tree arity.
    pub fn fixed(arity: usize) -> Arity {
        ForestArity{
            arity: arity,
            extendable: false
        }
    }

    /// Construct an extendable tree arity.
    pub fn extendable(arity: usize) -> Arity {
        ForestArity{
            arity: arity,
            extendable: true
        }
    }

    /// Construct a text arity.
    pub fn text() -> Arity {
        TextArity
    }

    /// Is this a text arity?
    pub fn is_text(&self) -> bool {
        match self {
            &TextArity => true,
            _          => false
        }
    }

    /// Get the arity number for this node:
    /// the minimum number of trees it must have as children.
    pub fn arity(&self) -> usize {
        match self {
            &ForestArity{ arity, ..} => arity,
            &TextArity             => 0 // correct?
        }
    }
}
