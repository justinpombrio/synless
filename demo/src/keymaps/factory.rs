use std::collections::HashMap;
use termion::event::Key;

use crate::prog::Prog;
use language::{ArityType, Sort};

/// Rules for when a particular item should be included in a keymap
#[derive(Clone, Debug)]
pub enum FilterRule {
    Always,
    Sort(Sort),
    ParentArity(Vec<ArityType>),
    SelfArity(Vec<ArityType>),
}

pub struct FilterContext {
    pub required_sort: Sort,
    pub parent_arity: ArityType,
    pub self_arity: ArityType,
}

pub struct TreeKmapFactory<'l>(HashMap<Key, (FilterRule, Prog<'l>)>);

impl<'l> TreeKmapFactory<'l> {
    pub fn new(v: Vec<(Key, FilterRule, Prog<'l>)>) -> Self {
        TreeKmapFactory(
            v.into_iter()
                .map(|(key, filter, prog)| (key, (filter, prog)))
                .collect(),
        )
    }

    pub fn get<'a>(&'a self, key: &Key) -> Option<&'a Prog<'l>> {
        self.0.get(key).map(|(_filter, prog)| prog)
    }

    pub fn filter<'a>(&'a self, context: &FilterContext) -> Vec<Key> {
        self.0
            .iter()
            .filter_map(|(&key, (filter, _))| match filter {
                FilterRule::Always => Some(key),
                FilterRule::Sort(sort) => {
                    if context.required_sort.accepts(sort) {
                        Some(key)
                    } else {
                        None
                    }
                }
                FilterRule::ParentArity(arity_types) => {
                    if arity_types.contains(&context.parent_arity) {
                        Some(key)
                    } else {
                        None
                    }
                }
                FilterRule::SelfArity(arity_types) => {
                    if arity_types.contains(&context.self_arity) {
                        Some(key)
                    } else {
                        None
                    }
                }
            })
            .collect()
    }
}
