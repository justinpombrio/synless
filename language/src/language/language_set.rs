//! An editable language.

use super::construct::{Arity, Construct, Sort};
use partial_pretty_printer::ValidNotation;
use std::collections::HashMap;
use std::iter::Iterator;
use typed_arena::Arena;
use utility::spanic;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstructId(u32);

pub struct LanguageSet<'l> {
    grammar_storage: &'l Arena<Grammar>,
    notation_storage: &'l Arena<NotationSet>,
    /// Language name -> Language
    languages: HashMap<String, Language<'l>>,
}

struct Language<'l> {
    grammar: &'l Grammar,
    current_notation: &'l NotationSet,
    all_notations: HashMap<String, &'l NotationSet>,
}

pub struct Grammar {
    language_name: String,
    sorts: Vec<Sort>,
    /// ConstructId -> Construct
    constructs: Vec<Construct>,
    constructs_of_sort: HashMap<Sort, Vec<ConstructId>>,
    keymap: HashMap<char, ConstructId>,
}

pub struct LanguageStorage {
    grammars: Arena<Grammar>,
    notations: Arena<NotationSet>,
}

pub struct NotationSet {
    name: String,
    /// Construct id -> ValidNotation
    notations: Vec<ValidNotation>,
}

impl<'l> LanguageSet<'l> {
    pub fn new(storage: &'l mut LanguageStorage) -> LanguageSet<'l> {
        let (builtins_grammar, builtins_notation) = make_builtins_language();
        let mut langs = LanguageSet {
            grammar_storage: &storage.grammars,
            notation_storage: &storage.notations,
            languages: HashMap::new(),
        };
        langs.add_language(builtins_grammar, "Default", builtins_notation);
        langs
    }

    pub fn add_language(
        &mut self,
        grammar: Grammar,
        default_notation_name: &str,
        default_notation: Vec<(String, ValidNotation)>,
    ) {
        let grammar: &'l Grammar = self.grammar_storage.alloc(grammar);
        let default_notation =
            NotationSet::new(default_notation_name.to_owned(), &grammar, default_notation);
        let default_notation: &'l NotationSet = self.notation_storage.alloc(default_notation);
        let mut all_notations = HashMap::new();
        all_notations.insert(grammar.language_name.clone(), default_notation);
        self.languages.insert(
            grammar.language_name.clone(),
            Language {
                grammar,
                current_notation: default_notation,
                all_notations,
            },
        );
    }

    pub fn add_notation_set(
        &mut self,
        language_name: &str,
        name: &str,
        notations: Vec<(String, ValidNotation)>,
    ) {
        let language = self.languages.get_mut(language_name).unwrap();
        let notation_set = NotationSet::new(name.to_owned(), language.grammar, notations);
        let notation_set: &'l NotationSet = self.notation_storage.alloc(notation_set);
        language.all_notations.insert(name.to_owned(), notation_set);
    }

    pub fn current_notation_set(&self, language_name: &str) -> &'l NotationSet {
        self.languages[language_name].current_notation
    }

    pub fn all_notation_sets(&self, language_name: &str) -> impl Iterator<Item = &NotationSet> {
        self.languages[language_name]
            .all_notations
            .values()
            .copied()
    }

    pub fn switch_notation_set(&mut self, language_name: &str, notation_set_name: &str) {
        let language = self.languages.get_mut(language_name).unwrap();
        language.current_notation = language.all_notations[notation_set_name];
    }

    pub fn builtin_hole_info(&self) -> (&'l Grammar, ConstructId) {
        // TODO: Avoid magic constants?
        let language = &self.languages["SynlessBuiltins"];
        (language.grammar, ConstructId(0))
    }

    pub fn builtin_root_info(&self) -> (&'l Grammar, ConstructId) {
        let language = &self.languages["SynlessBuiltins"];
        (language.grammar, ConstructId(1))
    }
}

fn make_builtins_language() -> (Grammar, Vec<(String, ValidNotation)>) {
    use partial_pretty_printer::notation_constructors::{child, lit};
    use partial_pretty_printer::{Color, Style};

    let mut grammar = Grammar::new("SynlessBuiltins");
    grammar.add_construct("Hole", Sort::any(), Arity::Fixed(vec![]), Some('?'));
    grammar.add_construct(
        "Root",
        Sort::named("Root"),
        Arity::Fixed(vec![Sort::any()]),
        None,
    );

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
    .expect("builtin hole notation is not valid");

    let root_notation = child(0)
        .validate()
        .expect("builtin root notation is not valid");

    let notations = vec![
        ("Hole".to_owned(), hole_notation),
        ("Root".to_owned(), root_notation),
    ];
    (grammar, notations)
}

impl NotationSet {
    pub fn new(
        name: String,
        grammar: &Grammar,
        notations: Vec<(String, ValidNotation)>,
    ) -> NotationSet {
        let mut notations_map = notations.into_iter().collect::<HashMap<_, _>>();
        let notations = grammar
            .constructs
            .iter()
            .map(|con| notations_map.remove(&con.name).unwrap())
            .collect::<Vec<_>>();
        NotationSet { name, notations }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn lookup(&self, construct_id: ConstructId) -> &ValidNotation {
        &self.notations[construct_id.0 as usize]
    }
}

impl Grammar {
    pub fn new(language_name: &str) -> Grammar {
        Grammar {
            language_name: language_name.to_owned(),
            sorts: vec![],
            constructs: vec![],
            constructs_of_sort: HashMap::new(),
            keymap: HashMap::new(),
        }
    }

    pub fn language_name(&self) -> &str {
        &self.language_name
    }

    pub fn lookup_key(&self, key: char) -> Option<&Construct> {
        Some(&self.constructs[self.keymap.get(&key)?.0 as usize])
    }

    pub fn construct(&self, construct_id: ConstructId) -> &Construct {
        &self.constructs[construct_id.0 as usize]
    }

    pub fn keymap(&self) -> impl Iterator<Item = (char, &str)> {
        self.keymap
            .iter()
            .map(move |(ch, con)| (*ch, self.construct(*con).name.as_ref()))
    }

    pub fn constructs(&self) -> impl Iterator<Item = &Construct> {
        self.constructs.iter()
    }

    fn add_sort(&mut self, sort: &Sort) {
        if !self.sorts.contains(&sort) {
            self.sorts.push(sort.to_owned());
            self.constructs_of_sort.insert(sort.to_owned(), vec![]);
        }
    }

    pub fn add_construct(&mut self, name: &str, sort: Sort, arity: Arity, key: Option<char>) {
        // Add the sort
        self.add_sort(&sort);

        // Add the construct
        let construct = Construct {
            name: name.to_owned(),
            sort: sort.clone(),
            arity,
            key,
        };
        let construct_id = ConstructId(self.constructs.len() as u32);
        self.constructs.push(construct);

        // Extend the keymap
        if let Some(key) = key {
            let duplicate = self.keymap.insert(key, construct_id);
            if duplicate.is_some() {
                spanic!("Duplicate key '{}'", key);
            }
        }

        // Extend the construct list for the sort
        let cons_list = self.constructs_of_sort.get_mut(&sort).unwrap();
        if !cons_list.contains(&construct_id) {
            cons_list.push(construct_id);
        }
    }
}
