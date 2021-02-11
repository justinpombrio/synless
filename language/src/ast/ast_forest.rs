use super::ast::{Ast, Id, NodeData};
use super::ast_ref::AstRef;
use super::text::Text;
use crate::language::LanguageSet;
use crate::language::{Arity, ConstructId, Grammar};
use forest::Forest;

/// All [`Asts`] belong to an `AstForest`.
///
/// It is your responsibility to ensure that `Ast`s are kept with the forest they came from. The
/// methods on `Ast`s may panic or worse if you use them on a different forest.
pub struct AstForest<'l> {
    pub(super) lang: LanguageSet<'l>,
    forest: Forest<NodeData<'l>, Text>,
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

    /// Create a new `hole` node in this forest.
    // TODO: 'cept for Id, this can take &self! Is that useful?
    pub fn new_hole(&mut self) -> Ast<'l> {
        let (grammar, construct_id) = self.lang.builtin_hole_info();
        let node = NodeData {
            grammar,
            construct_id,
            id: self.next_id(),
        };
        Ast::new(self.forest.new_branch(node, vec![]))
    }

    pub fn new_tree(&mut self, grammar: &'l Grammar, construct_id: ConstructId) -> Ast<'l> {
        let construct = grammar.construct(construct_id);
        let node = NodeData {
            grammar,
            construct_id,
            id: self.next_id(),
        };
        match &construct.arity {
            Arity::Texty => Ast::new(self.forest.new_leaf(node, Text::new_inactive())),
            Arity::Fixed(sorts) => {
                let children = (0..sorts.len())
                    .map(|_| self.new_hole().tree)
                    .collect::<Vec<_>>();
                Ast::new(self.forest.new_branch(node, children))
            }
            Arity::Listy(_) => Ast::new(self.forest.new_branch(node, vec![])),
        }
    }

    pub fn borrow<R>(&self, ast: &Ast<'l>, func: impl FnOnce(AstRef<'_, 'l>) -> R) -> R {
        ast.tree.borrow(|tree_ref| {
            func(AstRef {
                lang: &self.lang,
                tree_ref: tree_ref,
            })
        })
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
