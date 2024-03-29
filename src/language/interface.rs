use super::compiled::{
    compile_language, compile_notation_set, ArityCompiled, ConstructId, GrammarCompiled,
    LanguageCompiled, LanguageId, NotationSetId, SortId,
};
use super::specs::{LanguageSpec, NotationSetSpec};
use super::storage::Storage;
use super::LanguageError;
use crate::style::ValidNotation;
use crate::util::{bug, IndexedMap};

// NOTE: Why all the wrapper types, instead of using indexes? Two reasons:
//
// 1. It simplifies the caller. For example, instead of having to pass around a
//    pair of `(&'l Grammar, ConstructId)`, they can pass around a `Construct<'l>`.
// 2. It's safer. It disallows `grammar[construct_id]` where `grammar` and
//    `construct_id` are from different languages. This bug would be both easy
//    to introduce, and bewildering.

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

fn grammar(s: &Storage, language_id: LanguageId) -> &GrammarCompiled {
    &s.languages[language_id].grammar
}

impl Language {
    pub fn name(self, s: &Storage) -> &str {
        &s.languages[self.language].name
    }

    pub fn keymap(self, s: &Storage) -> impl ExactSizeIterator<Item = (char, Construct)> + '_ {
        grammar(s, self.language)
            .keymap
            .iter()
            .map(move |(key, c)| {
                (
                    *key,
                    Construct {
                        language: self.language,
                        construct: *c,
                    },
                )
            })
    }

    pub fn lookup_key(self, s: &Storage, key: char) -> Option<Construct> {
        grammar(s, self.language)
            .keymap
            .get(&key)
            .map(|id| Construct {
                language: self.language,
                construct: *id,
            })
    }

    pub fn notation_set_names(self, s: &Storage) -> impl ExactSizeIterator<Item = &str> + '_ {
        s.languages[self.language].notation_sets.names()
    }

    pub fn get_notation_set(self, s: &Storage, name: &str) -> Option<NotationSet> {
        s.languages[self.language]
            .notation_sets
            .id(name)
            .map(|id| NotationSet {
                language: self.language,
                notation_set: id,
            })
    }

    pub fn current_notation_set(self, s: &Storage) -> NotationSet {
        NotationSet {
            language: self.language,
            notation_set: s.languages[self.language].current_notation_set,
        }
    }

    pub fn get_construct(self, s: &Storage, construct_name: &str) -> Option<Construct> {
        let construct = grammar(s, self.language).constructs.id(construct_name)?;
        Some(Construct {
            language: self.language,
            construct,
        })
    }

    pub fn hole_construct(self, s: &Storage) -> Construct {
        Construct {
            language: self.language,
            construct: grammar(s, self.language).hole_construct,
        }
    }

    pub(super) fn from_id(id: LanguageId) -> Language {
        Language { language: id }
    }
}

impl NotationSet {
    pub fn notation(self, s: &Storage, construct: Construct) -> &ValidNotation {
        if self.language != construct.language {
            bug!("NotationSet::notation - language mismatch");
        }
        &s.languages[self.language].notation_sets[self.notation_set].notations[construct.construct]
    }
}

impl Sort {
    pub fn language(self) -> Language {
        Language {
            language: self.language,
        }
    }

    pub fn accepts(self, s: &Storage, candidate: Construct) -> bool {
        if self.language != candidate.language {
            return false;
        }

        grammar(s, self.language).sorts[self.sort]
            .0
            .contains(candidate.construct)
    }

    pub fn matching_constructs(self, s: &Storage) -> impl Iterator<Item = Construct> + '_ {
        grammar(s, self.language).sorts[self.sort]
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

    pub fn name(self, s: &Storage) -> &str {
        &grammar(s, self.language).constructs[self.construct].name
    }

    pub fn key(self, s: &Storage) -> Option<char> {
        grammar(s, self.language).constructs[self.construct].key
    }

    pub fn arity(self, s: &Storage) -> Arity {
        match grammar(s, self.language).constructs[self.construct].arity {
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

    pub fn notation(self, s: &Storage) -> &ValidNotation {
        self.language().current_notation_set(s).notation(s, self)
    }

    pub fn is_comment_or_ws(self, s: &Storage) -> bool {
        grammar(s, self.language).constructs[self.construct].is_comment_or_ws
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
    pub fn len(self, s: &Storage) -> usize {
        if let ArityCompiled::Fixed(sorts) =
            &grammar(s, self.language).constructs[self.construct].arity
        {
            sorts.len()
        } else {
            bug!("Language - FixedSort of wrong arity (len)");
        }
    }

    pub fn get(self, s: &Storage, i: usize) -> Option<Sort> {
        if let ArityCompiled::Fixed(sorts) =
            &grammar(s, self.language).constructs[self.construct].arity
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
