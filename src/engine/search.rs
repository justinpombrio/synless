use crate::language::{Construct, Storage};
use crate::tree::Node;

#[derive(Debug)]
pub struct Search {
    pattern: SearchPattern,
    pub highlight: bool,
}

#[derive(Debug)]
enum SearchPattern {
    /// Matches nodes of the given construct.
    Construct(Construct),
}

impl Search {
    pub fn new_construct(construct: Construct) -> Search {
        Search {
            pattern: SearchPattern::Construct(construct),
            highlight: true,
        }
    }

    pub fn matches(&self, s: &Storage, node: Node) -> bool {
        match &self.pattern {
            SearchPattern::Construct(construct) => node.construct(s) == *construct,
        }
    }
}
