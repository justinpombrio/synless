use lazy_static::lazy_static;
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;

use termion::event::Key;

use editor::{
    make_json_lang, Ast, AstForest, Clipboard, CommandGroup, Doc, EditorCmd, NotationSet, TextCmd,
    TreeCmd, TreeNavCmd,
};
use frontends::{Event, Frontend, Terminal};
use language::{LanguageName, LanguageSet};
use pretty::{Color, ColorTheme, CursorVis, DocLabel, Pane, PaneNotation, PaneSize, Style};
use utility::GrowOnlyMap;

mod error;
mod keymap;
mod keymap_lang;
mod message_lang;
mod prog;

use error::Error;
use keymap::Keymap;
use keymap_lang::make_keymap_lang;
use message_lang::make_message_lang;
use prog::{Prog, Stack, Word};

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
    message_doc: Ast<'static>,
    message_lang_name: LanguageName,
    stack: Stack<'static>,
    keymaps: HashMap<String, Keymap<'static>>,
    keymap_stack: Vec<String>,
    kmap_lang_name: LanguageName,
    key_hints: Ast<'static>,
    cut_stack: Clipboard<'static>,
}

impl Ed {
    fn new() -> Result<Self, Error> {
        let (json_lang, json_notes) = make_json_lang();
        let (kmap_lang, kmap_notes) = make_keymap_lang();
        let (msg_lang, msg_notes) = make_message_lang();
        let json_lang_name = json_lang.name().to_string();
        let kmap_lang_name = kmap_lang.name().to_string();
        let msg_lang_name = msg_lang.name().to_string();

        LANG_SET.insert(json_lang_name.clone(), json_lang);
        LANG_SET.insert(kmap_lang_name.clone(), kmap_lang);
        LANG_SET.insert(msg_lang_name.clone(), msg_lang);
        NOTE_SETS.insert(json_lang_name.clone(), json_notes);
        NOTE_SETS.insert(kmap_lang_name.clone(), kmap_notes);
        NOTE_SETS.insert(msg_lang_name.clone(), msg_notes);

        let forest = AstForest::new(&LANG_SET);

        let json_lang = LANG_SET.get(&json_lang_name).unwrap();
        let doc = Doc::new(
            "DemoJsonDoc",
            forest.new_fixed_tree(
                json_lang,
                json_lang.lookup_construct("root"),
                NOTE_SETS.get(&json_lang_name).unwrap(),
            ),
        );

        let kmap_lang = LANG_SET.get(&kmap_lang_name).unwrap();
        let key_hints = forest.new_fixed_tree(
            kmap_lang,
            kmap_lang.lookup_construct("root"),
            NOTE_SETS.get(&kmap_lang_name).unwrap(),
        );

        let msg_lang = LANG_SET.get(&msg_lang_name).unwrap();
        let msg_doc = forest.new_fixed_tree(
            msg_lang,
            msg_lang.lookup_construct("root"),
            NOTE_SETS.get(&msg_lang_name).unwrap(),
        );

        let mut maps = HashMap::new();
        maps.insert("tree".to_string(), Keymap::tree());
        maps.insert("speed_bool".to_string(), Keymap::speed_bool());
        maps.insert("node".to_string(), Keymap::node(json_lang));

        let mut ed = Ed {
            doc,
            lang_name: json_lang_name,
            forest,
            term: Terminal::new(ColorTheme::default_dark())?,
            messages: VecDeque::new(),
            message_doc: msg_doc,
            message_lang_name: msg_lang_name,
            stack: Stack::new(),
            keymaps: maps,
            kmap_lang_name,
            key_hints,
            keymap_stack: Vec::new(),
            cut_stack: Clipboard::new(),
        };

        // Set initial keymap
        ed.push(Word::MapName("tree".into()))?;
        ed.push(Word::PushMap)?;

        // Add an empty list to the document
        ed.push(Word::Usize(0))?;
        ed.push(Word::Child)?;
        ed.push(Word::LangConstruct(ed.lang_name.clone(), "list".into()))?;
        ed.push(Word::NodeByName)?;
        ed.push(Word::Replace)?;

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
                .insert_child(i, msg_node);
        }
        let mut root_node = self.node_by_name("root", &self.message_lang_name)?;
        root_node.inner().unwrap_fixed().replace_child(0, list_node);
        self.message_doc = root_node;
        Ok(())
    }

    fn pane_notation(&self) -> PaneNotation {
        let doc = PaneNotation::Doc {
            label: DocLabel::ActiveDoc,
            style: None,
        };

        let key_hints = PaneNotation::Doc {
            label: DocLabel::KeyHints,
            style: None,
        };

        let messages = PaneNotation::Doc {
            label: DocLabel::Messages,
            style: None,
        };

        let divider = PaneNotation::Fill {
            ch: '=',
            style: Some(Style::color(Color::Base03)),
        };

        PaneNotation::Vert {
            panes: vec![
                (PaneSize::Proportional(5), doc),
                (PaneSize::Fixed(1), divider.clone()),
                (PaneSize::Proportional(1), key_hints),
                (PaneSize::Fixed(1), divider),
                (PaneSize::Fixed(self.messages.len()), messages),
            ],
            style: None,
        }
    }

    fn redisplay(&mut self) -> Result<(), Error> {
        let notation = self.pane_notation();
        let doc = self.doc.ast_ref();
        let key_hints = self.key_hints.ast_ref();
        let messages = self.message_doc.ast_ref();
        self.term.draw_frame(|mut pane: Pane<Terminal>| {
            pane.render(&notation, None, |label: &DocLabel| match label {
                DocLabel::ActiveDoc => Some((doc.clone(), CursorVis::Show)),
                DocLabel::KeyHints => Some((key_hints.clone(), CursorVis::Hide)),
                DocLabel::Messages => Some((messages.clone(), CursorVis::Hide)),
                _ => None,
            })?;
            Ok(())
        })?;
        Ok(())
    }

    fn active_keymap(&self) -> Result<&Keymap<'static>, Error> {
        let name = self.keymap_stack.last().ok_or(Error::NoKeymap)?;
        self.keymaps
            .get(name)
            .ok_or_else(|| Error::UnknownKeymap(name.to_owned()))
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
            entry_node.inner().unwrap_fixed().replace_child(0, key_node);
            entry_node
                .inner()
                .unwrap_fixed()
                .replace_child(1, prog_node);
            let mut inner_dict = dict_node.inner().unwrap_flexible();
            inner_dict.insert_child(inner_dict.num_children(), entry_node);
        }
        let mut root_node = self.node_by_name("root", &self.kmap_lang_name)?;
        root_node.inner().unwrap_fixed().replace_child(0, dict_node);
        self.key_hints = root_node;
        Ok(())
    }

    fn lookup_key(&self, key: Key) -> Result<Prog<'static>, Error> {
        self.active_keymap()?
            .0
            .get(&key)
            .cloned()
            .ok_or(Error::UnknownKey(key))
    }

    fn handle_key(&mut self, key: Key) -> Result<(), Error> {
        let prog = self.lookup_key(key)?;
        for word in prog.words {
            self.push(word)?;
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

    fn push(&mut self, word: Word<'static>) -> Result<(), Error> {
        Ok(match word {
            Word::Tree(..) => self.stack.push(word),
            Word::Usize(..) => self.stack.push(word),
            Word::MapName(..) => self.stack.push(word),
            Word::LangConstruct(..) => self.stack.push(word),
            Word::Message(..) => self.stack.push(word),
            Word::Char(..) => self.stack.push(word),
            Word::Quote(..) => self.stack.push(word),
            Word::Apply => {
                let word = self.stack.pop_quote()?;
                self.push(word)?;
            }
            Word::Swap => {
                self.stack.swap()?;
            }
            Word::Pop => {
                self.stack.pop()?;
            }
            Word::Echo => {
                let message = self.stack.pop_message()?;
                self.msg(&message)?;
            }
            Word::NodeByName => {
                let (lang_name, construct_name) = self.stack.pop_lang_construct()?;
                let node = self.node_by_name(&construct_name, &lang_name)?;
                self.push(Word::Tree(node))?;
            }
            Word::PushMap => {
                let name = self.stack.pop_map_name()?;
                self.keymap_stack.push(name);
                self.update_key_hints()?;
            }
            Word::PopMap => {
                self.keymap_stack.pop();
                self.update_key_hints()?;
            }
            Word::Remove => self.exec(TreeCmd::Remove)?,
            Word::InsertChar => {
                let ch = self.stack.pop_char()?;
                self.exec(TextCmd::InsertChar(ch))?;
            }
            Word::InsertAfter => {
                let tree = self.stack.pop_tree()?;
                self.exec(TreeCmd::InsertAfter(tree))?;
            }
            Word::InsertBefore => {
                let tree = self.stack.pop_tree()?;
                self.exec(TreeCmd::InsertBefore(tree))?;
            }
            Word::InsertPrepend => {
                let tree = self.stack.pop_tree()?;
                self.exec(TreeCmd::InsertPrepend(tree))?;
            }
            Word::InsertPostpend => {
                let tree = self.stack.pop_tree()?;
                self.exec(TreeCmd::InsertPostpend(tree))?;
            }
            Word::Replace => {
                let tree = self.stack.pop_tree()?;
                self.exec(TreeCmd::Replace(tree))?;
            }
            Word::Left => self.exec(TreeNavCmd::Left)?,
            Word::Right => self.exec(TreeNavCmd::Right)?,
            Word::Parent => self.exec(TreeNavCmd::Parent)?,
            Word::Child => {
                let index = self.stack.pop_usize()?;
                self.exec(TreeNavCmd::Child(index))?;
            }
            Word::Undo => self.exec(CommandGroup::Undo)?,
            Word::Redo => self.exec(CommandGroup::Redo)?,
            Word::Cut => self.exec(EditorCmd::Cut)?,
            Word::Copy => self.exec(EditorCmd::Copy)?,
            Word::PasteAfter => self.exec(EditorCmd::PasteAfter)?,
            Word::PasteBefore => self.exec(EditorCmd::PasteBefore)?,
            Word::PastePrepend => self.exec(EditorCmd::PastePrepend)?,
            Word::PastePostpend => self.exec(EditorCmd::PastePostpend)?,
            Word::PasteReplace => self.exec(EditorCmd::PasteReplace)?,
        })
    }

    fn exec<T>(&mut self, cmd: T) -> Result<(), Error>
    where
        T: Debug + Into<CommandGroup<'static>>,
    {
        let name = format!("{:?}", cmd);
        self.doc
            .execute(cmd.into(), &mut self.cut_stack)
            .map_err(|_| Error::DocExec(name))
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
}
