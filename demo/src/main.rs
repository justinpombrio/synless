use std::collections::HashMap;
use std::fmt::Debug;

use termion::event::Key;

use editor::{Doc, EditorCmd, MetaCommand, NotationSet, TextCmd, TextNavCmd, TreeCmd, TreeNavCmd};
use frontends::{Event, Frontend, Terminal};
use language::Sort;
use pretty::ColorTheme;

mod core_editor;
mod demo_keymaps;
mod error;
mod keymap;
mod keymap_lang;
mod message_lang;
mod prog;

use core_editor::{Core, DocName};
use error::Error;
use keymap::{Kmap, TreeKmapFactory};
use prog::{CallStack, DataStack, KmapSpec, Prog, Value, Word};

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
    core: Core,
    frontend: Terminal,
    data_stack: DataStack<'static>,
    call_stack: CallStack<'static>,
    tree_keymaps: HashMap<String, TreeKmapFactory<'static>>,
    tree_keymap_stack: Vec<KmapSpec>,
    text_keymap: Kmap<'static>,
}

impl Ed {
    fn new() -> Result<Self, Error> {
        let core = Core::new()?;
        let mut tree_keymaps = HashMap::new();
        tree_keymaps.insert("tree".to_string(), demo_keymaps::make_tree_map());
        tree_keymaps.insert(
            "speed_bool".to_string(),
            demo_keymaps::make_speed_bool_map(),
        );
        tree_keymaps.insert(
            "node".to_string(),
            demo_keymaps::make_node_map(
                core.lang(&core.lang_name(&DocName::Active).expect("no active doc"))
                    .unwrap(),
            ),
        );

        let mut ed = Ed {
            core,
            frontend: Terminal::new(ColorTheme::default_dark())?,
            data_stack: DataStack::new(),
            call_stack: CallStack::new(),
            tree_keymaps,
            text_keymap: demo_keymaps::make_text_map(),
            tree_keymap_stack: Vec::new(),
        };

        // Set initial keymap
        ed.call(Word::Literal(Value::MapName("tree".into())))?;
        ed.call(Word::AnySort)?;
        ed.call(Word::PushMap)?;

        // Add an empty list to the document
        ed.call(Word::Literal(Value::Usize(0)))?;
        ed.call(Word::Child)?;
        ed.call(Word::Literal(Value::LangConstruct(
            ed.core
                .lang_name(&DocName::Active)
                .expect("no active doc")
                .to_owned(),
            "list".into(),
        )))?;
        ed.call(Word::NodeByName)?;
        ed.call(Word::Replace)?;

        ed.core.clear_messages();
        Ok(ed)
    }

    fn run(&mut self) -> Result<(), Error> {
        self.update_key_hints()?;
        self.core.redisplay(&mut self.frontend)?;
        loop {
            if let Some(word) = self.call_stack.next() {
                if let Err(err) = self.call(word) {
                    self.core.msg(&format!("Error: {:?}", err))?;
                }
            } else {
                self.update_key_hints()?;
                self.core.redisplay(&mut self.frontend)?;
                self.exec(MetaCommand::EndGroup)?;
                match self.handle_event() {
                    Ok(prog) => self.call_stack.push(prog),
                    Err(Error::KeyboardInterrupt) => Err(Error::KeyboardInterrupt)?,
                    Err(err) => self.core.msg(&format!("Error: {:?}", err))?,
                }
            }
        }
    }

    fn active_keymap(&self) -> Result<Kmap<'static>, Error> {
        if self.core.docs.active().in_tree_mode() {
            let kmap = self.tree_keymap_stack.last().ok_or(Error::NoKeymap)?;

            // TODO pass context struct to filter instead of entire ast!
            Ok(self
                .tree_keymaps
                .get(&kmap.name)
                .ok_or_else(|| Error::UnknownKeymap(kmap.name.to_owned()))?
                .filter(self.core.docs.active().ast_ref(), &kmap.required_sort))
        } else {
            // TODO avoid cloning every time!
            Ok(self.text_keymap.clone())
        }
    }

    fn update_key_hints(&mut self) -> Result<(), Error> {
        let lang_name = self
            .core
            .lang_name(&DocName::KeyHints)
            .expect("no keyhints lang"); // TODO return error

        let mut dict_node = self.core.node_by_name("dict", lang_name)?;

        for (key, prog) in self.active_keymap()?.hints() {
            let mut key_node = self.core.node_by_name("key", lang_name)?;
            key_node.inner().unwrap_text().text_mut(|t| {
                t.activate();
                t.set(key);
                t.inactivate();
            });

            let mut prog_node = self.core.node_by_name("prog", lang_name)?;
            prog_node.inner().unwrap_text().text_mut(|t| {
                t.activate();
                t.set(prog);
                t.inactivate();
            });

            let mut entry_node = self.core.node_by_name("entry", &lang_name)?;
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
        let mut root_node = self.core.node_by_name("root", &lang_name)?;
        root_node
            .inner()
            .unwrap_fixed()
            .replace_child(0, dict_node)
            .unwrap();

        let kmap_name = if self.core.in_tree_mode() {
            let mut s = String::new();
            for (i, spec) in self.tree_keymap_stack.iter().enumerate() {
                if i != 0 {
                    s += "→";
                }
                s += &spec.name;
            }
            s
        } else {
            "text".to_string()
        };
        // TODO modify the ast instead of replacing the whole doc?
        self.core.set_doc(
            DocName::KeyHints,
            Doc::new(&kmap_name, root_node),
            lang_name.to_owned(),
        );
        Ok(())
    }

    fn handle_event(&mut self) -> Result<Prog<'static>, Error> {
        match self.frontend.next_event() {
            Some(Ok(Event::KeyEvent(Key::Ctrl('c')))) => Err(Error::KeyboardInterrupt),
            Some(Ok(Event::KeyEvent(key))) => self.active_keymap()?.lookup(key),
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
                self.core.msg(&message)?;
            }
            Word::NodeByName => {
                let (lang_name, construct_name) = self.data_stack.pop_lang_construct()?;
                let node = self.core.node_by_name(&construct_name, &lang_name)?;
                self.data_stack.push(Value::Tree(node));
            }
            Word::PushMap => {
                let sort = self.data_stack.pop_sort()?;
                let name = self.data_stack.pop_map_name()?;
                self.tree_keymap_stack.push(KmapSpec {
                    name,
                    required_sort: sort,
                });
            }
            Word::PopMap => {
                self.tree_keymap_stack.pop();
            }
            Word::ChildSort => {
                self.data_stack.push(Value::Sort(self.core.child_sort()));
            }
            Word::SelfSort => {
                self.data_stack.push(Value::Sort(self.core.self_sort()));
            }
            Word::SiblingSort => {
                self.data_stack.push(Value::Sort(self.core.sibling_sort()));
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
                let mark = self.core.get_bookmark(name)?;
                self.exec(TreeNavCmd::GotoBookmark(mark))?;
            }
            Word::SetBookmark => {
                let name = self.data_stack.pop_char()?;
                self.core.add_bookmark(name);
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
        self.core.exec(cmd.into())?;
        Ok(())
    }
}
