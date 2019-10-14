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

pub struct TextKeymapFactory<'l>(HashMap<Key, Prog<'l>>);

pub struct TreeKeymapFactory<'l>(HashMap<Key, (FilterRule, Prog<'l>)>);

impl<'l> TextKeymapFactory<'l> {
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

impl<'l> TreeKeymapFactory<'l> {
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
