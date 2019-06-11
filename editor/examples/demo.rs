use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::{io, thread, time};

use termion::event::Key;

use frontends::{terminal, Event, Frontend, Terminal};
use pretty::{Color, ColorTheme, Pos, PrettyDocument, PrettyScreen, Style};

use editor::{make_json_lang, Ast, AstForest, CommandGroup, Doc, NotationSet, TreeCmd, TreeNavCmd};
use language::{LanguageName, LanguageSet};

mod keymap;
mod prog;

use keymap::Keymap;
use prog::{Prog, Thing};

lazy_static! {
    pub static ref LANG_SET: LanguageSet = make_json_lang().0;
    pub static ref NOTE_SET: NotationSet = make_json_lang().1;
}

fn main() -> Result<(), Error> {
    let mut ed = Ed::new()?;
    ed.run()?;
    drop(ed);
    println!("Exited alternate screen. Your cursor should be visible again.");
    Ok(())
}

#[derive(Debug)]
enum Error {
    UnknownKey(char),
    UnknownEvent,
    Io(io::Error),
    Term(terminal::Error),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

impl From<terminal::Error> for Error {
    fn from(e: terminal::Error) -> Error {
        Error::Term(e)
    }
}

/// Demonstrate a basic interactive tree editor
struct Ed {
    doc: Doc<'static>,
    lang_name: LanguageName,
    forest: AstForest<'static>,
    term: Terminal,
    messages: Vec<String>,
    stack: Vec<Thing<'static>>,
    keymaps: HashMap<String, Keymap<'static>>,
    keymap_stack: Vec<String>,
}

impl Ed {
    fn new() -> Result<Self, Error> {
        let forest = AstForest::new(&LANG_SET);
        let lang_name = "json".into();
        let lang = LANG_SET.get(&lang_name).unwrap();

        let doc = Doc::new(
            "MyDemoDoc",
            forest.new_fixed_tree(lang, lang.lookup_construct("root"), &NOTE_SET),
        );

        let mut maps = HashMap::new();
        maps.insert("normal".to_string(), Keymap::normal());
        maps.insert("space".to_string(), Keymap::space());
        let mut ed = Ed {
            doc,
            lang_name,
            forest,
            term: Terminal::new(ColorTheme::default_dark())?,
            messages: Vec::new(),
            stack: Vec::new(),
            keymaps: maps,
            keymap_stack: vec!["normal".to_string()],
        };

        // Add some json stuff to the document, as an example
        ed.exec(TreeNavCmd::Child(0));
        let node = ed.node_by_name("list");
        ed.exec(TreeCmd::Replace(node));
        let node = ed.node_by_name("true");
        ed.exec(TreeCmd::InsertPrepend(node));
        ed.messages.clear();

        Ok(ed)
    }

    fn run(&mut self) -> Result<(), Error> {
        self.msg(
            "welcome! i: insert, o: insert_postpend, t: true, f: false, l: list, u: undo, r: redo, arrows for navigation",
        );
        self.msg(&self.active_keymap().summary());
        loop {
            if !self.handle_event()? {
                break;
            }
            self.redisplay()?;
        }
        Ok(())
    }

    fn msg(&mut self, msg: &str) {
        self.messages.push(msg.to_owned());
        self.redisplay().unwrap();
    }

    fn print_messages(&mut self, num_recent: usize) -> Result<(), Error> {
        let size = self.term.size()?;
        for (i, msg) in self.messages.iter().rev().take(num_recent).enumerate() {
            let pos = Pos {
                row: size.row - (num_recent - i) as u32,
                col: 0,
            };
            let result = self
                .term
                .print(pos, msg, Style::reverse_color(Color::Base09));

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
        self.print_messages(10)?;
        self.term.show()?;
        Ok(())
    }
    fn active_keymap(&self) -> &Keymap<'static> {
        let name = self.keymap_stack.last().expect("no active keymap");
        self.keymaps.get(name).expect("unknown keymap name")
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
                    for thing in prog.words {
                        self.push(thing);
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

    fn handle_node_selection(&mut self) -> Result<Ast<'static>, Error> {
        match self.term.next_event().expect("no event")? {
            Event::KeyEvent(Key::Char(c)) => self.node_by_key(c).ok_or(Error::UnknownKey(c)),
            Event::KeyEvent(Key::Ctrl('c')) => panic!("got ctrl-c"),
            _ => Err(Error::UnknownEvent),
        }
    }

    fn pop_tree(&mut self) -> Ast<'static> {
        if let Some(Thing::Tree(tree)) = self.stack.pop() {
            tree
        } else {
            panic!("expected tree on stack")
        }
    }

    fn pop_usize(&mut self) -> usize {
        if let Some(Thing::Usize(num)) = self.stack.pop() {
            num
        } else {
            panic!("expected usize on stack")
        }
    }

    fn pop_map_name(&mut self) -> String {
        if let Some(Thing::MapName(s)) = self.stack.pop() {
            s
        } else {
            panic!("expected map name on stack")
        }
    }

    fn pop_node_name(&mut self) -> String {
        if let Some(Thing::NodeName(s)) = self.stack.pop() {
            s
        } else {
            panic!("expected node name on stack")
        }
    }

    fn pop_message(&mut self) -> String {
        if let Some(Thing::Message(s)) = self.stack.pop() {
            s
        } else {
            panic!("expected message on stack")
        }
    }

    fn push(&mut self, thing: Thing<'static>) {
        match thing {
            Thing::Tree(..) => self.stack.push(thing),
            Thing::Usize(..) => self.stack.push(thing),
            Thing::MapName(..) => self.stack.push(thing),
            Thing::NodeName(..) => self.stack.push(thing),
            Thing::Message(..) => self.stack.push(thing),
            Thing::Echo => {
                let message = self.pop_message();
                self.msg(&message);
            }
            Thing::SelectNode => {
                let node = self.handle_node_selection().unwrap();
                self.push(Thing::Tree(node));
            }
            Thing::NodeByName => {
                let name = self.pop_node_name();
                let node = self.node_by_name(&name);
                self.push(Thing::Tree(node));
            }
            Thing::PushMap => {
                let name = self.pop_map_name();
                self.keymap_stack.push(name);
                self.msg(&self.active_keymap().summary());
            }
            Thing::PopMap => {
                self.keymap_stack.pop();
                self.msg(&self.active_keymap().summary());
            }
            Thing::Remove => self.exec(TreeCmd::Remove),
            Thing::InsertAfter => {
                let tree = self.pop_tree();
                self.exec(TreeCmd::InsertAfter(tree));
            }
            Thing::InsertBefore => {
                let tree = self.pop_tree();
                self.exec(TreeCmd::InsertBefore(tree));
            }
            Thing::InsertPrepend => {
                let tree = self.pop_tree();
                self.exec(TreeCmd::InsertPrepend(tree));
            }
            Thing::InsertPostpend => {
                let tree = self.pop_tree();
                self.exec(TreeCmd::InsertPostpend(tree));
            }
            Thing::Replace => {
                let tree = self.pop_tree();
                self.exec(TreeCmd::Replace(tree));
            }
            Thing::Left => self.exec(TreeNavCmd::Left),
            Thing::Right => self.exec(TreeNavCmd::Right),
            Thing::Parent => self.exec(TreeNavCmd::Parent),
            Thing::Child => {
                let index = self.pop_usize();
                self.exec(TreeNavCmd::Child(index));
            }
            Thing::Undo => self.exec(CommandGroup::Undo),
            Thing::Redo => self.exec(CommandGroup::Redo),
        }
    }

    fn exec<T>(&mut self, cmd: T)
    where
        T: Debug + Into<CommandGroup<'static>>,
    {
        let name = format!("{:?}", cmd);
        if self.doc.execute(cmd.into()) {
            // self.msg(&format!("OK: {}", name))
        } else {
            self.msg(&format!("FAIL: {}", name))
        }
    }

    fn node_by_name(&self, name: &str) -> Ast<'static> {
        let name = name.to_owned();
        let lang = LANG_SET.get(&self.lang_name).unwrap();
        self.forest
            .new_tree(lang, &name, &NOTE_SET)
            .expect("unknown node name")
    }

    fn node_by_key(&self, key: char) -> Option<Ast<'static>> {
        let lang = LANG_SET.get(&self.lang_name).unwrap();
        let name = lang.lookup_key(key)?;
        self.forest.new_tree(lang, &name, &NOTE_SET)
    }
}

impl<'l> fmt::Display for Thing<'l> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Thing::Tree(..) => write!(f, "Tree"),
            Thing::Usize(n) => write!(f, "{}", n),
            Thing::MapName(s) => write!(f, "MapName(\"{}\")", s),
            Thing::NodeName(s) => write!(f, "NodeName(\"{}\")", s),
            Thing::Message(s) => write!(f, "Message(\"{}\")", s),
            Thing::Echo => write!(f, "Echo"),
            Thing::SelectNode => write!(f, "SelectNode"),
            Thing::NodeByName => write!(f, "NodeByName"),
            Thing::PushMap => write!(f, "PushMap"),
            Thing::PopMap => write!(f, "PopMap"),
            Thing::Remove => write!(f, "Remove"),
            Thing::InsertAfter => write!(f, "InsertAfter"),
            Thing::InsertBefore => write!(f, "InsertBefore"),
            Thing::InsertPrepend => write!(f, "InsertPrepend"),
            Thing::InsertPostpend => write!(f, "InsertPostpend"),
            Thing::Replace => write!(f, "Replace"),
            Thing::Left => write!(f, "Left"),
            Thing::Right => write!(f, "Right"),
            Thing::Parent => write!(f, "Parent"),
            Thing::Child => write!(f, "Child"),
            Thing::Undo => write!(f, "Undo"),
            Thing::Redo => write!(f, "Redo"),
        }
    }
}
