use lazy_static::lazy_static;
use std::fmt::Debug;
use std::{io, thread, time};

use termion::event::Key;

use frontends::{terminal, Event, Frontend, Terminal};
use pretty::{Color, ColorTheme, Pos, PrettyDocument, PrettyScreen, Style};

use editor::{make_json_lang, Ast, AstForest, CommandGroup, Doc, NotationSet, TreeCmd, TreeNavCmd};
use language::{LanguageName, LanguageSet};

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

        let mut ed = Ed {
            doc,
            lang_name,
            forest,
            term: Terminal::new(ColorTheme::default_dark())?,
            messages: Vec::new(),
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

    fn handle_event(&mut self) -> Result<bool, Error> {
        match self.term.next_event() {
            Some(Ok(Event::KeyEvent(Key::Right))) => self.exec(TreeNavCmd::Right),
            Some(Ok(Event::KeyEvent(Key::Left))) => self.exec(TreeNavCmd::Left),
            Some(Ok(Event::KeyEvent(Key::Up))) => self.exec(TreeNavCmd::Parent),
            Some(Ok(Event::KeyEvent(Key::Down))) => self.exec(TreeNavCmd::Child(0)),
            Some(Ok(Event::KeyEvent(Key::Char('u')))) => self.exec(CommandGroup::Undo),
            Some(Ok(Event::KeyEvent(Key::Char('r')))) => self.exec(CommandGroup::Redo),
            Some(Ok(Event::KeyEvent(Key::Char('i')))) => {
                self.msg("select node type to insert after...");
                let node = self.handle_node_selection()?;
                self.exec(TreeCmd::InsertAfter(node))
            }
            Some(Ok(Event::KeyEvent(Key::Char('o')))) => {
                self.msg("select node type to postpend...");
                let node = self.handle_node_selection()?;
                self.exec(TreeCmd::InsertPostpend(node))
            }
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

    fn exec<T>(&mut self, cmd: T)
    where
        T: Debug + Into<CommandGroup<'static>>,
    {
        let name = format!("{:?}", cmd);
        if self.doc.execute(cmd.into()) {
            self.msg(&format!("OK: {}", name))
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
