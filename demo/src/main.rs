use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fmt::Debug;

use termion::event::Key;

use editor::{
    make_json_lang, Ast, AstForest, Clipboard, CommandGroup, Doc, EditorCmd, NotationSet, TextCmd,
    TreeCmd, TreeNavCmd,
};
use frontends::{terminal, Event, Frontend, Terminal};
use language::{LanguageName, LanguageSet};
use pretty::{Color, ColorTheme, Pane, Pos, PrettyDocument, PrettyWindow, Shade, Style};
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
    messages: Vec<String>,
    stack: Stack<'static>,
    keymaps: HashMap<String, Keymap<'static>>,
    keymap_stack: Vec<String>,
    // keymap_lang_name: LanguageName,
    // kmap_doc: Doc<'static>,
    keymap_summary: String,
    cut_stack: Clipboard<'static>,
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
        self.messages.push(msg.to_owned());
        self.redisplay()
    }

    fn _print_keymap_summary(&mut self) -> Result<(), Error> {
        let size = self.term.size()?;

        let offset = Pos {
            row: size.row / 2,
            col: 0,
        };
        let map_path = self.keymap_stack.join("→");
        let mut pane = self.term.pane()?;
        pane.print(offset, &map_path, Style::reverse_color(Color::Base0B))?;

        match pane.print(
            offset + Pos { row: 1, col: 0 },
            &self.keymap_summary,
            Style::color(Color::Base0B),
        ) {
            // ignore, just let the long string get truncated
            Err(terminal::Error::OutOfBounds) => (),
            Ok(_) => (),
            err => err?,
        };
        Ok(())
    }

    fn _print_messages(&mut self, num_recent: usize) -> Result<(), Error> {
        let size = self.term.size()?;
        let mut pane = self.term.pane()?;
        pane.print(
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
            let result = pane.print(pos, msg, Style::color(Color::Base0C));

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
        let ast_ref = self.doc.ast_ref();
        self.term.draw_frame(|mut pane: Pane<Terminal>| {
            let size = pane.rect().size();

            // TODO pick which part of the doc to display, based on the cursor
            // position, instead of always showing the top!
            let doc_pos = Pos::zero();

            ast_ref
                .pretty_print(size.col, &mut pane, doc_pos)
                .expect("failed to pretty-print document");

            let cursor_region = ast_ref.locate_cursor(size.col);
            pane.shade(cursor_region, Shade(0))?;

            // self._print_messages(5).unwrap();
            // self._print_keymap_summary().unwrap();
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

    fn update_keymap_summary(&mut self) -> Result<(), Error> {
        Ok(self.keymap_summary = self.active_keymap()?.summary())
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
                self.update_keymap_summary()?;
            }
            Word::PopMap => {
                self.keymap_stack.pop();
                self.update_keymap_summary()?;
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
