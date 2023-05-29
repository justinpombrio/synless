use super::grammar::{
    AritySpec, Construct, ConstructSpec, Grammar, GrammarBuilder, LanguageId, SortSpec,
};
use super::LanguageError;
use partial_pretty_printer::ValidNotation;
use std::collections::HashMap;
use std::ops::Index;
use typed_arena::Arena;

/// The (unique) collection of all loaded [Language]s.
pub struct LanguageSet<'l> {
    grammar_storage: &'l Arena<Grammar>,
    notation_storage: &'l Arena<NotationSet>,
    builtins_language_id: LanguageId,
    languages_by_name: HashMap<String, LanguageId>,
    languages: Vec<Language<'l>>,
}

/// Backing storage for all languages. Borrowed to create a [LanguageSet].
#[derive(Default)]
pub struct LanguageStorage {
    grammars: Arena<Grammar>,
    notation_sets: Arena<NotationSet>,
}

/// A single language in which documents can be written. Consists of a
/// [Grammar] describing the structure of the language, a set of
/// [NotationSet]s each giving one way of displaying the language, and
/// the currently selected [NotationSet].
pub struct Language<'l> {
    grammar: &'l Grammar,
    notation_sets: HashMap<String, &'l NotationSet>,
    current_notation_set: &'l NotationSet,
}

pub struct NotationSet {
    name: String,
    /// Construct id -> ValidNotation
    notations: Vec<ValidNotation>,
}

impl LanguageStorage {
    pub fn new() -> LanguageStorage {
        LanguageStorage {
            grammars: Arena::new(),
            notation_sets: Arena::new(),
        }
    }
}

impl NotationSet {
    pub fn new(
        name: String,
        grammar: &Grammar,
        mut notation_map: HashMap<String, ValidNotation>,
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
    type Output = ValidNotation;

    fn index(&self, construct: Construct<'l>) -> &ValidNotation {
        &self.notations[construct.id()]
    }
}

fn make_builtin_language() -> (Grammar, NotationSet) {
    use partial_pretty_printer::notation_constructors::{child, lit};
    use partial_pretty_printer::{Color, Style};

    let mut builder = GrammarBuilder::new("Synless Builtins".to_owned());
    // Method get_hole_construct() relies on this being at index 0!
    builder
        .add_construct(ConstructSpec {
            name: "Hole".to_owned(),
            sort: SortSpec::Any,
            arity: AritySpec::Fixed(Vec::new()),
            key: Some('?'),
        })
        .unwrap();
    // Method get_root_construct() relies on this being at index 1!
    builder
        .add_construct(ConstructSpec {
            name: "Root".to_owned(),
            sort: SortSpec::Named("root".to_owned()),
            arity: AritySpec::Fixed(vec![SortSpec::Any]),
            key: None,
        })
        .unwrap();
    let grammar = builder.finish();

    let hole_notation = lit(
        "?",
        Style {
            color: Color::Base0C,
            bold: true,
            underlined: false,
            reversed: true,
        },
    )
    .validate()
    .expect("Builtin hole notation invalid");
    let root_notation = child(0).validate().expect("Builtin root notation invalid");

    let mut notations = HashMap::new();
    notations.insert("Hole".to_owned(), hole_notation);
    notations.insert("Root".to_owned(), root_notation);
    let notation_set = NotationSet::new("Default".to_owned(), &grammar, notations).unwrap();

    (grammar, notation_set)
}

impl<'l> LanguageSet<'l> {
    pub fn new(storage: &'l mut LanguageStorage) -> LanguageSet<'l> {
        let mut lang_set = LanguageSet {
            grammar_storage: &storage.grammars,
            notation_storage: &storage.notation_sets,
            languages_by_name: HashMap::new(),
            languages: Vec::new(),
            builtins_language_id: 0, // depends on the `add_language()` call below!
        };

        let (grammar, notation) = make_builtin_language();
        lang_set.add_language(grammar, notation);
        lang_set
    }

    pub fn add_language(&mut self, mut grammar: Grammar, default_notation_set: NotationSet) {
        let lang_id = self.languages.len();
        grammar.language_id = lang_id;
        let grammar = self.grammar_storage.alloc(grammar);
        let default_notation_set = self.notation_storage.alloc(default_notation_set);
        let language = Language {
            grammar,
            notation_sets: {
                let mut notation_sets = HashMap::<String, &NotationSet>::new();
                notation_sets.insert(default_notation_set.name.to_owned(), default_notation_set);
                notation_sets
            },
            current_notation_set: default_notation_set,
        };
        // TODO: check for languages with same name
        self.languages_by_name
            .insert(grammar.language_name().to_owned(), lang_id);
        self.languages.push(language);
    }

    pub fn add_notation_set(&mut self, language_id: LanguageId, notation_set: NotationSet) {
        let lang = &mut self.languages[language_id];
        let notation_set = self.notation_storage.alloc(notation_set);
        // TODO: check for duplicate name
        lang.notation_sets
            .insert(notation_set.name.to_owned(), notation_set);
    }

    pub fn all_languages(&self) -> impl ExactSizeIterator<Item = &Language<'l>> + '_ {
        self.languages.iter()
    }

    pub fn get_langauge(&self, language_name: &str) -> Option<&Language<'l>> {
        Some(&self.languages[*self.languages_by_name.get(language_name)?])
    }

    pub fn get_langauge_mut(&mut self, language_name: &str) -> Option<&mut Language<'l>> {
        Some(&mut self.languages[*self.languages_by_name.get(language_name)?])
    }

    pub fn get_notation(&self, construct: Construct<'l>) -> &'l ValidNotation {
        &self.languages[construct.grammar().language_id].current_notation_set[construct]
    }

    pub fn builtin_hole_construct(&self) -> Construct<'l> {
        let builtins = &self.languages[self.builtins_language_id];
        builtins.grammar.all_constructs().nth(0).unwrap()
    }

    pub fn builtin_root_construct(&self) -> Construct<'l> {
        let builtins = &self.languages[self.builtins_language_id];
        builtins.grammar.all_constructs().nth(1).unwrap()
    }
}

impl<'l> Language<'l> {
    pub fn grammar(&self) -> &'l Grammar {
        self.grammar
    }

    pub fn notation_sets(&self) -> impl ExactSizeIterator<Item = &'l NotationSet> + '_ {
        self.notation_sets.values().copied()
    }

    pub fn current_notation_set(&self) -> &'l NotationSet {
        self.current_notation_set
    }

    pub fn set_current_notation_set(&mut self, notation_set_name: &str) -> Option<()> {
        self.current_notation_set = self.notation_sets.get(notation_set_name)?;
        Some(())
    }
}
