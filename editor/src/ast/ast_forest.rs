use forest::Forest;
use pretty::Bounds;
use language::{Language, Arity, LanguageSet, Construct, HOLE};

use crate::notationset::NotationSet;
use crate::ast::ast::{Ast, Node};


pub struct AstForest<'l> {
    language_set: &'l LanguageSet,
    forest: Forest<Node<'l>, String>
}

impl<'l> AstForest<'l> {
    pub fn new(language_set: &'l LanguageSet) -> AstForest<'l> {
        AstForest {
            language_set: language_set,
            forest: Forest::new()
        }
    }

    pub fn new_hole(&self, language: &'l Language, notation_set: &'l NotationSet)
                    -> Ast<'l>
    {
        let node = Node {
            bounds: Bounds::empty(),
            language: language,
            construct: &HOLE,
            notation: notation_set.lookup(&HOLE.name)
        };
        Ast::new(self.forest.new_branch(node, vec!()))
    }

    // TODO: is this right?
    pub fn new_text_tree(&self, text: &str) -> Ast<'l> {
        Ast::new(self.forest.new_leaf(text.to_string()))
    }

    // TODO: check that language has construct! UUID?
    pub fn new_fixed_tree(&self,
                          language: &'l Language,
                          construct: &'l Construct,
                          notation_set: &'l NotationSet)
                          -> Ast<'l>
    {
        let node = Node {
            bounds: Bounds::empty(),
            language: language,
            construct: construct,
            notation: notation_set.lookup(&construct.name)
        };
        let arity = match &construct.arity {
            Arity::Fixed(sorts) => sorts.len(),
            a => panic!("AstForest::new_fixed_tree - expected a node of fixed arity, but found arity {}", a)
        };
        let children = vec!(self.new_hole(language, notation_set).tree; arity);
        Ast::new(self.forest.new_branch(node, children))
    }

    pub fn new_flexible_tree(&self,
                             language: &'l Language,
                             construct: &'l Construct,
                             notation_set: &'l NotationSet)
                             -> Ast<'l>
    {
        let node = Node {
            bounds: Bounds::empty(),
            language: language,
            construct: construct,
            notation: notation_set.lookup(&construct.name)
        };
        match &construct.arity {
            Arity::Flexible(_) => (),
            a => panic!("AstForest::new_flexible_tree - expected a node of flexible arity, but found arity {}", a)
        };
        Ast::new(self.forest.new_branch(node, vec!()))
    }

    pub fn new_mixed_tree(&self,
                          language: &'l Language,
                          construct: &'l Construct,
                          notation_set: &'l NotationSet) -> Ast<'l>
    {
        // TODO: probably shouldn't be copy-pasting this
        let node = Node {
            bounds: Bounds::empty(),
            language: language,
            construct: construct,
            notation: notation_set.lookup(&construct.name)
        };
        match &construct.arity {
            Arity::Mixed(_) => (),
            a => panic!("AstForest::new_mixed_tree - expected a node of mixed arity, but found arity {}", a)
        };
        Ast::new(self.forest.new_branch(node, vec!()))
    }
}
