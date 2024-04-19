use super::specs::{
    AritySpec, ConstructSpec, GrammarSpec, LanguageSpec, NotationSetSpec, SortSpec,
};
use crate::language::LanguageError;
use crate::style::{StyleLabel, ValidNotation};
use crate::util::{IndexedMap, SynlessBug};
use bit_set::BitSet;
use partial_pretty_printer as ppp;
use std::collections::HashMap;

const HOLE_NAME: &str = "$hole";

// Other options: ✵ ✶ ✦ ✳ ✪ ✺ ⍟ ❂ ★ ◯ ☐ ☉ ◼
const HOLE_LITERAL: &str = "☐";

pub type SortId = usize;
pub type ConstructId = usize;
pub type LanguageId = usize;
pub type NotationSetId = usize;

#[derive(Debug)]
pub struct ConstructCompiled {
    pub name: String,
    pub arity: ArityCompiled,
    pub is_comment_or_ws: bool,
    pub key: Option<char>,
}

#[derive(Debug)]
pub enum ArityCompiled {
    Texty,
    Fixed(Vec<(SortId, SortSpec)>),
    Listy(SortId, SortSpec),
}

/// ConstructId -> "is contained in sort"
#[derive(Debug)]
pub struct SortCompiled(pub BitSet);

#[derive(Debug)]
pub struct GrammarCompiled {
    pub constructs: IndexedMap<ConstructCompiled>,
    /// SortId -> SortCompiled
    pub sorts: Vec<SortCompiled>,
    /// The unique top-level construct.
    pub root_construct: ConstructId,
    pub hole_construct: ConstructId,
    /// Key -> ConstructId
    pub keymap: HashMap<char, ConstructId>,
}

#[derive(Debug)]
pub struct LanguageCompiled {
    pub name: String,
    pub grammar: GrammarCompiled,
    pub notation_sets: IndexedMap<NotationSetCompiled>,
    pub source_notation: Option<NotationSetId>,
    pub display_notation: NotationSetId,
    /// Load files with these extensions using this language. Must include the `.`.
    pub file_extensions: Vec<String>,
}

#[derive(Debug)]
pub struct NotationSetCompiled {
    pub name: String,
    /// ConstructId -> ValidNotation
    pub notations: Vec<ValidNotation>,
}

pub fn compile_language(language_spec: LanguageSpec) -> Result<LanguageCompiled, LanguageError> {
    let grammar = language_spec.grammar.compile()?;

    let notation_set = compile_notation_set(language_spec.default_display_notation, &grammar)?;
    let mut notation_sets = IndexedMap::new();
    notation_sets.insert(notation_set.name.to_owned(), notation_set);

    Ok(LanguageCompiled {
        name: language_spec.name,
        grammar,
        notation_sets,
        source_notation: None,
        display_notation: 0,
        file_extensions: language_spec.file_extensions,
    })
}

fn inject_notation_set_builtins(notation_set_spec: &mut NotationSetSpec) {
    use ppp::notation_constructors::{lit, style};
    let hole_notation = style(StyleLabel::Hole, lit(HOLE_LITERAL));
    notation_set_spec
        .notations
        .push((HOLE_NAME.to_owned(), hole_notation));
}

pub(super) fn compile_notation_set(
    mut notation_set: NotationSetSpec,
    grammar: &GrammarCompiled,
) -> Result<NotationSetCompiled, LanguageError> {
    inject_notation_set_builtins(&mut notation_set);

    // Put notations in a HashMap, checking for duplicate entries.
    let mut notations_map = HashMap::new();
    for (construct_name, notation) in notation_set.notations {
        if notations_map
            .insert(construct_name.clone(), notation)
            .is_some()
        {
            return Err(LanguageError::DuplicateNotation(
                notation_set.name,
                construct_name.clone(),
            ));
        }
    }

    // Look up the notation of every construct in the grammar,
    // putting them in a Vec ordered by ConstructId.
    let mut notations = Vec::new();
    for id in &grammar.constructs {
        let construct = &grammar.constructs[id];
        if let Some(notation) = notations_map.remove(&construct.name) {
            let valid_notation = notation.validate().map_err(|err| {
                LanguageError::InvalidNotation(
                    notation_set.name.clone(),
                    construct.name.clone(),
                    err,
                )
            })?;
            notations.push(valid_notation);
        } else {
            return Err(LanguageError::MissingNotation(
                notation_set.name,
                construct.name.clone(),
            ));
        }
    }

    // Any remaining notations don't name any construct in the grammar!
    if let Some(construct_name) = notations_map.into_keys().next() {
        return Err(LanguageError::UndefinedNotation(
            notation_set.name,
            construct_name,
        ));
    }

    Ok(NotationSetCompiled {
        name: notation_set.name,
        notations,
    })
}

struct GrammarCompiler {
    constructs: IndexedMap<ConstructSpec>,
    sorts: HashMap<String, SortSpec>,
    root_construct: String,
}

impl GrammarSpec {
    fn compile(self) -> Result<GrammarCompiled, LanguageError> {
        let mut builder = GrammarCompiler::new(self.root_construct);
        for construct in self.constructs {
            builder.add_construct(construct)?;
        }
        for (name, sort) in self.sorts {
            builder.add_sort(name, sort)?;
        }
        builder.finish()
    }
}

impl GrammarCompiler {
    fn new(root_construct: String) -> GrammarCompiler {
        GrammarCompiler {
            constructs: IndexedMap::new(),
            sorts: HashMap::new(),
            root_construct,
        }
    }

    fn add_construct(&mut self, construct: ConstructSpec) -> Result<(), LanguageError> {
        if self.sorts.contains_key(&construct.name) {
            return Err(LanguageError::DuplicateConstructAndSort(
                construct.name.clone(),
            ));
        }

        if self.constructs.contains_name(&construct.name) {
            return Err(LanguageError::DuplicateConstruct(construct.name));
        }
        self.constructs.insert(construct.name.clone(), construct);
        Ok(())
    }

    fn add_sort(&mut self, name: String, sort: SortSpec) -> Result<(), LanguageError> {
        if self.sorts.contains_key(&name) {
            return Err(LanguageError::DuplicateSort(name));
        } else if self.constructs.contains_name(&name) {
            return Err(LanguageError::DuplicateConstructAndSort(name));
        }

        self.sorts.insert(name, sort);
        Ok(())
    }

    /// Adds the $hole construct to the grammar.
    fn inject_builtins(&mut self) -> Result<(), LanguageError> {
        // Allow all fixed children to be holes
        for id in &self.constructs {
            let construct_spec = &mut self.constructs[id];
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
        self.inject_builtins()?;

        let root_construct = self
            .constructs
            .id(&self.root_construct)
            .ok_or_else(|| LanguageError::UndefinedConstruct(self.root_construct.to_owned()))?;

        if matches!(
            self.constructs.get(root_construct).bug().arity,
            AritySpec::Texty
        ) {
            return Err(LanguageError::TextyRoot(self.root_construct.to_owned()));
        }

        let mut grammar = GrammarCompiled {
            constructs: IndexedMap::new(),
            sorts: Vec::new(),
            root_construct,
            hole_construct: self.constructs.id(HOLE_NAME).bug(),
            keymap: HashMap::new(),
        };

        for sort in self.sorts.values() {
            self.compile_sort(&mut grammar, sort)?;
        }
        for id in &self.constructs {
            let construct = &self.constructs[id];
            self.compile_construct(&mut grammar, id, construct)?;
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
            if let Some(construct_id) = self.constructs.id(name) {
                bitset.insert(construct_id);
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
                    .collect::<Result<Vec<_>, LanguageError>>()?,
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
        grammar.constructs.insert(
            construct.name.clone(),
            ConstructCompiled {
                name: construct.name.clone(),
                arity,
                is_comment_or_ws: construct.is_comment_or_ws,
                key: construct.key,
            },
        );
        Ok(())
    }
}
