use lazy_static::lazy_static;
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;

use termion::event::Key;

use editor::{
    make_json_lang, Ast, AstForest, Clipboard, Doc, EditorCmd, MetaCommand, NotationSet, TextCmd,
    TextNavCmd, TreeCmd, TreeNavCmd,
};
use forest::Bookmark;
use frontends::{Event, Frontend, Terminal};
use language::{LanguageName, LanguageSet, Sort};
use pretty::{
    Color, ColorTheme, CursorVis, DocLabel, DocPosSpec, Pane, PaneNotation, PaneSize, Style,
};
use utility::GrowOnlyMap;

mod demo_keymaps;
mod error;
mod keymap;
mod keymap_lang;
mod message_lang;
mod prog;

use error::Error;
use keymap::{Kmap, TreeKmapFactory};
use keymap_lang::make_keymap_lang;
use message_lang::make_message_lang;
use prog::{DataStack, KmapSpec, Value, Word};

lazy_static! {
    pub static ref LANG_SET: LanguageSet = LanguageSet::new();
    pub static ref NOTE_SETS: GrowOnlyMap<LanguageName, NotationSet> = GrowOnlyMap::new();
}

fn main() -> Result<(), Error> {
    let mut ed = Ed::new()?;
    let err = ed.run();
    drop(ed);
    println!("Error: {:?}", err);
    println!("Exited alternate screen. Your cursor should be visible again.");
    Ok(())
}

/// Demonstrate a basic interactive tree editor
struct Ed {
    doc: Doc<'static>,
    lang_name: LanguageName,
    forest: AstForest<'static>,
    term: Terminal,
    messages: VecDeque<String>,
    message_doc: Doc<'static>,
    message_lang_name: LanguageName,
    data_stack: DataStack<'static>,
    tree_keymaps: HashMap<String, TreeKmapFactory<'static>>,
    tree_keymap_stack: Vec<KmapSpec>,
    text_keymap: Kmap<'static>,
    kmap_lang_name: LanguageName,
    key_hints: Doc<'static>,
    cut_stack: Clipboard<'static>,
    bookmarks: HashMap<char, Bookmark>,
}

impl Ed {
    fn new() -> Result<Self, Error> {
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

        let doc = new_doc("DemoJsonDoc", &json_lang_name, &forest);
        let key_hints = new_doc("(no keymap)", &kmap_lang_name, &forest);
        let msg_doc = new_doc("Messages", &msg_lang_name, &forest);

        let mut tree_keymaps = HashMap::new();
        tree_keymaps.insert("tree".to_string(), demo_keymaps::make_tree_map());
        tree_keymaps.insert(
            "speed_bool".to_string(),
            demo_keymaps::make_speed_bool_map(),
        );
        tree_keymaps.insert(
            "node".to_string(),
            demo_keymaps::make_node_map(LANG_SET.get(&json_lang_name).unwrap()),
        );

        let mut ed = Ed {
            doc,
            lang_name: json_lang_name,
            forest,
            term: Terminal::new(ColorTheme::default_dark())?,
            messages: VecDeque::new(),
            message_doc: msg_doc,
            message_lang_name: msg_lang_name,
            data_stack: DataStack::new(),
            tree_keymaps,
            text_keymap: demo_keymaps::make_text_map(),
            kmap_lang_name,
            key_hints,
            tree_keymap_stack: Vec::new(),
            cut_stack: Clipboard::new(),
            bookmarks: HashMap::new(),
        };

        // Set initial keymap
        ed.call(Word::Literal(Value::MapName("tree".into())))?;
        ed.call(Word::AnySort)?;
        ed.call(Word::PushMap)?;

        // Add an empty list to the document
        ed.call(Word::Literal(Value::Usize(0)))?;
        ed.call(Word::Child)?;
        ed.call(Word::Literal(Value::LangConstruct(
            ed.lang_name.clone(),
            "list".into(),
        )))?;
        ed.call(Word::NodeByName)?;
        ed.call(Word::Replace)?;

        ed.messages.clear();
        Ok(ed)
    }

    fn run(&mut self) -> Result<(), Error> {
        loop {
            self.redisplay()?;
            self.handle_event()?;
        }
    }

    fn msg(&mut self, msg: &str) -> Result<(), Error> {
        self.messages.push_front(msg.to_owned());
        self.messages.truncate(5);

        let mut list_node = self.node_by_name("list", &self.message_lang_name)?;
        for (i, msg) in self.messages.iter().enumerate() {
            let mut msg_node = self.node_by_name("message", &self.message_lang_name)?;
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
        let mut root_node = self.node_by_name("root", &self.message_lang_name)?;
        root_node
            .inner()
            .unwrap_fixed()
            .replace_child(0, list_node)
            .unwrap();
        self.message_doc = Doc::new(self.message_doc.name(), root_node);
        Ok(())
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

    fn redisplay(&mut self) -> Result<(), Error> {
        let notation = self.pane_notation();
        let doc = self.doc.ast_ref();
        let doc_name_ast = self.to_ast(self.doc.name())?;
        let doc_name = doc_name_ast.ast_ref();
        let key_hints = self.key_hints.ast_ref();
        let key_hints_name_ast = self.to_ast(self.key_hints.name())?;
        let key_hints_name = key_hints_name_ast.ast_ref();
        let messages = self.message_doc.ast_ref();
        self.term.draw_frame(|mut pane: Pane<Terminal>| {
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

    fn active_keymap(&self) -> Result<Kmap<'static>, Error> {
        if self.doc.in_tree_mode() {
            let kmap = self.tree_keymap_stack.last().ok_or(Error::NoKeymap)?;
            Ok(self
                .tree_keymaps
                .get(&kmap.name)
                .ok_or_else(|| Error::UnknownKeymap(kmap.name.to_owned()))?
                .filter(self.doc.ast_ref(), &kmap.required_sort))
        } else {
            // TODO avoid cloning every time!
            Ok(self.text_keymap.clone())
        }
    }

    fn update_key_hints(&mut self) -> Result<(), Error> {
        let mut dict_node = self.node_by_name("dict", &self.kmap_lang_name)?;

        for (key, prog) in self.active_keymap()?.hints() {
            let mut key_node = self.node_by_name("key", &self.kmap_lang_name)?;
            key_node.inner().unwrap_text().text_mut(|t| {
                t.activate();
                t.set(key);
                t.inactivate();
            });

            let mut prog_node = self.node_by_name("prog", &self.kmap_lang_name)?;
            prog_node.inner().unwrap_text().text_mut(|t| {
                t.activate();
                t.set(prog);
                t.inactivate();
            });

            let mut entry_node = self.node_by_name("entry", &self.kmap_lang_name)?;
            entry_node
                .inner()
                .unwrap_fixed()
                .replace_child(0, key_node)
                .unwrap();
            entry_node
                .inner()
                .unwrap_fixed()
                .replace_child(1, prog_node)
                .unwrap();
            let mut inner_dict = dict_node.inner().unwrap_flexible();
            inner_dict
                .insert_child(inner_dict.num_children(), entry_node)
                .unwrap();
        }
        let mut root_node = self.node_by_name("root", &self.kmap_lang_name)?;
        root_node
            .inner()
            .unwrap_fixed()
            .replace_child(0, dict_node)
            .unwrap();

        let kmap_name = if self.doc.in_tree_mode() {
            let mut s = String::new();
            for (i, spec) in self.tree_keymap_stack.iter().enumerate() {
                if i != 0 {
                    s += "→";
                }
                s += &spec.name;
            }
            s
        } else {
            "text".to_owned()
        };

        self.key_hints = Doc::new(&kmap_name, root_node);
        Ok(())
    }

    fn handle_key(&mut self, key: Key) -> Result<(), Error> {
        let prog = self.active_keymap()?.lookup(key)?;
        for word in prog.words {
            self.call(word)?;
        }
        Ok(())
    }

    fn handle_event(&mut self) -> Result<(), Error> {
        match self.term.next_event() {
            Some(Ok(Event::KeyEvent(Key::Ctrl('c')))) => Err(Error::KeyboardInterrupt),
            Some(Ok(Event::KeyEvent(key))) => {
                if let Err(err) = self.handle_key(key) {
                    self.msg(&format!("Error: {:?}", err))?;
                }
                Ok(())
            }
            Some(Err(err)) => Err(err.into()),
            _ => Err(Error::UnknownEvent),
        }
    }

    fn call(&mut self, word: Word<'static>) -> Result<(), Error> {
        Ok(match word {
            Word::Literal(value) => self.data_stack.push(value),
            Word::Apply => {
                let word = self.data_stack.pop_quote()?;
                self.call(word)?;
            }
            Word::Swap => {
                self.data_stack.swap()?;
            }
            Word::Pop => {
                self.data_stack.pop()?;
            }
            Word::Echo => {
                let message = self.data_stack.pop_message()?;
                self.msg(&message)?;
            }
            Word::NodeByName => {
                let (lang_name, construct_name) = self.data_stack.pop_lang_construct()?;
                let node = self.node_by_name(&construct_name, &lang_name)?;
                self.data_stack.push(Value::Tree(node));
            }
            Word::PushMap => {
                let sort = self.data_stack.pop_sort()?;
                let name = self.data_stack.pop_map_name()?;
                self.tree_keymap_stack.push(KmapSpec {
                    name,
                    required_sort: sort,
                });
                self.update_key_hints()?;
            }
            Word::PopMap => {
                self.tree_keymap_stack.pop();
                self.update_key_hints()?;
            }
            Word::ChildSort => {
                let sort = self.doc.ast_ref().arity().uniform_child_sort().to_owned();
                self.data_stack.push(Value::Sort(sort));
            }
            Word::SelfSort => {
                let (parent, index) = self
                    .doc
                    .ast_ref()
                    .parent()
                    .expect("you shouldn't be at the root!");
                let sort = parent.arity().child_sort(index).to_owned();
                self.data_stack.push(Value::Sort(sort));
            }
            Word::SiblingSort => {
                let (parent, _) = self
                    .doc
                    .ast_ref()
                    .parent()
                    .expect("you shouldn't be at the root!");
                let sort = parent.arity().uniform_child_sort().to_owned();
                self.data_stack.push(Value::Sort(sort));
            }
            Word::AnySort => {
                self.data_stack.push(Value::Sort(Sort::any()));
            }
            Word::Remove => self.exec(TreeCmd::Remove)?,
            Word::Clear => self.exec(TreeCmd::Clear)?,
            Word::InsertHoleAfter => {
                self.exec(TreeCmd::InsertHoleAfter)?;
            }
            Word::InsertHoleBefore => {
                self.exec(TreeCmd::InsertHoleBefore)?;
            }
            Word::InsertHolePrepend => {
                self.exec(TreeCmd::InsertHolePrepend)?;
            }
            Word::InsertHolePostpend => {
                self.exec(TreeCmd::InsertHolePostpend)?;
            }
            Word::Replace => {
                let tree = self.data_stack.pop_tree()?;
                self.exec(TreeCmd::Replace(tree))?;
            }
            Word::Left => self.exec(TreeNavCmd::Left)?,
            Word::Right => self.exec(TreeNavCmd::Right)?,
            Word::Parent => self.exec(TreeNavCmd::Parent)?,
            Word::Child => {
                let index = self.data_stack.pop_usize()?;
                self.exec(TreeNavCmd::Child(index))?;
            }
            Word::Undo => self.exec(MetaCommand::Undo)?,
            Word::Redo => self.exec(MetaCommand::Redo)?,
            Word::Cut => self.exec(EditorCmd::Cut)?,
            Word::Copy => self.exec(EditorCmd::Copy)?,
            Word::PasteSwap => self.exec(EditorCmd::PasteSwap)?,
            Word::PopClipboard => self.exec(EditorCmd::PopClipboard)?,
            Word::GotoBookmark => {
                let name = self.data_stack.pop_char()?;
                let mark = self.bookmarks.get(&name).ok_or(Error::UnknownBookmark)?;
                self.exec(TreeNavCmd::GotoBookmark(*mark))?;
            }
            Word::SetBookmark => {
                let name = self.data_stack.pop_char()?;
                let mark = self.doc.bookmark();
                self.bookmarks.insert(name, mark);
            }
            Word::InsertChar => {
                let ch = self.data_stack.pop_char()?;
                self.exec(TextCmd::InsertChar(ch))?;
            }
            Word::DeleteCharBackward => self.exec(TextCmd::DeleteCharBackward)?,
            Word::DeleteCharForward => self.exec(TextCmd::DeleteCharForward)?,
            Word::TreeMode => self.exec(TextNavCmd::TreeMode)?,
            Word::TextLeft => self.exec(TextNavCmd::Left)?,
            Word::TextRight => self.exec(TextNavCmd::Right)?,
        })
    }

    fn exec<T>(&mut self, cmd: T) -> Result<(), Error>
    where
        T: Debug + Into<MetaCommand<'static>>,
    {
        let result = self.doc.execute(cmd.into(), &mut self.cut_stack);
        self.doc
            .execute(MetaCommand::EndGroup, &mut self.cut_stack)
            .unwrap();
        result?;
        self.update_key_hints()?;
        Ok(())
    }

    fn node_by_name(
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
        let mut text_node = self.node_by_name("message", &self.message_lang_name)?;
        text_node.inner().unwrap_text().text_mut(|t| {
            t.activate();
            t.set(text.into());
            t.inactivate();
        });
        let mut root_node = self.node_by_name("root", &self.message_lang_name)?;
        root_node
            .inner()
            .unwrap_fixed()
            .replace_child(0, text_node)
            .unwrap();
        Ok(root_node)
    }
}

fn new_doc(doc_name: &str, lang_name: &LanguageName, forest: &AstForest<'static>) -> Doc<'static> {
    let lang = LANG_SET.get(lang_name).unwrap();
    Doc::new(
        doc_name,
        forest.new_fixed_tree(
            lang,
            lang.lookup_construct("root"),
            NOTE_SETS.get(lang_name).unwrap(),
        ),
    )
}
