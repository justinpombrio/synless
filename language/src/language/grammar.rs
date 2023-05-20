use super::LanguageError;
use std::collections::HashMap;

// NOTE: Why all the wrapper types, instead of using indexes? Two reasons:
//
// 1. It simplifies the caller. For example, instead of having to pass around a
//    pair of `(&'l Grammar, ConstructId)`, they can pass around a `Construct<'l>`.
// 2. It's safer. It disallows `grammar[construct_id]` where `grammar` and
//    `construct_id` are from different languages. This bug would be both easy
//    to introduce, and bewildering.

type SortId = usize;
type ConstructId = usize;

/// The "type" of a construct. Used to determine which constructs are
/// allowed to be children of other constructs (see [AritySpec]).
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

/// Used to construct a Grammar
pub struct GrammarBuilder {
    grammar: Grammar,
    sort_map: HashMap<SortSpec, SortId>,
}

/// Describes the structure of a language, e.g. which constructs can appear
/// in which positions.
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
        (0..self.sorts.len()).map(move |id| Sort::new(self, id))
    }

    pub fn all_constructs(&self) -> impl ExactSizeIterator<Item = Construct> + '_ {
        (0..self.constructs.len()).map(move |id| Construct::new(self, id))
    }

    /// All (key, construct) pairs in the key map.
    pub fn keymap(&self) -> impl ExactSizeIterator<Item = (char, Construct)> + '_ {
        self.keymap
            .iter()
            .map(move |(key, c)| (*key, Construct::new(self, *c)))
    }

    /// Look up one key in the keymap.
    pub fn lookup_key(&self, key: char) -> Option<Construct> {
        self.keymap.get(&key).map(|id| Construct::new(self, *id))
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
        let construct_id = self.grammar.constructs.len();
        self.grammar.constructs.push(compiled_construct);
        self.grammar.constructs_of_sort[sort_id].push(construct_id);
        if let Some(key) = construct.key {
            let duplicate = self.grammar.keymap.insert(key, construct_id);
            if let Some(prev_construct) = duplicate {
                return Err(LanguageError::DuplicateKey(
                    key,
                    self.grammar.constructs[prev_construct].name.clone(),
                    self.grammar.constructs[construct_id].name.clone(),
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
            let id = self.grammar.sorts.len();
            self.sort_map.insert(sort.clone(), id);
            self.grammar.sorts.push(sort);
            self.grammar.constructs_of_sort.push(Vec::new());
            id
        }
    }
}

/// The "type" of a construct. Used to determine which constructs are
/// allowed to be children of other constructs (see [Arity]).
pub struct Sort<'l> {
    grammar: &'l Grammar,
    id: SortId,
}

/// A kind of node that can appear in a document.
pub struct Construct<'l> {
    grammar: &'l Grammar,
    id: ConstructId,
}

/// The sorts of children that a node is allowed to contain.
pub enum Arity<'l> {
    Texty,
    Fixed(SortList<'l>),
    Listy(Sort<'l>),
}

/// Essentially a slice of [Sort]s. (The only reason this isn't literally a slice
/// is because there are wrapper types involved.)
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

    /// Get the i'th [Sort], or **panic** if out of bounds.
    pub fn get(&self, i: usize) -> Sort<'l> {
        Sort::new(self.grammar, self.ids[i])
    }
}

impl<'l> Sort<'l> {
    fn new(grammar: &'l Grammar, id: SortId) -> Sort<'l> {
        Sort { grammar, id }
    }

    /// The grammar that this sort is a part of.
    pub fn grammar(&self) -> &'l Grammar {
        self.grammar
    }

    /// Return whether a hole with this sort can accept a node with the `candidate` sort.
    pub fn accepts(&self, candidate: Sort) -> bool {
        match (
            &self.grammar.sorts[self.id],
            &candidate.grammar.sorts[candidate.id],
        ) {
            (_, SortSpec::Any) | (SortSpec::Any, _) => true,
            (SortSpec::Named(x), SortSpec::Named(y)) => x == y,
        }
    }

    /// All [Construct]s whose sort is this sort.
    pub fn matching_constructs(&self) -> impl ExactSizeIterator<Item = Construct> + '_ {
        self.grammar.constructs_of_sort[self.id]
            .iter()
            .map(move |id| Construct::new(self.grammar, *id))
    }
}

impl<'l> Construct<'l> {
    fn new(grammar: &'l Grammar, id: ConstructId) -> Construct<'l> {
        Construct { grammar, id }
    }

    /// The grammar that this construct is a part of.
    pub fn grammar(&self) -> &'l Grammar {
        self.grammar
    }

    pub fn name(&self) -> &str {
        &self.grammar.constructs[self.id].name
    }

    /// Determines where this construct is allowed to be placed:
    /// it's allowed if the `sort()` of the parent [Sort::accepts]
    /// the [Construct::arity] of the child.
    pub fn sort(&self) -> Sort {
        Sort::new(self.grammar, self.grammar.constructs[self.id].sort)
    }

    /// Determines what children this construct is allowed to have:
    /// it's allowed if the [Construct::sort] of the parent [Sort::accepts]
    /// the [Construct::arity] of the child.
    pub fn arity(&self) -> Arity {
        match &self.grammar.constructs[self.id].arity {
            ArityCompiled::Texty => Arity::Texty,
            ArityCompiled::Fixed(sort_ids) => Arity::Fixed(SortList {
                grammar: self.grammar,
                ids: sort_ids,
            }),
            ArityCompiled::Listy(sort_id) => Arity::Listy(Sort::new(self.grammar, *sort_id)),
        }
    }

    pub fn key(&self) -> Option<char> {
        self.grammar.constructs[self.id].key
    }

    // Used to index into NotationSets
    pub(super) fn id(&self) -> usize {
        self.id
    }
}
