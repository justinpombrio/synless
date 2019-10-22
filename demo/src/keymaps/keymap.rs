use frontends::Key;
use std::collections::HashMap;

use crate::prog::Prog;
use language::{ArityType, Sort};

/// Stores all the keybindings of a text-keymap, except for keys that just
/// insert themselves as literal characters.
pub struct TextKeymap<'l>(HashMap<Key, Prog<'l>>);

/// Stores all the keybindings of a tree-keymap, along with rules for which
/// keybindings should be available in which contexts.
pub struct TreeKeymap<'l>(HashMap<Key, (FilterRule, Prog<'l>)>);

/// Rules for when a particular item should be included in a keymap.
#[derive(Clone, Debug)]
pub enum FilterRule {
    /// Unconditionally include the item.
    Always,
    /// Only include the item if the given `Sort` is acceptable in the current position.
    Sort(Sort),
    /// Only include the item if the arity-type of the current node's parent is contained in the given list.
    ParentArity(Vec<ArityType>),
    /// Only include the item if the arity-type of the current node is contained in the given list.
    SelfArity(Vec<ArityType>),
}

/// Information needed to apply a FilterRule, based on the context within a
/// document.
pub struct FilterContext {
    /// The Sort that any node in the current node's position is required to have.
    pub sort: Sort,
    /// The arity of the current node's parent.
    pub parent_arity: ArityType,
    /// The arity of the current node.
    pub self_arity: ArityType,
}

impl<'l> TextKeymap<'l> {
    pub fn new(non_literal_keys: HashMap<Key, Prog<'l>>) -> Self {
        Self(non_literal_keys)
    }

    pub(super) fn empty() -> Self {
        Self(HashMap::new())
    }

    pub(super) fn get<'a>(&'a self, key: &Key) -> Option<&'a Prog<'l>> {
        self.0.get(key)
    }

    pub(super) fn keys_and_names<'a>(&'a self) -> Vec<(&'a Key, Option<&'a str>)> {
        self.0
            .iter()
            .map(|(key, prog)| (key, prog.name()))
            .collect()
    }
}

impl<'l> TreeKeymap<'l> {
    pub fn new(v: Vec<(Key, FilterRule, Prog<'l>)>) -> Self {
        Self(
            v.into_iter()
                .map(|(key, filter, prog)| (key, (filter, prog)))
                .collect(),
        )
    }

    pub(super) fn get<'a>(&'a self, key: &Key) -> Option<&'a Prog<'l>> {
        self.0.get(key).map(|(_filter, prog)| prog)
    }

    pub(super) fn filter<'a>(&'a self, context: &FilterContext) -> Vec<Key> {
        self.0
            .iter()
            .filter_map(|(&key, (filter, _))| {
                if filter.matches(context) {
                    Some(key)
                } else {
                    None
                }
            })
            .collect()
    }
}

impl FilterRule {
    /// True if an item with this filter rule should be included in this context.
    fn matches(&self, context: &FilterContext) -> bool {
        match self {
            FilterRule::Always => true,
            FilterRule::Sort(sort) => context.sort.accepts(sort),
            FilterRule::ParentArity(arity_types) => arity_types.contains(&context.parent_arity),
            FilterRule::SelfArity(arity_types) => arity_types.contains(&context.self_arity),
        }
    }
}
