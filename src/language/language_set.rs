use super::LanguageError;
use crate::infra::bug;
use crate::style::ValidNotation;
use bit_set::BitSet;
use partial_pretty_printer as ppp;
use std::collections::HashMap;

const HOLE_NAME: &str = "$hole";

// NOTE: Why all the wrapper types, instead of using indexes? Two reasons:
//
// 1. It simplifies the caller. For example, instead of having to pass around a
//    pair of `(&'l Grammar, ConstructId)`, they can pass around a `Construct<'l>`.
// 2. It's safer. It disallows `grammar[construct_id]` where `grammar` and
//    `construct_id` are from different languages. This bug would be both easy
//    to introduce, and bewildering.

/********************************************
 *         Grammar Specs                    *
 ********************************************/

/// A kind of node that can appear in a document.
#[derive(Debug, Clone)]
pub struct ConstructSpec {
    pub name: String,
    pub arity: AritySpec,
    pub is_comment_or_ws: bool,
    // TODO: https://github.com/justinpombrio/synless/issues/88
    pub key: Option<char>,
}

/// A set of constructs. Can both include and be included by other sorts.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SortSpec(pub Vec<String>);

/// The sorts of children that a node is allowed to contain.
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

/// Describes the structure of a language, e.g. which constructs can appear
/// in which positions.
#[derive(Debug, Clone)]
pub struct GrammarSpec {
    pub language_name: String,
    pub constructs: Vec<ConstructSpec>,
    pub sorts: Vec<(String, SortSpec)>,
    pub root_sort: SortSpec,
}

/********************************************
 *         Compiled Grammar                 *
 ********************************************/

type SortId = usize;
type ConstructId = usize;
type LanguageId = usize;
type NotationSetId = usize;

#[derive(Debug)]
struct ConstructCompiled {
    name: String,
    arity: ArityCompiled,
    is_comment_or_ws: bool,
    key: Option<char>,
}

#[derive(Debug)]
enum ArityCompiled {
    Texty,
    Fixed(Vec<(SortId, SortSpec)>),
    Listy(SortId, SortSpec),
}

/// ConstructId -> "is contained in sort"
#[derive(Debug)]
struct SortCompiled(BitSet);

struct GrammarCompiled {
    language_name: String,
    /// ConstructId -> ConstructCompiled
    constructs: Vec<ConstructCompiled>,
    /// SortId -> SortCompiled
    sorts: Vec<SortCompiled>,
    /// Which constructs are allowed at the top level
    root_sort: SortId,
    hole_construct: ConstructId,
    /// Key -> ConstructId
    keymap: HashMap<char, ConstructId>,
}

// TODO: need to be able to add a NotationSet! It should inject notation for holes.
struct LanguageCompiled {
    grammar: GrammarCompiled,
    notation_sets_by_name: HashMap<String, NotationSetId>,
    current_notation_set: NotationSetId,
    /// NotationSetId -> NotationSetCompiled
    notation_sets: Vec<NotationSetCompiled>,
}

struct NotationSetCompiled {
    /// ConstructId -> ValidNotation
    notations: Vec<ValidNotation>,
}

/********************************************
 *         Public Interface                 *
 ********************************************/

/// The (unique) collection of all loaded [`Language`]s.
pub struct LanguageSet {
    languages_by_name: HashMap<String, LanguageId>,
    /// LanguageId -> LanguageCompiled
    languages: Vec<LanguageCompiled>,
}

/// The "type" of a construct. Used to determine which constructs are
/// allowed to be children of other constructs (see [`Arity`]).
#[derive(Debug, Clone, Copy)]
pub struct Sort {
    language: LanguageId,
    sort: SortId,
}

/// The sorts of children that a node is allowed to contain.
#[derive(Debug, Clone, Copy)]
pub enum Arity {
    Texty,
    Fixed(FixedSorts),
    Listy(Sort),
}

/// The sorts in an [`Arity::Fixed`]
#[derive(Debug, Clone, Copy)]
pub struct FixedSorts {
    language: LanguageId,
    construct: ConstructId,
}

/// A kind of node that can appear in a document.
#[derive(Debug, Clone, Copy)]
pub struct Construct {
    language: LanguageId,
    construct: ConstructId,
}

/// A single language in which documents can be written. Consists of:
/// - The structure of a language, e.g. which constructs can appear
///   in which positions.
/// - [`NotationSet`]s saying how those constructs should be displayed.
/// - The currently selected [`NotationSet`].
#[derive(Debug, Clone, Copy)]
pub struct Language {
    language: LanguageId,
}

/// One set of notations for all constructs in a language. One language may have multiple
/// `NotationSets`, so that it can be displayed in multiple ways.
#[derive(Debug, Clone, Copy)]
pub struct NotationSet {
    language: LanguageId,
    notation_set: NotationSetId,
}

impl Language {
    pub fn name(self, l: &LanguageSet) -> &str {
        &l.grammar(self.language).language_name
    }

    // TODO do we need all_sorts or all_constructs?

    pub fn keymap(self, l: &LanguageSet) -> impl ExactSizeIterator<Item = (char, Construct)> + '_ {
        l.grammar(self.language).keymap.iter().map(move |(key, c)| {
            (
                *key,
                Construct {
                    language: self.language,
                    construct: *c,
                },
            )
        })
    }

    pub fn lookup_key(self, l: &LanguageSet, key: char) -> Option<Construct> {
        l.grammar(self.language)
            .keymap
            .get(&key)
            .map(|id| Construct {
                language: self.language,
                construct: *id,
            })
    }

    pub fn notation_set_names(self, l: &LanguageSet) -> impl ExactSizeIterator<Item = &str> + '_ {
        l.languages[self.language]
            .notation_sets_by_name
            .keys()
            .map(|s| s.as_ref())
    }

    pub fn get_notation_set(self, l: &LanguageSet, name: &str) -> Option<NotationSet> {
        l.languages[self.language]
            .notation_sets_by_name
            .get(name)
            .map(|id| NotationSet {
                language: self.language,
                notation_set: *id,
            })
    }

    pub fn current_notation_set(self, l: &LanguageSet) -> NotationSet {
        NotationSet {
            language: self.language,
            notation_set: l.languages[self.language].current_notation_set,
        }
    }

    pub fn hole_construct(self, l: &LanguageSet) -> Construct {
        Construct {
            language: self.language,
            construct: l.grammar(self.language).hole_construct,
        }
    }
}

impl NotationSet {
    pub fn notation(self, l: &LanguageSet, construct: Construct) -> &ValidNotation {
        if self.language != construct.language {
            bug!("NotationSet::notation - language mismatch");
        }
        &l.languages[self.language].notation_sets[self.notation_set].notations[construct.construct]
    }
}

impl Sort {
    pub fn language(self) -> Language {
        Language {
            language: self.language,
        }
    }

    pub fn accepts(self, l: &LanguageSet, candidate: Construct) -> bool {
        if self.language != candidate.language {
            return false;
        }

        l.grammar(self.language).sorts[self.sort]
            .0
            .contains(candidate.construct)
    }

    pub fn matching_constructs(self, l: &LanguageSet) -> impl Iterator<Item = Construct> + '_ {
        l.grammar(self.language).sorts[self.sort]
            .0
            .iter()
            .map(move |id| Construct {
                language: self.language,
                construct: id,
            })
    }
}

impl Construct {
    pub fn language(self) -> Language {
        Language {
            language: self.language,
        }
    }

    pub fn name(self, l: &LanguageSet) -> &str {
        &l.grammar(self.language).constructs[self.construct].name
    }

    pub fn key(self, l: &LanguageSet) -> Option<char> {
        l.grammar(self.language).constructs[self.construct].key
    }

    pub fn arity(self, l: &LanguageSet) -> Arity {
        match l.grammar(self.language).constructs[self.construct].arity {
            ArityCompiled::Texty => Arity::Texty,
            ArityCompiled::Fixed(_) => Arity::Fixed(FixedSorts {
                language: self.language,
                construct: self.construct,
            }),
            ArityCompiled::Listy(sort_id, _) => Arity::Listy(Sort {
                language: self.language,
                sort: sort_id,
            }),
        }
    }

    pub fn notation(self, l: &LanguageSet) -> &ValidNotation {
        self.language().current_notation_set(l).notation(l, self)
    }

    pub fn is_comment_or_ws(self, l: &LanguageSet) -> bool {
        l.grammar(self.language).constructs[self.construct].is_comment_or_ws
    }
}

impl FixedSorts {
    pub fn len(self, l: &LanguageSet) -> usize {
        if let ArityCompiled::Fixed(sorts) =
            &l.grammar(self.language).constructs[self.construct].arity
        {
            sorts.len()
        } else {
            bug!("Language - FixedSort of wrong arity (len)");
        }
    }

    pub fn get(self, l: &LanguageSet, i: usize) -> Option<Sort> {
        if let ArityCompiled::Fixed(sorts) =
            &l.grammar(self.language).constructs[self.construct].arity
        {
            sorts.get(i).map(|(sort_id, _)| Sort {
                language: self.language,
                sort: *sort_id,
            })
        } else {
            bug!("Language - FixedSort of wrong arity (get)");
        }
    }
}

impl LanguageSet {
    fn grammar(&self, language_id: LanguageId) -> &GrammarCompiled {
        &self.languages[language_id].grammar
    }
}

/********************************************
 *         Grammar Builder                  *
 ********************************************/

struct GrammarBuilder {
    language_name: String,
    constructs: HashMap<String, (ConstructId, ConstructSpec)>,
    sorts: HashMap<String, SortSpec>,
    root_sort: SortSpec,
}

impl GrammarSpec {
    fn compile(mut self) -> Result<GrammarCompiled, LanguageError> {
        let mut builder = GrammarBuilder::new(self.language_name, self.root_sort);
        for construct in self.constructs {
            builder.add_construct(construct)?;
        }
        for (name, sort) in self.sorts {
            builder.add_sort(name, sort)?;
        }
        builder.finish()
    }
}

impl GrammarBuilder {
    fn new(language_name: String, root_sort: SortSpec) -> GrammarBuilder {
        GrammarBuilder {
            language_name,
            constructs: HashMap::new(),
            sorts: HashMap::new(),
            root_sort,
        }
    }

    fn add_construct(&mut self, construct: ConstructSpec) -> Result<ConstructId, LanguageError> {
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
        Ok(id)
    }

    fn add_sort(&mut self, name: String, sort: SortSpec) -> Result<(), LanguageError> {
        if self.sorts.contains_key(&name) {
            return Err(LanguageError::DuplicateSort(name));
        } else if self.constructs.contains_key(&name) {
            return Err(LanguageError::DuplicateConstructAndSort(name));
        }

        self.sorts.insert(name, sort);
        Ok(())
    }

    /// Adds the $hole construct to the grammar. Returns its id.
    fn inject_builtins(&mut self) -> Result<ConstructId, LanguageError> {
        // Allow all fixed children to be holes
        for (_, (_, construct_spec)) in &mut self.constructs {
            if let AritySpec::Fixed(children) = &mut construct_spec.arity {
                for sort_spec in children {
                    sort_spec.0.push(HOLE_NAME.to_owned());
                }
            }
        }
        // Add the hole construct
        self.add_construct(ConstructSpec {
            name: HOLE_NAME.to_owned(),
            arity: AritySpec::Fixed(Vec::new()),
            is_comment_or_ws: false,
            key: None,
        })
    }

    fn finish(mut self) -> Result<GrammarCompiled, LanguageError> {
        let hole_id = self.inject_builtins()?;

        let mut grammar = GrammarCompiled {
            language_name: self.language_name.clone(),
            constructs: Vec::new(),
            sorts: Vec::new(),
            root_sort: 0,
            hole_construct: hole_id,
            keymap: HashMap::new(),
        };

        grammar.root_sort = self.compile_sort(&mut grammar, &self.root_sort)?;
        for sort in self.sorts.values() {
            self.compile_sort(&mut grammar, sort)?;
        }
        for (id, construct) in self.constructs.values() {
            self.compile_construct(&mut grammar, *id, construct)?;
        }

        Ok(grammar)
    }

    fn compile_sort(
        &self,
        grammar: &mut GrammarCompiled,
        sort: &SortSpec,
    ) -> Result<SortId, LanguageError> {
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

        for (sort_id, compiled_sort) in grammar.sorts.iter().enumerate() {
            if compiled_sort.0 == bitset {
                return Ok(sort_id);
            }
        }

        let sort_id = grammar.sorts.len();
        grammar.sorts.push(SortCompiled(bitset));
        Ok(sort_id)
    }

    fn compile_construct(
        &self,
        grammar: &mut GrammarCompiled,
        construct_id: ConstructId,
        construct: &ConstructSpec,
    ) -> Result<(), LanguageError> {
        let arity = match &construct.arity {
            AritySpec::Texty => ArityCompiled::Texty,
            AritySpec::Fixed(sort_specs) => ArityCompiled::Fixed(
                sort_specs
                    .iter()
                    .map(|sort_spec| {
                        Ok((self.compile_sort(grammar, sort_spec)?, sort_spec.clone()))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            AritySpec::Listy(sort_spec) => {
                ArityCompiled::Listy(self.compile_sort(grammar, sort_spec)?, sort_spec.clone())
            }
        };

        if let Some(key) = construct.key {
            if let Some(other_id) = grammar.keymap.get(&key) {
                return Err(LanguageError::DuplicateKey(
                    key,
                    construct.name.clone(),
                    grammar.constructs[*other_id].name.to_owned(),
                ));
            }
            grammar.keymap.insert(key, construct_id);
        }

        assert_eq!(construct_id, grammar.constructs.len());
        grammar.constructs.push(ConstructCompiled {
            name: construct.name.clone(),
            arity,
            is_comment_or_ws: construct.is_comment_or_ws,
            key: construct.key,
        });
        Ok(())
    }
}
