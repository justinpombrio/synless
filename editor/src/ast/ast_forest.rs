use forest::Forest;
use language::{Arity, Construct, ConstructName, Language, LanguageSet, BUILTIN_CONSTRUCTS};
use pretty::Bounds;

use crate::ast::ast::{Ast, Node};
use crate::notationset::NotationSet;
use crate::text::Text;

/// All [Asts](Ast) belong to an AstForest.
///
/// It is your responsibility to ensure that Asts are kept with the
/// forest they came from. The methods on Asts will panic if you use
/// them on a different forest.
pub struct AstForest<'l> {
    language_set: &'l LanguageSet,
    forest: Forest<Node<'l>, Text>,
}

impl<'l> AstForest<'l> {
    /// Construct a new, empty, forest.
    pub fn new(language_set: &'l LanguageSet) -> AstForest<'l> {
        AstForest {
            language_set,
            forest: Forest::new(),
        }
    }

    pub fn new_tree(
        &self,
        language: &'l Language,
        construct_name: &ConstructName,
        notation_set: &'l NotationSet,
    ) -> Option<Ast<'l>> {
        let construct = language.lookup_construct(construct_name);
        let node = Node {
            bounds: Bounds::empty(),
            language,
            construct,
            notation: notation_set.lookup(construct_name),
        };
        let ast = match &construct.arity {
            Arity::Fixed(sorts) => {
                let children = vec![self.new_hole_tree(language, notation_set).tree; sorts.len()];
                Ast::new(self.forest.new_branch(node, children))
            }
            Arity::Flexible(_) => Ast::new(self.forest.new_branch(node, vec![])),
            Arity::Text => {
                let leaf = self.forest.new_leaf(Text::new_inactive());
                Ast::new(self.forest.new_branch(node, vec![leaf]))
            }
            Arity::Mixed(..) => unimplemented!("Mixed-arity trees"),
        };
        Some(ast)
    }

    /// Construct a hole in this forest, that represents a gap in the document.
    pub fn new_hole_tree(&self, language: &'l Language, notation_set: &'l NotationSet) -> Ast<'l> {
        let hole = BUILTIN_CONSTRUCTS
            .get("hole")
            .expect("no builtin 'hole' construct found");
        let node = Node {
            bounds: Bounds::empty(),
            language,
            construct: hole,
            notation: notation_set.lookup(&hole.name),
        };
        Ast::new(self.forest.new_branch(node, vec![]))
    }

    /// Construct a new tree in this forest, of `Text` arity.
    pub fn new_text_tree(
        &self,
        language: &'l Language,
        construct: &'l Construct,
        notation_set: &'l NotationSet,
    ) -> Ast<'l> {
        let node = Node {
            bounds: Bounds::empty(),
            language,
            construct,
            notation: notation_set.lookup(&construct.name),
        };
        if !construct.arity.is_text() {
            panic!(
                "AstForest::new_text_tree - expected a node of text arity, but found arity {}",
                construct.arity
            )
        }
        let leaf = self.forest.new_leaf(Text::new_inactive());
        Ast::new(self.forest.new_branch(node, vec![leaf]))
    }

    // TODO: check that language has construct! UUID?
    /// Construct a new tree in this forest, of `Fixed` arity.
    pub fn new_fixed_tree(
        &self,
        language: &'l Language,
        construct: &'l Construct,
        notation_set: &'l NotationSet,
    ) -> Ast<'l> {
        let node = Node {
            bounds: Bounds::empty(),
            language,
            construct,
            notation: notation_set.lookup(&construct.name),
        };
        let arity = match &construct.arity {
            Arity::Fixed(sorts) => sorts.len(),
            a => panic!(
                "AstForest::new_fixed_tree - expected a node of fixed arity, but found arity {}",
                a
            ),
        };
        let children = vec![self.new_hole_tree(language, notation_set).tree; arity];
        Ast::new(self.forest.new_branch(node, children))
    }

    /// Construct a new tree in this forest, of `Flexible` arity.
    pub fn new_flexible_tree(
        &self,
        language: &'l Language,
        construct: &'l Construct,
        notation_set: &'l NotationSet,
    ) -> Ast<'l> {
        let node = Node {
            bounds: Bounds::empty(),
            language,
            construct,
            notation: notation_set.lookup(&construct.name),
        };
        if !construct.arity.is_flexible() {
            panic!("AstForest::new_flexible_tree - expected a node of flexible arity, but found arity {}", construct.arity)
        }
        Ast::new(self.forest.new_branch(node, vec![]))
    }

    /// Construct a new tree in this forest, of `Mixed` arity.
    pub fn new_mixed_tree(
        &self,
        language: &'l Language,
        construct: &'l Construct,
        notation_set: &'l NotationSet,
    ) -> Ast<'l> {
        // TODO: probably shouldn't be copy-pasting this
        let node = Node {
            bounds: Bounds::empty(),
            language,
            construct,
            notation: notation_set.lookup(&construct.name),
        };
        if !construct.arity.is_mixed() {
            panic!(
                "AstForest::new_mixed_tree - expected a node of mixed arity, but found arity {}",
                construct.arity
            )
        }
        Ast::new(self.forest.new_branch(node, vec![]))
    }
}
