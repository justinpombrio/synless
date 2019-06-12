use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fmt::Debug;
use std::{thread, time};

use termion::event::Key;

use editor::{
    make_json_lang, Ast, AstForest, CommandGroup, Doc, EditorCmd, NotationSet, TextCmd, TreeCmd,
    TreeNavCmd,
};
use frontends::{terminal, Event, Frontend, Terminal};
use language::{LanguageName, LanguageSet};
use pretty::{Color, ColorTheme, Pos, PrettyDocument, PrettyScreen, Shade, Style};
use utility::GrowOnlyMap;

mod error;
mod keymap;
mod keymap_lang;
mod prog;

use error::Error;
use keymap::Keymap;
use keymap_lang::make_keymap_lang;
use prog::{Prog, Stack, Word};

lazy_static! {
    pub static ref LANG_SET: LanguageSet = LanguageSet::new();
    pub static ref NOTE_SETS: GrowOnlyMap<LanguageName, NotationSet> = GrowOnlyMap::new();
}

fn main() -> Result<(), Error> {
    let mut ed = Ed::new()?;
    ed.run()?;
    drop(ed);
    println!("Exited alternate screen. Your cursor should be visible again.");
    Ok(())
}

/// Demonstrate a basic interactive tree editor
struct Ed {
    doc: Doc<'static>,
    lang_name: LanguageName,
    forest: AstForest<'static>,
    term: Terminal,
    messages: Vec<String>,
    stack: Stack<'static>,
    keymaps: HashMap<String, Keymap<'static>>,
    keymap_stack: Vec<String>,
    // keymap_lang_name: LanguageName,
    // kmap_doc: Doc<'static>,
    keymap_summary: String,
    cut_stack: Vec<Ast<'static>>,
}

impl Ed {
    fn new() -> Result<Self, Error> {
        let (json_lang, json_notes) = make_json_lang();
        let (kmap_lang, kmap_notes) = make_keymap_lang();
        let json_lang_name = json_lang.name().to_string();
        let kmap_lang_name = kmap_lang.name().to_string();

        LANG_SET.insert(json_lang_name.clone(), json_lang);
        LANG_SET.insert(kmap_lang_name.clone(), kmap_lang);
        NOTE_SETS.insert(json_lang_name.clone(), json_notes);
        NOTE_SETS.insert(kmap_lang_name.clone(), kmap_notes);

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

        // let kmap_lang = LANG_SET.get(&kmap_lang_name).unwrap();
        // let kmap_doc = Doc::new(
        //     "KeymapSummaryDoc",
        //     forest.new_fixed_tree(
        //         kmap_lang,
        //         kmap_lang.lookup_construct("root"),
        //         NOTE_SETS.get(&kmap_lang_name).unwrap(),
        //     ),
        // );

        let mut maps = HashMap::new();
        maps.insert("tree".to_string(), Keymap::tree());
        maps.insert("speed_bool".to_string(), Keymap::speed_bool());
        maps.insert("node".to_string(), Keymap::node(json_lang));

        let mut ed = Ed {
            doc,
            lang_name: json_lang_name,
            forest,
            term: Terminal::new(ColorTheme::default_dark())?,
            messages: Vec::new(),
            stack: Stack::new(),
            keymaps: maps,
            // keymap_lang_name: kmap_lang_name,
            // kmap_doc,
            keymap_stack: Vec::new(),
            keymap_summary: String::new(),
            cut_stack: Vec::new(),
        };
        // Set initial keymap
        ed.push(Word::MapName("tree".into()));
        ed.push(Word::PushMap);

        // Add some json stuff to the document, as an example
        ed.exec(TreeNavCmd::Child(0));
        ed.exec(TreeCmd::Replace(ed.node_by_name("list", &ed.lang_name)));
        ed.exec(TreeCmd::InsertPrepend(
            ed.node_by_name("true", &ed.lang_name),
        ));

        // ed.exec_kmap(TreeNavCmd::Child(0));
        // ed.exec_kmap(TreeCmd::Replace(
        //     ed.node_by_name("dict", &ed.keymap_lang_name),
        // ));
        // ed.exec_kmap(TreeCmd::InsertPrepend(
        //     ed.node_by_name("entry", &ed.keymap_lang_name),
        // ));
        ed.messages.clear();

        Ok(ed)
    }

    fn run(&mut self) -> Result<(), Error> {
        loop {
            self.redisplay()?;
            if !self.handle_event()? {
                break;
            }
        }
        Ok(())
    }

    fn msg(&mut self, msg: &str) {
        self.messages.push(msg.to_owned());
        self.redisplay().unwrap();
    }

    fn print_keymap_summary(&mut self) -> Result<(), Error> {
        let size = self.term.size()?;
        let offset = Pos {
            row: size.row / 2,
            col: 0,
        };
        let map_path = self.keymap_stack.join("→");
        self.term
            .print(offset, &map_path, Style::reverse_color(Color::Base0B))?;
        self.term.print(
            offset + Pos { row: 1, col: 0 },
            &self.keymap_summary,
            Style::color(Color::Base0B),
        )?;
        Ok(())
    }

    fn print_messages(&mut self, num_recent: usize) -> Result<(), Error> {
        let size = self.term.size()?;
        self.term
            .print(
                Pos {
                    row: size.row - (num_recent + 1) as u32,
                    col: 0,
                },
                "messages:",
                Style::reverse_color(Color::Base0C),
            )
            .is_ok();

        for (i, msg) in self.messages.iter().rev().take(num_recent).enumerate() {
            let pos = Pos {
                row: size.row - (num_recent - i) as u32,
                col: 0,
            };
            let result = self.term.print(pos, msg, Style::color(Color::Base0C));

            // For this demo, just ignore out of bounds errors. The real editor
            // shouldn't ever try to print out of bounds.
            match result {
                Err(terminal::Error::OutOfBounds) | Ok(()) => (),
                other_err => other_err?,
            };
        }
        Ok(())
    }

    fn redisplay(&mut self) -> Result<(), Error> {
        let size = self.term.update_size()?;

        self.doc
            .ast_ref()
            .pretty_print(size.col, &mut self.term)
            .unwrap();

        let cursor_region = self.doc.ast_ref().locate_cursor::<Terminal>(size.col)?;
        self.term.shade(cursor_region, Shade(0))?;

        // self.kmap_doc
        //     .ast_ref()
        //     .pretty_print(size.col, &mut self.term)
        //     .unwrap();

        self.print_messages(5)?;
        self.print_keymap_summary()?;
        self.term.show()?;
        Ok(())
    }

    fn active_keymap(&self) -> &Keymap<'static> {
        let name = self.keymap_stack.last().expect("no active keymap");
        self.keymaps.get(name).expect("unknown keymap name")
    }

    fn update_keymap_summary(&mut self) {
        self.keymap_summary = self.active_keymap().summary();
    }

    fn lookup_key(&self, key: Key) -> Option<Prog<'static>> {
        self.active_keymap().0.get(&key).cloned()
    }

    fn handle_event(&mut self) -> Result<bool, Error> {
        match self.term.next_event() {
            Some(Ok(Event::KeyEvent(Key::Char('q')))) => {
                self.msg("Quitting, goodbye!");
                thread::sleep(time::Duration::from_secs(1));
                return Ok(false);
            }
            Some(Ok(Event::KeyEvent(Key::Ctrl('c')))) => {
                return Ok(false);
            }
            Some(Err(err)) => {
                self.msg(&format!("got error: {:?}", err));
            }
            Some(Ok(Event::KeyEvent(key))) => {
                let prog = self.lookup_key(key);
                if let Some(prog) = prog {
                    for word in prog.words {
                        self.push(word);
                    }
                } else {
                    self.msg(&format!("unknown key: {:?}", key));
                }
            }

            _ => {
                self.msg(&format!("unexpected event, or no event"));
            }
        }
        Ok(true)
    }

    fn push(&mut self, word: Word<'static>) {
        match word {
            Word::Tree(..) => self.stack.push(word),
            Word::Usize(..) => self.stack.push(word),
            Word::MapName(..) => self.stack.push(word),
            Word::LangConstruct(..) => self.stack.push(word),
            Word::Message(..) => self.stack.push(word),
            Word::Char(..) => self.stack.push(word),
            Word::Quote(..) => self.stack.push(word),
            Word::Apply => {
                let word = self.stack.pop_quote();
                self.push(word);
            }
            Word::Swap => {
                self.stack.swap();
            }
            Word::Pop => {
                self.stack.pop();
            }
            Word::Echo => {
                let message = self.stack.pop_message();
                self.msg(&message);
            }
            Word::NodeByName => {
                let (lang_name, construct_name) = self.stack.pop_lang_construct();
                let node = self.node_by_name(&construct_name, &lang_name);
                self.push(Word::Tree(node));
            }
            Word::PushMap => {
                let name = self.stack.pop_map_name();
                self.keymap_stack.push(name);
                self.update_keymap_summary();
            }
            Word::PopMap => {
                self.keymap_stack.pop();
                self.update_keymap_summary();
            }
            Word::Remove => self.exec(TreeCmd::Remove),
            Word::InsertChar => {
                let ch = self.stack.pop_char();
                self.exec(TextCmd::InsertChar(ch));
            }
            Word::InsertAfter => {
                let tree = self.stack.pop_tree();
                self.exec(TreeCmd::InsertAfter(tree));
            }
            Word::InsertBefore => {
                let tree = self.stack.pop_tree();
                self.exec(TreeCmd::InsertBefore(tree));
            }
            Word::InsertPrepend => {
                let tree = self.stack.pop_tree();
                self.exec(TreeCmd::InsertPrepend(tree));
            }
            Word::InsertPostpend => {
                let tree = self.stack.pop_tree();
                self.exec(TreeCmd::InsertPostpend(tree));
            }
            Word::Replace => {
                let tree = self.stack.pop_tree();
                self.exec(TreeCmd::Replace(tree));
            }
            Word::Left => self.exec(TreeNavCmd::Left),
            Word::Right => self.exec(TreeNavCmd::Right),
            Word::Parent => self.exec(TreeNavCmd::Parent),
            Word::Child => {
                let index = self.stack.pop_usize();
                self.exec(TreeNavCmd::Child(index));
            }
            Word::Undo => self.exec(CommandGroup::Undo),
            Word::Redo => self.exec(CommandGroup::Redo),
            Word::Cut => self.cut(),
            Word::Copy => self.copy(),
            Word::PasteAfter => {
                if let Some(tree) = self.cut_stack.pop() {
                    // TODO if the insert fails, we'll lose the tree forever...
                    self.exec(TreeCmd::InsertAfter(tree))
                }
            }
            Word::PasteBefore => {
                if let Some(tree) = self.cut_stack.pop() {
                    self.exec(TreeCmd::InsertBefore(tree))
                }
            }
            Word::PastePrepend => {
                if let Some(tree) = self.cut_stack.pop() {
                    self.exec(TreeCmd::InsertPrepend(tree))
                }
            }
            Word::PastePostpend => {
                if let Some(tree) = self.cut_stack.pop() {
                    self.exec(TreeCmd::InsertPostpend(tree))
                }
            }
            Word::PasteReplace => {
                if let Some(tree) = self.cut_stack.pop() {
                    self.exec(TreeCmd::Replace(tree))
                }
            }
        }
    }

    fn cut(&mut self) {
        match self.doc.execute(EditorCmd::Cut.into()) {
            Ok(asts) => {
                self.cut_stack.extend(asts);
            }
            Err(..) => self.msg("FAIL: couldn't cut!"),
        }
    }

    fn copy(&mut self) {
        match self.doc.execute(EditorCmd::Copy.into()) {
            Ok(asts) => {
                self.cut_stack.extend(asts);
            }
            Err(..) => self.msg("FAIL: couldn't copy!"),
        }
    }

    fn exec<T>(&mut self, cmd: T)
    where
        T: Debug + Into<CommandGroup<'static>>,
    {
        let name = format!("{:?}", cmd);
        if !self.doc.execute(cmd.into()).is_ok() {
            self.msg(&format!("FAIL: {}", name))
        }
    }

    // fn exec_kmap<T>(&mut self, cmd: T)
    // where
    //     T: Debug + Into<CommandGroup<'static>>,
    // {
    //     let name = format!("{:?}", cmd);
    //     if !self.kmap_doc.execute(cmd.into()).is_ok() {
    //         self.msg(&format!("FAIL(kmap): {}", name))
    //     }
    // }

    fn node_by_name(&self, name: &str, lang_name: &LanguageName) -> Ast<'static> {
        let name = name.to_string();
        let lang = LANG_SET.get(lang_name).unwrap();
        let notes = NOTE_SETS.get(lang_name).unwrap();
        self.forest
            .new_tree(lang, &name, notes)
            .expect("unknown node name")
    }
}