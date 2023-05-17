// Disable stupid warnings
#![allow(clippy::iter_nth_zero)]
#![allow(clippy::module_inception)]

//mod ast;
mod language;

pub use crate::language::{
    Arity, AritySpec, Construct, ConstructSpec, Grammar, GrammarBuilder, Language, LanguageError,
    LanguageSet, LanguageStorage, NotationSet, Sort, SortList, SortSpec,
};
//pub use ast::{Ast, AstCase, AstForest, AstRef, FixedAst, ListyAst, Text, TextyAst};
