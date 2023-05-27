use super::ast::{Ast, Id, NodeData};
//use super::ast_ref::AstRef;
use super::forest::Forest;
use super::text::Text;
use crate::language::LanguageSet;
use crate::language::{Arity, ConstructId, Grammar};

/// All [`Asts`] belong to an `AstForest`.
///
/// It is your responsibility to ensure that `Ast`s are kept with the forest they came from. The
/// methods on `Ast`s may panic or worse if you use them on a different forest.
pub struct AstForest<'l> {
    pub(super) lang: LanguageSet<'l>,
    pub(super) forest: Forest<NodeData<'l>>,
    next_id: Id,
}

impl<'l> AstForest<'l> {
    /// Construct a new, empty, forest.
    pub fn new(language_set: LanguageSet<'l>) -> AstForest<'l> {
        AstForest {
            lang: language_set,
            forest: Forest::new(),
            next_id: Id(0),
        }
    }

    /// Create a new `hole` tree in this forest.
    pub fn new_hole(&mut self) -> Ast {
        let (grammar, construct_id) = self.lang.builtin_hole_info();
        let node = NodeData {
            grammar,
            construct_id,
            text: None,
            id: self.next_id(),
        };
        Ast(self.forest.new_node(node))
    }

    /// Create a new ast tree with no children. If it has `Fixed` arity,
    /// it will come with Holes for children.
    pub fn new_tree(&mut self, grammar: &'l Grammar, construct_id: ConstructId) -> Ast {
        let construct = grammar.construct(construct_id);
        let mut node = NodeData {
            grammar,
            construct_id,
            text: None,
            id: self.next_id(),
        };
        match &construct.arity {
            Arity::Texty => {
                node.text = Some(Text::new_inactive());
                Ast(self.forest.new_node(node))
            }
            Arity::Fixed(sorts) => {
                let index = self.forest.new_node(node);
                for _ in 0..sorts.len() {
                    let child = self.new_hole().0;
                    self.forest.insert_last_child(index, child);
                }
                Ast(index)
            }
            Arity::Listy(_) => Ast(self.forest.new_node(node)),
        }
    }

    /*
    pub fn borrow<'f>(&'f self, ast: &'f Ast<'l>) -> AstRef<'f, 'l> {
        AstRef {
            lang: &self.lang,
            tree_ref: ast.tree.borrow(),
        }
    }
    */

    fn next_id(&mut self) -> Id {
        self.next_id.0 += 1;
        Id(self.next_id.0)
    }
}
