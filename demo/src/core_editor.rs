use lazy_static::lazy_static;
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;

use editor::{make_json_lang, Ast, AstForest, Clipboard, Doc, MetaCommand, NotationSet};
use forest::Bookmark;
use frontends::{Frontend, Terminal};
use language::{Language, LanguageName, LanguageSet};
use pretty::{Color, CursorVis, DocLabel, DocPosSpec, Pane, PaneNotation, PaneSize, Style};
use utility::GrowOnlyMap;

use crate::error::Error;
use crate::keymap_lang::make_keymap_lang;
use crate::message_lang::make_message_lang;

lazy_static! {
    pub static ref LANG_SET: LanguageSet = LanguageSet::new();
    pub static ref NOTE_SETS: GrowOnlyMap<LanguageName, NotationSet> = GrowOnlyMap::new();
}

// TODO combine with DocLabel? After pretty rewrite?
#[derive(Clone, Eq, PartialEq, Hash)]
pub enum DocName {
    Active,
    KeyHints,
    Messages,
}

struct DocEntry<'l> {
    doc: Doc<'l>,
    lang_name: LanguageName,
}

pub struct Docs<'l> {
    map: HashMap<DocName, DocEntry<'l>>,
}

impl<'l> Docs<'l> {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    // fn insert(&mut self, name: DocName, doc: Doc<'l>, lang_name: LanguageName) {
    //     self.map.insert(name, DocEntry { doc, lang_name });
    // }
    // TODO keep entry private
    fn insert(&mut self, name: DocName, entry: DocEntry<'l>) {
        self.map.insert(name, entry);
    }

    fn get<'a>(&'a self, name: &DocName) -> Option<&'a Doc<'l>> {
        self.map.get(name).map(|entry| &entry.doc)
    }

    fn get_mut<'a>(&'a mut self, name: &DocName) -> Option<&'a mut Doc<'l>> {
        self.map.get_mut(name).map(|entry| &mut entry.doc)
    }

    pub fn lang_name<'a>(&'a self, name: &DocName) -> Option<&'a LanguageName> {
        self.map.get(name).map(|entry| &entry.lang_name)
    }

    // TODO get rid of these lazy specific getters
    pub fn active(&self) -> &Doc<'l> {
        &self.get(&DocName::Active).expect("no active doc")
    }

    fn active_mut(&mut self) -> &mut Doc<'l> {
        self.get_mut(&DocName::Active).expect("no active doc")
    }

    pub fn key_hints(&self) -> &Doc<'l> {
        self.get(&DocName::KeyHints).expect("no key hints doc")
    }
    pub fn messages(&self) -> &Doc<'l> {
        self.get(&DocName::Messages).expect("no messages doc")
    }
}

// TODO all private
pub struct Core {
    pub docs: Docs<'static>,
    forest: AstForest<'static>,
    messages: VecDeque<String>,
    bookmarks: HashMap<char, Bookmark>,
    cut_stack: Clipboard<'static>,
}

impl Core {
    pub fn new() -> Result<Self, Error> {
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

        let forest = AstForest::new(&LANG_SET);

        let mut docs = Docs::new();
        docs.insert(
            DocName::Active,
            DocEntry::new("DemoJsonDoc", json_lang_name.clone(), &forest),
        );
        docs.insert(
            DocName::KeyHints,
            DocEntry::new("(no keymap)", kmap_lang_name.clone(), &forest),
        );
        docs.insert(
            DocName::Messages,
            DocEntry::new("Messages", msg_lang_name.clone(), &forest),
        );
        Ok(Core {
            docs,
            forest,
            messages: VecDeque::new(),
            cut_stack: Clipboard::new(),
            bookmarks: HashMap::new(),
        })
    }

    pub fn lang_name<'a>(&'a self, doc_name: &DocName) -> Option<&'a LanguageName> {
        self.docs.lang_name(doc_name)
    }

    pub fn lang(&self, lang_name: &LanguageName) -> Option<&'static Language> {
        LANG_SET.get(lang_name)
    }

    pub fn msg(&mut self, msg: &str) -> Result<(), Error> {
        self.messages.push_front(msg.to_owned());
        self.messages.truncate(5);

        let lang_name = self
            .lang_name(&DocName::Messages)
            .expect("no messages lang");

        let mut list_node = self.node_by_name("list", lang_name)?;
        for (i, msg) in self.messages.iter().enumerate() {
            let mut msg_node = self.node_by_name("message", lang_name)?;
            msg_node.inner().unwrap_text().text_mut(|t| {
                t.activate();
                t.set(msg.to_owned());
                t.inactivate();
            });
            list_node
                .inner()
                .unwrap_flexible()
                .insert_child(i, msg_node)
                .unwrap();
        }
        let mut root_node = self.node_by_name("root", lang_name)?;
        root_node
            .inner()
            .unwrap_fixed()
            .replace_child(0, list_node)
            .unwrap();

        self.set_doc(
            DocName::Messages,
            // TODO don't hardcode name
            Doc::new("Messages", root_node),
            lang_name.to_owned(),
        );
        Ok(())
    }

    pub fn clear_messages(&mut self) {
        self.messages.clear()
    }

    fn pane_notation(&self) -> PaneNotation {
        let doc = PaneNotation::Doc {
            label: DocLabel::ActiveDoc,
        };

        let doc_name = PaneNotation::Doc {
            label: DocLabel::ActiveDocName,
        };

        let key_hints_name = PaneNotation::Doc {
            label: DocLabel::KeymapName,
        };

        let key_hints = PaneNotation::Doc {
            label: DocLabel::KeyHints,
        };

        let messages = PaneNotation::Doc {
            label: DocLabel::Messages,
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
                (PaneSize::Proportional(1), doc_name),
                (PaneSize::Proportional(1), divider.clone()),
            ],
        };

        PaneNotation::Vert {
            panes: vec![
                (PaneSize::Proportional(1), doc),
                (PaneSize::Fixed(1), status_bar),
                (PaneSize::DynHeight, key_hints),
                (PaneSize::Fixed(1), divider),
                (PaneSize::DynHeight, messages),
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
    pub fn redisplay(&self, frontend: &mut Terminal) -> Result<(), Error> {
        let notation = self.pane_notation();
        let doc = self.docs.active().ast_ref();
        let doc_name_ast = self.to_ast(self.docs.active().name())?;
        let doc_name = doc_name_ast.ast_ref();
        let key_hints = self.docs.key_hints().ast_ref();
        let key_hints_name_ast = self.to_ast(self.docs.key_hints().name())?;
        let key_hints_name = key_hints_name_ast.ast_ref();
        let messages = self.docs.messages().ast_ref();
        frontend.draw_frame(|mut pane: Pane<<Terminal as Frontend>::Window>| {
            pane.render(&notation, |label: &DocLabel| match label {
                DocLabel::ActiveDoc => Some((
                    doc.clone(),
                    CursorVis::Show,
                    DocPosSpec::CursorHeight { fraction: 0.6 },
                )),
                DocLabel::ActiveDocName => {
                    Some((doc_name.clone(), CursorVis::Hide, DocPosSpec::Beginning))
                }
                DocLabel::KeymapName => Some((
                    key_hints_name.clone(),
                    CursorVis::Hide,
                    DocPosSpec::Beginning,
                )),
                DocLabel::KeyHints => {
                    Some((key_hints.clone(), CursorVis::Hide, DocPosSpec::Beginning))
                }
                DocLabel::Messages => {
                    Some((messages.clone(), CursorVis::Hide, DocPosSpec::Beginning))
                }
                _ => None,
            })?;
            Ok(())
        })?;
        Ok(())
    }

    pub fn node_by_name(
        &self,
        construct_name: &str,
        lang_name: &LanguageName,
    ) -> Result<Ast<'static>, Error> {
        let construct_name = construct_name.to_string();
        let lang = LANG_SET
            .get(lang_name)
            .ok_or_else(|| Error::UnknownLang(lang_name.to_owned()))?;
        let notes = NOTE_SETS
            .get(lang_name)
            .ok_or_else(|| Error::UnknownLang(lang_name.to_owned()))?;

        self.forest
            .new_tree(lang, &construct_name, notes)
            .ok_or_else(|| Error::UnknownConstruct {
                construct: construct_name.to_owned(),
                lang: lang_name.to_owned(),
            })
    }

    /// Create a quick and dirty Ast for storing only this string.
    fn to_ast<T: Into<String>>(&self, text: T) -> Result<Ast<'static>, Error> {
        let lang_name = self
            .lang_name(&DocName::Messages)
            .expect("no messages lang");
        let mut text_node = self.node_by_name("message", &lang_name)?;
        text_node.inner().unwrap_text().text_mut(|t| {
            t.activate();
            t.set(text.into());
            t.inactivate();
        });
        let mut root_node = self.node_by_name("root", &lang_name)?;
        root_node
            .inner()
            .unwrap_fixed()
            .replace_child(0, text_node)
            .unwrap();
        Ok(root_node)
    }

    // TODO exec on things other than the active doc?
    pub fn exec<T>(&mut self, cmd: T) -> Result<(), Error>
    where
        T: Debug + Into<MetaCommand<'static>>,
    {
        self.docs
            .active_mut()
            .execute(cmd.into(), &mut self.cut_stack)?;
        Ok(())
    }

    pub fn add_bookmark(&mut self, name: char) {
        let mark = self.docs.active_mut().bookmark();
        self.bookmarks.insert(name, mark);
    }

    pub fn get_bookmark(&mut self, name: char) -> Result<Bookmark, Error> {
        // TODO handle bookmarks into multiple documents
        self.bookmarks
            .get(&name)
            .cloned()
            .ok_or(Error::UnknownBookmark)
    }
    pub fn set_doc(&mut self, doc_name: DocName, doc: Doc<'static>, lang_name: LanguageName) {
        self.docs.insert(doc_name, DocEntry { doc, lang_name });
    }

    pub fn in_tree_mode(&self) -> bool {
        self.docs.active().in_tree_mode()
    }
}

impl<'l> DocEntry<'l> {
    // TODO rename or move, new doc!
    fn new(doc_name: &str, lang_name: LanguageName, forest: &AstForest<'l>) -> DocEntry<'l> {
        let lang = LANG_SET.get(&lang_name).unwrap();
        let doc = Doc::new(
            doc_name,
            forest.new_fixed_tree(
                lang,
                lang.lookup_construct("root"),
                NOTE_SETS.get(&lang_name).unwrap(),
            ),
        );
        DocEntry { doc, lang_name }
    }
}
