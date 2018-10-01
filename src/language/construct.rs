// TODO: fix example
// TODO: use or remove commented code

pub type ConstructName = String;
pub type TypeName = String;

/// A syntactic construct.
///
/// For example,
/// `Construct::new("plus", ForestArity{arity: 2, flexible: false}, some_syntax_for_plus)`
/// might represent binary addition.
#[derive(Debug)]
pub struct Construct {
    pub name: ConstructName,
    pub typ:  TypeName,
    pub sig:  Signature
}

impl Construct {
    pub fn new(name: &str, typ: &str, sig: Signature) -> Construct {
        Construct{
            name: name.to_string(),
            typ:  typ.to_string(),
            sig:  sig
        }
    }
}

#[derive(Debug)]
pub enum Signature {
    Text,
    Mixed(TypeName),
    Forest(Vec<TypeName>, Option<TypeName>)
}

lazy_static! {
    /// A hole in the document, for when your program is incomplete.
    pub static ref HOLE: Construct =
        Construct::new("?", "Any", Signature::Forest(vec!(), None));
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
