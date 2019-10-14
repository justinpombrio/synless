use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fmt::Debug;

use editor::{
    Ast, AstForest, AstRef, Clipboard, Doc, MetaCommand, NotationSet, TreeCmd, TreeNavCmd,
};
use forest::Bookmark;
use frontends::{Frontend, Terminal};
use language::{Language, LanguageName, LanguageSet};
use pretty::{DocLabel, Pane, PaneNotation};
use utility::GrowOnlyMap;

use crate::error::CoreError;

lazy_static! {
    pub static ref LANG_SET: LanguageSet = LanguageSet::new();
    pub static ref NOTE_SETS: GrowOnlyMap<LanguageName, NotationSet> = GrowOnlyMap::new();
}

/// This assumes that there is a single language associated with each Doc. That
/// might not be true forever! But it's a huge pain to create new nodes for the
/// Doc if we don't know what language to use.
struct DocEntry<'l> {
    doc: Doc<'l>,
    lang_name: LanguageName,
}

struct Docs<'l>(HashMap<DocLabel, DocEntry<'l>>);

impl<'l> Docs<'l> {
    fn new() -> Self {
        Self(HashMap::new())
    }

    fn insert(&mut self, label: DocLabel, doc: Doc<'l>, lang_name: LanguageName) {
        self.0.insert(label, DocEntry { doc, lang_name });
    }

    fn get_doc<'a>(&'a self, label: &DocLabel) -> Option<&'a Doc<'l>> {
        self.0.get(label).map(|entry| &entry.doc)
    }

    fn get_doc_mut<'a>(&'a mut self, label: &DocLabel) -> Option<&'a mut Doc<'l>> {
        self.0.get_mut(label).map(|entry| &mut entry.doc)
    }

    fn get_lang_name<'a>(&'a self, label: &DocLabel) -> Option<&'a LanguageName> {
        self.0.get(label).map(|entry| &entry.lang_name)
    }

    fn get_ast_ref<'a>(&'a self, label: &DocLabel) -> Option<AstRef<'a, 'l>> {
        self.get_doc(label).map(|doc| (doc.ast_ref()))
    }
}

pub struct Core<'l> {
    docs: Docs<'l>,
    forest: AstForest<'l>,
    bookmarks: HashMap<char, Bookmark>,
    cut_stack: Clipboard<'l>,
    pane_notation: PaneNotation,
}

impl<'l> Core<'l> {
    pub fn new(
        pane_notation: PaneNotation,
        keyhint_lang: (Language, NotationSet),
        message_lang: (Language, NotationSet),
        active_lang: (Language, NotationSet),
    ) -> Result<Self, CoreError<'l>> {
        let mut core = Core {
            docs: Docs::new(),
            forest: AstForest::new(&LANG_SET),
            cut_stack: Clipboard::new(),
            bookmarks: HashMap::new(),
            pane_notation,
        };
        let keyhint_lang_name = core.register_language(keyhint_lang);
        let message_lang_name = core.register_language(message_lang);
        let active_lang_name = core.register_language(active_lang);

        core.new_doc(DocLabel::KeyHints, "KeyHints", keyhint_lang_name)?;
        core.new_doc(
            DocLabel::KeymapName,
            "KeymapName",
            message_lang_name.clone(),
        )?;
        core.new_doc(DocLabel::ActiveDoc, "DemoDoc", active_lang_name)?;
        core.new_doc(DocLabel::Messages, "Messages", message_lang_name)?;
        core.clear_messages()?;
        Ok(core)
    }

    pub fn register_language(&self, lang_info: (Language, NotationSet)) -> LanguageName {
        let (lang, notation_set) = lang_info;
        let name = lang.name().to_owned();
        LANG_SET.insert(name.clone(), lang);
        NOTE_SETS.insert(name.clone(), notation_set);
        name
    }

    pub fn active_doc(&self) -> Result<&Doc<'l>, CoreError<'l>> {
        self.docs
            .get_doc(&DocLabel::ActiveDoc)
            .ok_or_else(|| CoreError::UnknownDocLabel(DocLabel::ActiveDoc))
    }

    pub fn lang_name_of<'a>(&'a self, label: &DocLabel) -> Result<&'a LanguageName, CoreError<'l>> {
        self.docs
            .get_lang_name(label)
            .ok_or_else(|| CoreError::UnknownDocLabel(label.to_owned()))
    }

    pub fn language(&self, lang_name: &LanguageName) -> Result<&'l Language, CoreError<'l>> {
        LANG_SET
            .get(lang_name)
            .ok_or_else(|| CoreError::UnknownLang(lang_name.to_owned()))
    }

    fn notation_set(&self, lang_name: &LanguageName) -> Result<&'l NotationSet, CoreError<'l>> {
        NOTE_SETS
            .get(lang_name)
            .ok_or_else(|| CoreError::UnknownLang(lang_name.to_owned()))
    }

    pub fn show_message(&mut self, msg: &str) -> Result<(), CoreError<'l>> {
        let mut msg_node = self.new_node_in_doc_lang("message", &DocLabel::Messages)?;
        msg_node.inner().unwrap_text().text_mut(|t| {
            t.activate();
            t.set(msg.to_owned());
            t.inactivate();
        });
        self.exec_on(TreeCmd::InsertHolePrepend, &DocLabel::Messages)?;
        self.exec_on(TreeCmd::Replace(msg_node), &DocLabel::Messages)?;
        self.exec_on(TreeNavCmd::Parent, &DocLabel::Messages)?;
        Ok(())
    }

    pub fn clear_messages(&mut self) -> Result<(), CoreError<'l>> {
        self.exec_on(
            TreeCmd::Replace(self.new_node_in_doc_lang("list", &DocLabel::Messages)?),
            &DocLabel::Messages,
        )
    }

    // TODO take generic Frontend. Requires type param in Error? Wait until the
    // `pretty` rewrite is merged.
    //
    // pub fn redisplay<F>(&self, window: &mut F) -> Result<(), Error>
    // where
    //     F: Frontend,
    // {
    pub fn redisplay(&self, frontend: &mut Terminal) -> Result<(), CoreError<'l>> {
        frontend.draw_frame(|mut pane: Pane<<Terminal as Frontend>::Window>| {
            pane.render(&self.pane_notation, |label: &DocLabel| {
                self.docs.get_ast_ref(label)
            })
        })?;
        Ok(())
    }

    /// Create a new node in the same language as the given doc.
    pub fn new_node_in_doc_lang(
        &self,
        construct_name: &str,
        doc_label: &DocLabel,
    ) -> Result<Ast<'l>, CoreError<'l>> {
        self.new_node(construct_name, self.lang_name_of(doc_label)?)
    }

    pub fn new_node(
        &self,
        construct_name: &str,
        lang_name: &LanguageName,
    ) -> Result<Ast<'l>, CoreError<'l>> {
        let construct_name = construct_name.to_string();
        let lang = self.language(lang_name)?;
        let notes = self.notation_set(lang_name)?;

        self.forest
            .new_tree(lang, &construct_name, notes)
            .ok_or_else(|| CoreError::UnknownConstruct {
                construct: construct_name.to_owned(),
                lang: lang_name.to_owned(),
            })
    }

    pub fn exec<T>(&mut self, cmd: T) -> Result<(), CoreError<'l>>
    where
        T: Debug + Into<MetaCommand<'l>>,
    {
        self.exec_on(cmd.into(), &DocLabel::ActiveDoc)
    }

    pub fn exec_on<T>(&mut self, cmd: T, doc_label: &DocLabel) -> Result<(), CoreError<'l>>
    where
        T: Debug + Into<MetaCommand<'l>>,
    {
        self.docs
            .get_doc_mut(doc_label)
            .ok_or_else(|| CoreError::UnknownDocLabel(doc_label.to_owned()))?
            .execute(cmd.into(), &mut self.cut_stack)?;
        Ok(())
    }

    pub fn add_bookmark(&mut self, name: char, doc_label: &DocLabel) -> Result<(), CoreError<'l>> {
        let mark = self
            .docs
            .get_doc_mut(doc_label)
            .ok_or_else(|| CoreError::UnknownDocLabel(doc_label.to_owned()))?
            .bookmark();
        self.bookmarks.insert(name, mark);
        Ok(())
    }

    pub fn get_bookmark(&mut self, name: char) -> Result<Bookmark, CoreError<'l>> {
        // TODO handle bookmarks into multiple documents
        self.bookmarks
            .get(&name)
            .cloned()
            .ok_or(CoreError::UnknownBookmark)
    }

    /// Create and store a new document. The document will consist of a root
    /// with a hole as its child, and the cursor will be positioned on the hole.
    fn new_doc(
        &mut self,
        label: DocLabel,
        doc_name: &str,
        lang_name: LanguageName,
    ) -> Result<(), CoreError<'l>> {
        let mut root_node = self.new_node("root", &lang_name)?;
        let hole = root_node.new_hole();
        root_node
            .inner()
            .unwrap_fixed()
            .replace_child(0, hole)
            .unwrap();
        root_node.inner().unwrap_fixed().goto_child(0);

        self.docs
            .insert(label, Doc::new(doc_name, root_node), lang_name);
        Ok(())
    }
}
