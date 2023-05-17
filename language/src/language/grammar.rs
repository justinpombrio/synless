use super::LanguageError;
use std::collections::HashMap;
use std::ops::Deref;
use std::ops::Index;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SortId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ConstructId(usize);

/// The "type" of a construct. Used to determine which constructs are
/// allowed to be children of other constructs (see [Arity]).
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum SortSpec {
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
    pub sort: SortSpec,
    pub arity: AritySpec,
    pub key: Option<char>,
}

#[derive(Debug)]
struct ConstructCompiled {
    name: String,
    sort: SortId,
    arity: ArityCompiled,
    key: Option<char>,
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
    /// `Vec<SortSpec>` contains the sort of each of its children respectively.
    Fixed(Vec<SortSpec>),
    /// Designates a node containing any number of tree children,
    /// all of the same sort.
    Listy(SortSpec),
}

#[derive(Debug)]
enum ArityCompiled {
    Texty,
    Fixed(Vec<SortId>),
    Listy(SortId),
}

pub struct GrammarBuilder {
    grammar: Grammar,
    sort_map: HashMap<SortSpec, SortId>,
}

pub struct Grammar {
    language_name: String,
    /// SortId -> SortSpec
    sorts: Vec<SortSpec>,
    /// ConstructId -> ConstructCompiled
    constructs: Vec<ConstructCompiled>,
    /// SortId -> [ConstructId]
    constructs_of_sort: Vec<Vec<ConstructId>>,
    /// Key -> ConstructId
    keymap: HashMap<char, ConstructId>,
}

impl Grammar {
    pub fn language_name(&self) -> &str {
        &self.language_name
    }

    pub fn all_sorts(&self) -> impl ExactSizeIterator<Item = Sort> + '_ {
        (0..self.sorts.len()).map(move |id| Sort::new(self, SortId(id)))
    }

    pub fn all_constructs(&self) -> impl ExactSizeIterator<Item = Construct> + '_ {
        (0..self.constructs.len()).map(move |id| Construct::new(self, ConstructId(id)))
    }

    pub fn keymap(&self) -> impl ExactSizeIterator<Item = (char, Construct)> + '_ {
        self.keymap
            .iter()
            .map(move |(key, c)| (*key, Construct::new(self, *c)))
    }

    pub fn construct_of_key(&self, key: char) -> Option<Construct> {
        self.keymap.get(&key).map(|id| Construct::new(self, *id))
    }
}

impl Index<ConstructId> for Grammar {
    type Output = ConstructCompiled;

    fn index(&self, id: ConstructId) -> &ConstructCompiled {
        &self.constructs[id.0]
    }
}

impl Index<SortId> for Grammar {
    type Output = SortSpec;

    fn index(&self, id: SortId) -> &SortSpec {
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
        let compiled_construct = ConstructCompiled {
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

    fn compile_arity(&mut self, arity: AritySpec) -> ArityCompiled {
        match arity {
            AritySpec::Texty => ArityCompiled::Texty,
            AritySpec::Fixed(sorts) => {
                ArityCompiled::Fixed(sorts.into_iter().map(|sort| self.add_sort(sort)).collect())
            }
            AritySpec::Listy(sort) => ArityCompiled::Listy(self.add_sort(sort)),
        }
    }

    fn add_sort(&mut self, sort: SortSpec) -> SortId {
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

// TODO: move these?

pub struct Sort<'l> {
    grammar: &'l Grammar,
    id: SortId,
}

pub struct Construct<'l> {
    grammar: &'l Grammar,
    id: ConstructId,
}

pub enum Arity<'l> {
    Texty,
    Fixed(SortList<'l>),
    Listy(Sort<'l>),
}

pub struct SortList<'l> {
    grammar: &'l Grammar,
    ids: &'l [SortId],
}

impl<'l> SortList<'l> {
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn get(&self, i: usize) -> Sort<'l> {
        Sort::new(self.grammar, self.ids[i])
    }
}

impl<'l> Sort<'l> {
    fn new(grammar: &'l Grammar, id: SortId) -> Sort<'l> {
        Sort { grammar, id }
    }

    pub fn grammar(&self) -> &'l Grammar {
        self.grammar
    }

    /// Return true if a hole with this sort can accept a node with the given sort.
    pub fn accepts(&self, candidate: Sort) -> bool {
        match (self.deref(), candidate.deref()) {
            (_, SortSpec::Any) | (SortSpec::Any, _) => true,
            (SortSpec::Named(x), SortSpec::Named(y)) => x == y,
        }
    }

    pub fn matching_constructs(&self) -> impl ExactSizeIterator<Item = Construct> + '_ {
        self.grammar.constructs_of_sort[self.id.0]
            .iter()
            .map(move |id| Construct::new(self.grammar, *id))
    }
}

impl<'l> Deref for Sort<'l> {
    type Target = SortSpec;

    fn deref(&self) -> &SortSpec {
        &self.grammar[self.id]
    }
}

impl<'l> Construct<'l> {
    fn new(grammar: &'l Grammar, id: ConstructId) -> Construct<'l> {
        Construct { grammar, id }
    }

    pub fn grammar(&self) -> &'l Grammar {
        self.grammar
    }

    pub fn name(&self) -> &str {
        &self.grammar[self.id].name
    }

    pub fn sort(&self) -> Sort {
        Sort::new(self.grammar, self.grammar[self.id].sort)
    }

    pub fn arity(&self) -> Arity {
        match &self.grammar[self.id].arity {
            ArityCompiled::Texty => Arity::Texty,
            ArityCompiled::Fixed(sort_ids) => Arity::Fixed(SortList {
                grammar: self.grammar,
                ids: sort_ids,
            }),
            ArityCompiled::Listy(sort_id) => Arity::Listy(Sort::new(self.grammar, *sort_id)),
        }
    }

    pub fn key(&self) -> Option<char> {
        self.grammar[self.id].key
    }

    // Used to index into NotationSets
    pub(super) fn id(&self) -> usize {
        self.id.0
    }
}
