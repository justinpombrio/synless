use super::compiled::{
    compile_language, compile_notation_set, ArityCompiled, ConstructId, GrammarCompiled,
    LanguageCompiled, LanguageId, NotationSetId, SortId,
};
use super::specs::{LanguageSpec, NotationSetSpec};
use crate::language::LanguageError;
use crate::style::ValidNotation;
use crate::util::{bug, IndexedMap};

// NOTE: Why all the wrapper types, instead of using indexes? Two reasons:
//
// 1. It simplifies the caller. For example, instead of having to pass around a
//    pair of `(&'l Grammar, ConstructId)`, they can pass around a `Construct<'l>`.
// 2. It's safer. It disallows `grammar[construct_id]` where `grammar` and
//    `construct_id` are from different languages. This bug would be both easy
//    to introduce, and bewildering.

// TODO: Eliminate this type. Inline it & its methods into Storage.
/// The (unique) collection of all loaded [`Language`]s.
pub struct LanguageSet {
    languages: IndexedMap<LanguageCompiled>,
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
        &l.languages[self.language].name
    }

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
        l.languages[self.language].notation_sets.names()
    }

    pub fn get_notation_set(self, l: &LanguageSet, name: &str) -> Option<NotationSet> {
        l.languages[self.language]
            .notation_sets
            .id(name)
            .map(|id| NotationSet {
                language: self.language,
                notation_set: id,
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

    /// This construct must never be used!
    pub(crate) fn invalid_dummy() -> Construct {
        Construct {
            language: 666,
            construct: 666,
        }
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
    pub fn new() -> LanguageSet {
        LanguageSet {
            languages: IndexedMap::new(),
        }
    }

    pub fn add_language(&mut self, language_spec: LanguageSpec) -> Result<(), LanguageError> {
        let language = compile_language(language_spec)?;
        self.languages
            .insert(language.name.clone(), language)
            .map_err(LanguageError::DuplicateLanguage)
    }

    pub fn add_notation_set(
        &mut self,
        language_name: &str,
        notation_set: NotationSetSpec,
    ) -> Result<(), LanguageError> {
        if let Some(language) = self.languages.get_by_name_mut(language_name) {
            let notation_set = compile_notation_set(notation_set, &language.grammar)?;
            language
                .notation_sets
                .insert(notation_set.name.clone(), notation_set)
                .map_err(|name| LanguageError::DuplicateNotationSet(language_name.to_owned(), name))
        } else {
            Err(LanguageError::UndefinedLanguage(language_name.to_owned()))
        }
    }

    fn grammar(&self, language_id: LanguageId) -> &GrammarCompiled {
        &self.languages[language_id].grammar
    }
}
