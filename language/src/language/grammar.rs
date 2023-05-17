use super::{ConstructId, LanguageError};
use std::collections::HashMap;
use std::ops::Deref;
use std::ops::Index;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SortId(usize);

/// The "type" of a construct. Used to determine which constructs are
/// allowed to be children of other constructs (see [AritySpec]).
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Sort {
    Any,
    Named(String),
}

/// A kind of node that can appear in a document.
///
/// (This is used when constructing a grammar with [GrammarBuilder].
/// The final [Grammar] instead contains [Construct]s.)
#[derive(Debug)]
pub struct ConstructSpec {
    pub name: String,
    pub sort: Sort,
    pub arity: AritySpec,
    pub key: Option<char>,
}

/// A kind of node that can appear in a document.
#[derive(Debug)]
pub struct Construct {
    pub name: String,
    pub sort: SortId,
    pub arity: Arity,
    pub key: Option<char>,
}

/// The sorts of children that a node is allowed to contain.
///
/// (This is used when constructing a grammar with [GrammarBuilder].
/// The final [Grammar] instead contains [Arity]s.)
#[derive(Debug)]
pub enum AritySpec {
    /// Designates a pure text node.
    Texty,
    /// Designates a node containing a fixed number of tree children.
    /// `Vec<Sort>` contains the sort of each of its children respectively.
    Fixed(Vec<Sort>),
    /// Designates a node containing any number of tree children,
    /// all of the same sort.
    Listy(Sort),
}

/// The sorts of children that a node is allowed to contain.
#[derive(Debug)]
pub enum Arity {
    /// Designates a pure text node.
    Texty,
    /// Designates a node containing a fixed number of tree children.
    /// `Vec<Sort>` contains the sort of each of its children respectively.
    Fixed(Vec<SortId>),
    /// Designates a node containing any number of tree children,
    /// all of the same sort.
    Listy(SortId),
}

pub struct GrammarBuilder {
    grammar: Grammar,
    sort_map: HashMap<Sort, SortId>,
}

pub struct Grammar {
    language_name: String,
    /// SortId -> Sort
    sorts: Vec<Sort>,
    /// ConstructId -> ConstructSpec
    constructs: Vec<Construct>,
    /// SortId -> [ConstructId]
    constructs_of_sort: Vec<Vec<ConstructId>>,
    /// Key -> ConstructId
    keymap: HashMap<char, ConstructId>,
}

impl Sort {
    /// Return true if a hole with this sort can accept a node with the given sort.
    pub fn accepts(&self, candidate: impl AsRef<Sort>) -> bool {
        match (self, candidate.as_ref()) {
            (_, Sort::Any) | (Sort::Any, _) => true,
            (Sort::Named(x), Sort::Named(y)) => x == y,
        }
    }
}

impl Grammar {
    pub fn language_name(&self) -> &str {
        &self.language_name
    }

    pub fn all_sorts(&self) -> impl ExactSizeIterator<Item = SortRef> + '_ {
        (0..self.sorts.len()).map(move |id| SortRef::new(self, SortId(id)))
    }

    pub fn all_constructs(&self) -> impl ExactSizeIterator<Item = ConstructRef> + '_ {
        (0..self.constructs.len()).map(move |id| ConstructRef::new(self, ConstructId(id)))
    }

    pub fn constructs_of_sort(
        &self,
        sort: SortId,
    ) -> impl ExactSizeIterator<Item = ConstructId> + '_ {
        self.constructs_of_sort[sort.0].iter().copied()
    }

    pub fn keymap(&self) -> impl ExactSizeIterator<Item = (char, ConstructRef)> + '_ {
        self.keymap
            .iter()
            .map(move |(key, c)| (*key, ConstructRef::new(self, *c)))
    }

    pub fn construct_of_key(&self, key: char) -> Option<ConstructRef> {
        self.keymap.get(&key).map(|id| ConstructRef::new(self, *id))
    }
}

impl Index<ConstructId> for Grammar {
    type Output = Construct;

    fn index(&self, id: ConstructId) -> &Construct {
        &self.constructs[id.0]
    }
}

impl Index<SortId> for Grammar {
    type Output = Sort;

    fn index(&self, id: SortId) -> &Sort {
        &self.sorts[id.0]
    }
}

impl GrammarBuilder {
    pub fn new(language_name: String) -> GrammarBuilder {
        GrammarBuilder {
            grammar: Grammar {
                language_name,
                sorts: Vec::new(),
                constructs: Vec::new(),
                constructs_of_sort: Vec::new(),
                keymap: HashMap::new(),
            },
            sort_map: HashMap::new(),
        }
    }

    pub fn add_construct(&mut self, construct: ConstructSpec) -> Result<(), LanguageError> {
        let sort_id = self.add_sort(construct.sort);
        let arity = self.compile_arity(construct.arity);
        let compiled_construct = Construct {
            name: construct.name,
            sort: sort_id,
            arity,
            key: construct.key,
        };
        let construct_id = ConstructId(self.grammar.constructs.len());
        self.grammar.constructs.push(compiled_construct);
        self.grammar.constructs_of_sort[sort_id.0].push(construct_id);
        if let Some(key) = construct.key {
            let duplicate = self.grammar.keymap.insert(key, construct_id);
            if let Some(prev_construct) = duplicate {
                return Err(LanguageError::DuplicateKey(
                    key,
                    self.grammar[prev_construct].name.clone(),
                    self.grammar[construct_id].name.clone(),
                ));
            }
        }
        Ok(())
    }

    pub fn finish(self) -> Grammar {
        self.grammar
    }

    fn compile_arity(&mut self, arity: AritySpec) -> Arity {
        match arity {
            AritySpec::Texty => Arity::Texty,
            AritySpec::Fixed(sorts) => {
                Arity::Fixed(sorts.into_iter().map(|sort| self.add_sort(sort)).collect())
            }
            AritySpec::Listy(sort) => Arity::Listy(self.add_sort(sort)),
        }
    }

    fn add_sort(&mut self, sort: Sort) -> SortId {
        if let Some(id) = self.sort_map.get(&sort) {
            *id
        } else {
            let id = SortId(self.grammar.sorts.len());
            self.sort_map.insert(sort.clone(), id);
            self.grammar.sorts.push(sort);
            self.grammar.constructs_of_sort.push(Vec::new());
            id
        }
    }
}

// TODO: move these

pub struct SortRef<'l> {
    grammar: &'l Grammar,
    id: SortId,
}

pub struct ConstructRef<'l> {
    grammar: &'l Grammar,
    id: ConstructId,
}

pub enum ArityRef<'l> {
    Texty,
    Fixed(SortRefList<'l>),
    Listy(SortRef<'l>),
}

pub struct SortRefList<'l> {
    grammar: &'l Grammar,
    ids: &'l [SortId],
}

impl<'l> SortRefList<'l> {
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn get(&self, i: usize) -> SortRef<'l> {
        SortRef::new(self.grammar, self.ids[i])
    }
}

impl<'l> SortRef<'l> {
    fn new(grammar: &'l Grammar, id: SortId) -> SortRef<'l> {
        SortRef { grammar, id }
    }

    pub fn grammar(&self) -> &'l Grammar {
        self.grammar
    }

    pub fn matching_constructs(&self) -> impl ExactSizeIterator<Item = ConstructRef> + '_ {
        self.grammar.constructs_of_sort[self.id.0]
            .iter()
            .map(move |id| ConstructRef::new(self.grammar, *id))
    }
}

impl<'l> Deref for SortRef<'l> {
    type Target = Sort;

    fn deref(&self) -> &Sort {
        &self.grammar[self.id]
    }
}

impl<'l> ConstructRef<'l> {
    fn new(grammar: &'l Grammar, id: ConstructId) -> ConstructRef<'l> {
        ConstructRef { grammar, id }
    }

    pub fn grammar(&self) -> &'l Grammar {
        self.grammar
    }

    pub fn name(&self) -> &str {
        &self.grammar[self.id].name
    }

    pub fn sort(&self) -> SortRef {
        SortRef::new(self.grammar, self.grammar[self.id].sort)
    }

    pub fn arity(&self) -> ArityRef {
        match &self.grammar[self.id].arity {
            Arity::Texty => ArityRef::Texty,
            Arity::Fixed(sort_ids) => ArityRef::Fixed(SortRefList {
                grammar: self.grammar,
                ids: sort_ids,
            }),
            Arity::Listy(sort_id) => ArityRef::Listy(SortRef::new(self.grammar, *sort_id)),
        }
    }

    pub fn key(&self) -> Option<char> {
        self.grammar[self.id].key
    }
}
