use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fmt::Debug;

use editor::{
    make_json_lang, Ast, AstForest, AstRef, Clipboard, Doc, MetaCommand, NotationSet, TreeCmd,
    TreeNavCmd,
};
use forest::Bookmark;
use frontends::{Frontend, Terminal};
use language::{Language, LanguageName, LanguageSet};
use pretty::{Color, CursorVis, DocLabel, DocPosSpec, Pane, PaneNotation, PaneSize, Style};
use utility::GrowOnlyMap;

use crate::error::CoreError;
use crate::keymap_lang::make_keymap_lang;
use crate::message_lang::make_message_lang;

lazy_static! {
    pub static ref LANG_SET: LanguageSet = LanguageSet::new();
    pub static ref NOTE_SETS: GrowOnlyMap<LanguageName, NotationSet> = GrowOnlyMap::new();
}

struct DocEntry<'l> {
    doc: Doc<'l>,
    lang_name: LanguageName,
}

pub struct Docs<'l> {
    map: HashMap<DocLabel, DocEntry<'l>>,
}

impl<'l> Docs<'l> {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    // fn insert(&mut self, name: DocLabel, doc: Doc<'l>, lang_name: LanguageName) {
    //     self.map.insert(name, DocEntry { doc, lang_name });
    // }
    // TODO keep entry private
    fn insert(&mut self, label: DocLabel, entry: DocEntry<'l>) {
        self.map.insert(label, entry);
    }

    fn get<'a>(&'a self, label: &DocLabel) -> Option<&'a Doc<'l>> {
        self.map.get(label).map(|entry| &entry.doc)
    }

    fn get_mut<'a>(&'a mut self, label: &DocLabel) -> Option<&'a mut Doc<'l>> {
        self.map.get_mut(label).map(|entry| &mut entry.doc)
    }

    pub fn lang_name<'a>(&'a self, label: &DocLabel) -> Option<&'a LanguageName> {
        self.map.get(label).map(|entry| &entry.lang_name)
    }

    fn content<'a>(&'a self, label: &DocLabel) -> Option<AstRef<'a, 'l>> {
        self.get(label).map(|doc| (doc.ast_ref()))
    }

    // TODO get rid of these lazy specific getters
    pub fn active(&self) -> &Doc<'l> {
        &self.get(&DocLabel::ActiveDoc).expect("no active doc")
    }

    fn active_mut(&mut self) -> &mut Doc<'l> {
        self.get_mut(&DocLabel::ActiveDoc).expect("no active doc")
    }
}

// TODO all private
pub struct Core {
    pub docs: Docs<'static>,
    forest: AstForest<'static>,
    bookmarks: HashMap<char, Bookmark>,
    cut_stack: Clipboard<'static>,
}

impl Core {
    pub fn new() -> Result<Self, CoreError> {
        let (json_lang, json_notes) = make_json_lang();
        let (kmap_lang, kmap_notes) = make_keymap_lang();
        let (msg_lang, msg_notes) = make_message_lang();
        let json_lang_name = json_lang.name().to_owned();
        let kmap_lang_name = kmap_lang.name().to_owned();
        let msg_lang_name = msg_lang.name().to_owned();

        LANG_SET.insert(json_lang_name.clone(), json_lang);
        LANG_SET.insert(kmap_lang_name.clone(), kmap_lang);
        LANG_SET.insert(msg_lang_name.clone(), msg_lang);
        NOTE_SETS.insert(json_lang_name.clone(), json_notes);
        NOTE_SETS.insert(kmap_lang_name.clone(), kmap_notes);
        NOTE_SETS.insert(msg_lang_name.clone(), msg_notes);

        let mut core = Core {
            docs: Docs::new(),
            forest: AstForest::new(&LANG_SET),
            cut_stack: Clipboard::new(),
            bookmarks: HashMap::new(),
        };
        core.new_doc(DocLabel::KeyHints, "KeyHints", kmap_lang_name.clone())?;
        core.new_doc(DocLabel::KeymapName, "KeymapName", msg_lang_name.clone())?;
        core.new_doc(DocLabel::ActiveDoc, "DemoJsonDoc", json_lang_name.clone())?;
        core.new_doc(DocLabel::Messages, "Messages", msg_lang_name)?;
        core.clear_messages()?;
        Ok(core)
    }

    pub fn lang_name<'a>(&'a self, label: &DocLabel) -> Option<&'a LanguageName> {
        self.docs.lang_name(label)
    }

    pub fn language(&self, lang_name: &LanguageName) -> Result<&'static Language, CoreError> {
        LANG_SET
            .get(lang_name)
            .ok_or_else(|| CoreError::UnknownLang(lang_name.to_owned()))
    }

    fn notation_set(&self, lang_name: &LanguageName) -> Result<&'static NotationSet, CoreError> {
        NOTE_SETS
            .get(lang_name)
            .ok_or_else(|| CoreError::UnknownLang(lang_name.to_owned()))
    }

    pub fn msg(&mut self, msg: &str) -> Result<(), CoreError> {
        let mut msg_node = self.node_in_doc_lang("message", &DocLabel::Messages)?;
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

    pub fn clear_messages(&mut self) -> Result<(), CoreError> {
        self.exec_on(
            TreeCmd::Replace(self.node_in_doc_lang("list", &DocLabel::Messages)?),
            &DocLabel::Messages,
        )
    }

    fn pane_notation(&self) -> PaneNotation {
        let active = PaneNotation::Doc {
            label: DocLabel::ActiveDoc,
            cursor_visibility: CursorVis::Show,
            scroll_strategy: DocPosSpec::CursorHeight { fraction: 0.6 },
        };

        let key_hints_name = PaneNotation::Doc {
            label: DocLabel::KeymapName,
            cursor_visibility: CursorVis::Hide,
            scroll_strategy: DocPosSpec::Beginning,
        };

        let key_hints = PaneNotation::Doc {
            label: DocLabel::KeyHints,
            cursor_visibility: CursorVis::Hide,
            scroll_strategy: DocPosSpec::Beginning,
        };

        let messages = PaneNotation::Doc {
            label: DocLabel::Messages,
            cursor_visibility: CursorVis::Hide,
            scroll_strategy: DocPosSpec::Beginning,
        };

        let divider = PaneNotation::Fill {
            ch: '=',
            style: Style::color(Color::Base03),
        };

        let status_bar = PaneNotation::Horz {
            panes: vec![
                (PaneSize::Proportional(1), divider.clone()),
                (PaneSize::Proportional(1), key_hints_name),
                (PaneSize::Proportional(1), divider.clone()),
            ],
        };

        PaneNotation::Vert {
            panes: vec![
                (PaneSize::Proportional(1), active),
                (PaneSize::Fixed(1), status_bar),
                (PaneSize::DynHeight, key_hints),
                (PaneSize::Fixed(1), divider),
                (PaneSize::Fixed(5), messages),
            ],
        }
    }

    // TODO take generic Frontend. Requires type param in Error? Wait until the
    // `pretty` rewrite is merged.
    //
    // pub fn redisplay<F>(&self, window: &mut F) -> Result<(), Error>
    // where
    //     F: Frontend,
    // {
    pub fn redisplay(&self, frontend: &mut Terminal) -> Result<(), CoreError> {
        let notation = self.pane_notation();
        frontend.draw_frame(|mut pane: Pane<<Terminal as Frontend>::Window>| {
            pane.render(&notation, |label: &DocLabel| self.docs.content(label))
        })?;
        Ok(())
    }

    /// Create a new node in the same language as the given doc.
    pub fn node_in_doc_lang(
        &self,
        construct_name: &str,
        doc_label: &DocLabel,
    ) -> Result<Ast<'static>, CoreError> {
        let lang_name = self
            .lang_name(doc_label)
            .ok_or_else(|| CoreError::UnknownDocLabel(doc_label.to_owned()))?;
        self.node_by_name(construct_name, lang_name)
    }

    pub fn node_by_name(
        &self,
        construct_name: &str,
        lang_name: &LanguageName,
    ) -> Result<Ast<'static>, CoreError> {
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

    pub fn exec<T>(&mut self, cmd: T) -> Result<(), CoreError>
    where
        T: Debug + Into<MetaCommand<'static>>,
    {
        self.exec_on(cmd.into(), &DocLabel::ActiveDoc)
    }

    pub fn exec_on<T>(&mut self, cmd: T, doc_label: &DocLabel) -> Result<(), CoreError>
    where
        T: Debug + Into<MetaCommand<'static>>,
    {
        self.docs
            .get_mut(doc_label)
            .ok_or_else(|| CoreError::UnknownDocLabel(doc_label.to_owned()))?
            .execute(cmd.into(), &mut self.cut_stack)?;
        Ok(())
    }

    pub fn add_bookmark(&mut self, name: char) {
        let mark = self.docs.active_mut().bookmark();
        self.bookmarks.insert(name, mark);
    }

    pub fn get_bookmark(&mut self, name: char) -> Result<Bookmark, CoreError> {
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
    ) -> Result<(), CoreError> {
        let mut root_node = self.node_by_name("root", &lang_name)?;
        let hole = root_node.new_hole();
        root_node
            .inner()
            .unwrap_fixed()
            .replace_child(0, hole)
            .unwrap();
        root_node.inner().unwrap_fixed().goto_child(0);

        self.docs.insert(
            label,
            DocEntry {
                doc: Doc::new(doc_name, root_node),
                lang_name,
            },
        );
        Ok(())
    }

    pub fn in_tree_mode(&self) -> bool {
        self.docs.active().in_tree_mode()
    }
}
