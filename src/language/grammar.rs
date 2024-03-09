use super::LanguageError;
use bit_set::BitSet;
use std::collections::HashMap;

/********************************************
 *         Grammar Specs                    *
 ********************************************/

/// A kind of node that can appear in a document.
///
/// This is used when constructing a grammar with [GrammarBuilder].
/// The final [Grammar] instead contains [Construct]s.
#[derive(Debug, Clone)]
pub struct ConstructSpec {
    pub name: String,
    pub arity: AritySpec,
    // TODO: https://github.com/justinpombrio/synless/issues/88
    pub key: Option<char>,
}

/// A set of constructs. Can both include and be included by other sorts.
///
/// This is used when constructing a grammar with [GrammarBuilder].
/// The final [Grammar] instead contains [Sort]s.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SortSpec(Vec<String>);

/// The sorts of children that a node is allowed to contain.
///
/// This is used when constructing a grammar with [GrammarBuilder].
/// The final [Grammar] instead contains [Arity]s.
#[derive(Debug, Clone)]
pub enum AritySpec {
    /// Designates a pure text node.
    Texty,
    /// Designates a node containing a fixed number of tree children.
    /// `Vec<ConstructSet>` contains the sort of each of its children respectively.
    Fixed(Vec<SortSpec>),
    /// Designates a node containing any number of tree children,
    /// all of the same sort.
    Listy(SortSpec),
}

/********************************************
 *         Compiled Grammar                 *
 ********************************************/

type LanguageId = usize;
type SortId = usize;
type ConstructId = usize;

#[derive(Debug)]
struct ConstructCompiled {
    name: String,
    arity: ArityCompiled,
    key: Option<char>,
}

#[derive(Debug)]
enum ArityCompiled {
    Texty,
    Fixed(Vec<(SortId, SortSpec)>),
    Listy((SortId, SortSpec)),
}

/// ConstructId -> "is contained in sort"
#[derive(Debug)]
struct SortCompiled(BitSet);

/// Describes the structure of a language, e.g. which constructs can appear
/// in which positions.
struct GrammarCompiled {
    language_name: String,
    language_id: LanguageId,
    /// ConstructId -> ConstructCompiled
    constructs: Vec<ConstructCompiled>,
    /// SortId -> SortCompiled
    sorts: Vec<SortCompiled>,
    /// Key -> ConstructId
    keymap: HashMap<char, ConstructId>,
}

/********************************************
 *         Grammar Builder                  *
 ********************************************/

/// Used to construct a [`Grammar`].
pub struct GrammarBuilder {
    constructs: HashMap<String, (ConstructId, ConstructSpec)>,
    sorts: HashMap<String, SortSpec>,
    grammar: GrammarCompiled,
}

impl GrammarBuilder {
    pub fn new(language_name: String) -> GrammarBuilder {
        GrammarBuilder {
            constructs: HashMap::new(),
            sorts: HashMap::new(),
            grammar: GrammarCompiled {
                language_name,
                language_id: 0, // Will be set when added to LanguageSet!
                constructs: Vec::new(),
                sorts: Vec::new(),
                keymap: HashMap::new(),
            },
        }
    }

    pub fn add_construct(&mut self, construct: ConstructSpec) -> Result<(), LanguageError> {
        if self.constructs.contains_key(&construct.name) {
            return Err(LanguageError::DuplicateConstruct(construct.name.clone()));
        } else if self.sorts.contains_key(&construct.name) {
            return Err(LanguageError::DuplicateConstructAndSort(
                construct.name.clone(),
            ));
        }

        let id = self.constructs.len();
        self.constructs
            .insert(construct.name.clone(), (id, construct));
        Ok(())
    }

    pub fn add_sort(&mut self, name: String, sort: SortSpec) -> Result<(), LanguageError> {
        if self.sorts.contains_key(&name) {
            return Err(LanguageError::DuplicateSort(name));
        } else if self.constructs.contains_key(&name) {
            return Err(LanguageError::DuplicateConstructAndSort(name));
        }

        self.sorts.insert(name, sort);
        Ok(())
    }

    fn compile_sort(&mut self, sort: &SortSpec) -> Result<SortId, LanguageError> {
        let mut bitset = BitSet::new();
        let mut names = sort.0.iter().collect::<Vec<_>>();
        while let Some(name) = names.pop() {
            if let Some((construct_id, _)) = self.constructs.get(name) {
                bitset.insert(*construct_id);
            } else if let Some(child_sort) = self.sorts.get(name) {
                for child_name in &child_sort.0 {
                    names.push(child_name);
                }
            } else {
                return Err(LanguageError::UndefinedConstructOrSort(name.to_owned()));
            }
        }

        for (sort_id, compiled_sort) in self.grammar.sorts.iter().enumerate() {
            if compiled_sort.0 == bitset {
                return Ok(sort_id);
            }
        }

        let sort_id = self.grammar.sorts.len();
        self.grammar.sorts.push(SortCompiled(bitset));
        Ok(sort_id)
    }

    fn compile_construct(
        &mut self,
        construct_id: ConstructId,
        construct: ConstructSpec,
    ) -> Result<(), LanguageError> {
        let arity = match construct.arity {
            AritySpec::Texty => ArityCompiled::Texty,
            AritySpec::Fixed(sort_specs) => ArityCompiled::Fixed(
                sort_specs
                    .into_iter()
                    .map(|sort_spec| Ok((self.compile_sort(&sort_spec)?, sort_spec)))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            AritySpec::Listy(sort_spec) => {
                ArityCompiled::Listy((self.compile_sort(&sort_spec)?, sort_spec))
            }
        };

        if let Some(key) = construct.key {
            if let Some(other_id) = self.grammar.keymap.get(&key) {
                return Err(LanguageError::DuplicateKey(
                    key,
                    construct.name.clone(),
                    self.grammar.constructs[*other_id].name.to_owned(),
                ));
            }
            self.grammar.keymap.insert(key, construct_id);
        }

        assert_eq!(construct_id, self.grammar.constructs.len());
        self.grammar.constructs.push(ConstructCompiled {
            name: construct.name,
            arity,
            key: construct.key,
        });
        Ok(())
    }

    fn finish(mut self) -> Result<GrammarCompiled, LanguageError> {
        for sort in self.sorts.values().cloned().collect::<Vec<_>>() {
            self.compile_sort(&sort)?;
        }
        for (id, construct) in self.constructs.values().cloned().collect::<Vec<_>>() {
            self.compile_construct(id, construct)?;
        }
        Ok(self.grammar)
    }
}
