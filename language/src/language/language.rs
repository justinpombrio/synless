use super::grammar::{AritySpec, Construct, ConstructSpec, Grammar, GrammarBuilder, SortSpec};
use super::LanguageError;
use partial_pretty_printer::Notation;
use std::collections::HashMap;
use std::ops::Index;
use typed_arena::Arena;

/// The (unique) collection of all loaded [Language]s.
pub struct LanguageSet<'l> {
    grammar_storage: &'l Arena<Grammar>,
    notation_storage: &'l Arena<NotationSet>,
    /// Language name -> Language
    languages: HashMap<String, Language<'l>>,
}

/// Backing storage for all languages. Borrowed to create a [LanguageSet].
#[derive(Default)]
pub struct LanguageStorage {
    grammars: Arena<Grammar>,
    notations: Arena<NotationSet>,
}

/// A single language in which documents can be written. Consists of a
/// [Grammar] describing the structure of the language, and a set of
/// [NotationSet]s each giving one way of displaying the language.
pub struct Language<'l> {
    pub grammar: &'l Grammar,
    pub notations: Vec<&'l NotationSet>,
}

pub struct NotationSet {
    name: String,
    /// Construct id -> Notation
    notations: Vec<Notation>,
}

impl LanguageStorage {
    pub fn new() -> LanguageStorage {
        LanguageStorage {
            grammars: Arena::new(),
            notations: Arena::new(),
        }
    }
}

impl NotationSet {
    pub fn new(
        name: String,
        grammar: &Grammar,
        mut notation_map: HashMap<String, Notation>,
    ) -> Result<NotationSet, LanguageError> {
        // TODO: further notation validation (e.g. check arity)
        let mut notations = Vec::with_capacity(grammar.all_constructs().len());
        for construct in grammar.all_constructs() {
            if let Some(notation) = notation_map.remove(construct.name()) {
                notations.push(notation);
            } else {
                return Err(LanguageError::MissingNotation(construct.name().to_owned()));
            }
        }
        Ok(NotationSet { name, notations })
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<'l> Index<Construct<'l>> for NotationSet {
    type Output = Notation;

    fn index(&self, construct: Construct<'l>) -> &Notation {
        &self.notations[construct.id()]
    }
}

fn make_builtin_language() -> (Grammar, NotationSet) {
    use partial_pretty_printer::notation_constructors::{child, lit};
    use partial_pretty_printer::{Color, Style};

    let mut builder = GrammarBuilder::new("Synless Builtins".to_owned());
    builder
        .add_construct(ConstructSpec {
            name: "Hole".to_owned(),
            sort: SortSpec::Any,
            arity: AritySpec::Fixed(Vec::new()),
            key: Some('?'),
        })
        .unwrap();
    builder
        .add_construct(ConstructSpec {
            name: "Root".to_owned(),
            sort: SortSpec::Named("root".to_owned()),
            arity: AritySpec::Fixed(vec![SortSpec::Any]),
            key: None,
        })
        .unwrap();
    let grammar = builder.finish();

    let mut notations = HashMap::new();
    notations.insert(
        "Hole".to_owned(),
        lit(
            "?",
            Style {
                color: Color::Base0C,
                bold: true,
                underlined: false,
                reversed: true,
            },
        ),
    );
    notations.insert("Root".to_owned(), child(0));
    let notation_set = NotationSet::new("Default".to_owned(), &grammar, notations).unwrap();

    (grammar, notation_set)
}

impl<'l> LanguageSet<'l> {
    pub fn new(storage: &'l mut LanguageStorage) -> LanguageSet<'l> {
        let mut lang_set = LanguageSet {
            grammar_storage: &storage.grammars,
            notation_storage: &storage.notations,
            languages: HashMap::new(),
        };

        let (grammar, notation) = make_builtin_language();
        let lang_name = grammar.language_name().to_owned();
        lang_set.add_language(grammar);
        lang_set.add_notations(&lang_name, notation);
        lang_set
    }

    pub fn add_language(&mut self, grammar: Grammar) {
        // TODO: check for duplicate lang name
        let grammar = self.grammar_storage.alloc(grammar);
        let language = Language {
            grammar,
            notations: Vec::new(),
        };
        self.languages
            .insert(grammar.language_name().to_owned(), language);
    }

    pub fn add_notations(&mut self, language_name: &str, notation_set: NotationSet) {
        // TODO: Err on no such language name
        let lang = self.languages.get_mut(language_name).unwrap();
        let notation_set = self.notation_storage.alloc(notation_set);
        lang.notations.push(notation_set);
    }

    pub fn get_language(&self, language_name: &str) -> Option<&Language<'l>> {
        self.languages.get(language_name)
    }

    pub fn builtin_hole_info(&self) -> Construct<'l> {
        // TODO: Avoid magic constants?
        let lang = &self.languages["Synless Builtins"];
        lang.grammar.all_constructs().nth(0).unwrap()
    }

    pub fn builtin_root_info(&self) -> Construct<'l> {
        let lang = &self.languages["Synless Builtins"];
        lang.grammar.all_constructs().nth(1).unwrap()
    }
}
