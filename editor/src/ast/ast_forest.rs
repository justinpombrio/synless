use forest::Forest;

use crate::ast::ast::Node;

pub struct AstForest<'l> {
    forest: Forest<Node<'l>, String>
}
