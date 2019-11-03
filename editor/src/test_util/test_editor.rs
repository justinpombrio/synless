use crate::{Ast, AstForest, AstRef, Clipboard, Doc, DocError, MetaCommand, NotationSet};
use language::{ConstructName, Language, LanguageName, LanguageSet};
use pretty::{
    Col, CursorVisibility, PlainText, Pos, PrettyDocument, PrettyWindow, RenderOptions, Row,
    ScrollStrategy, WidthStrategy,
};

/// A simple wrapper around a Doc that makes it more convenient to write tests
/// that execute commands to edit the document and check if the document
/// is rendered correctly.
pub struct TestEditor<'l> {
    pub doc: Doc<'l>,
    pub clipboard: Clipboard<'l>,
    forest: AstForest<'l>,
    /// Which language in the `lang_set` to use for the `doc`.
    lang_name: LanguageName,
    /// Must at least contain the language `lang_name`.
    lang_set: &'l LanguageSet,
    /// Notations for the `lang_name` language.
    note_set: &'l NotationSet,
}

impl<'l> TestEditor<'l> {
    /// Create a new TestEditor containing a Doc in the given language.
    pub fn new(
        lang_set: &'l LanguageSet,
        note_set: &'l NotationSet,
        lang_name: LanguageName,
    ) -> Self {
        let lang = lang_set.get(&lang_name).unwrap();
        let forest = AstForest::new(&lang_set);

        let doc = Doc::new(
            "MyTestDoc",
            forest.new_fixed_tree(&lang, lang.lookup_construct(&"root".into()), note_set),
        );

        TestEditor {
            doc,
            clipboard: Clipboard::new(),
            forest,
            note_set,
            lang_set,
            lang_name,
        }
    }

    /// Execute the given command or meta-command, and return its result.
    pub fn exec<T>(&mut self, cmd: T) -> Result<(), DocError>
    where
        T: Into<MetaCommand<'l>>,
    {
        self.doc.execute(cmd.into(), &mut self.clipboard)
    }

    /// Try to create a new node in the forest with the given construct name.
    pub fn node(&self, construct_name: &ConstructName) -> Option<Ast<'l>> {
        let lang = self.lang_set.get(&self.lang_name).unwrap();
        self.forest.new_tree(lang, construct_name, self.note_set)
    }

    /// Render the Doc as a string, and assert that it's equal to the `expected`
    /// string. Use a default width and doc position for rendering.
    pub fn assert_render(&self, expected: &str) {
        self.assert_render_with(expected, 80, ScrollStrategy::Fixed(Pos::zero()))
    }

    /// Render the Doc as a string, and assert that it's equal to the `expected`
    /// string. Use the given width and doc position for rendering.
    pub fn assert_render_with(&self, expected: &str, width: Col, scroll_strategy: ScrollStrategy) {
        let mut window = PlainText::new(Pos {
            col: width,
            row: Row::max_value() / 2,
        });

        let options = RenderOptions {
            scroll_strategy,
            width_strategy: WidthStrategy::Fixed(width),
            cursor_visibility: CursorVisibility::Hide,
        };

        self.doc
            .ast_ref()
            .pretty_print(&mut window.pane().unwrap(), options)
            .unwrap();
        assert_eq!(window.to_string(), expected)
    }

    pub fn ast_ref<'a>(&'a self) -> AstRef<'l, 'a> {
        self.doc.ast_ref()
    }
}

/// Create a new LanguageSet containing only the given Language. Return the
/// LanguageSet along with the name of the given Language.
pub fn make_singleton_lang_set(lang: Language) -> (LanguageSet, LanguageName) {
    let lang_name = lang.name().to_owned();
    let lang_set = LanguageSet::new();
    lang_set.insert(lang_name.clone(), lang);
    (lang_set, lang_name)
}
