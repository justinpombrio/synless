use std::fmt::Debug;

use termion::event::Key;

use editor::{
    make_json_lang, EditorCmd, MetaCommand, NotationSet, TextCmd, TextNavCmd, TreeCmd, TreeNavCmd,
};
use frontends::{Event, Frontend, Terminal};
use language::Sort;
use pretty::{Color, ColorTheme, CursorVis, DocLabel, DocPosSpec, PaneNotation, PaneSize, Style};

mod core_editor;
mod data;
mod error;
mod keymap;
mod prog;

use core_editor::Core;
use error::ShellError;
use keymap::{FilterContext, Keymaps, Kmap};
use prog::{CallStack, DataStack, Prog, Value, Word};

use data::example_keymaps;
use data::keyhint_lang::make_keyhint_lang;
use data::message_lang::make_message_lang;

fn main() -> Result<(), ShellError> {
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
    keymaps: Keymaps<'static>,
}

impl Ed {
    fn new() -> Result<Self, ShellError> {
        let core = Core::new(
            demo_pane_notation(),
            make_keyhint_lang(),
            make_message_lang(),
            make_json_lang(),
        )?;
        let mut keymaps = Keymaps::new();
        keymaps.insert_mode("tree".into(), example_keymaps::make_tree_map());
        keymaps.insert_mode("speed_bool".into(), example_keymaps::make_speed_bool_map());
        keymaps.insert_menu(
            "node".into(),
            example_keymaps::make_node_map(
                core.language(core.lang_name_of(&DocLabel::ActiveDoc)?)?,
            ),
        );
        keymaps.set_text_keymap(example_keymaps::make_text_map());

        let mut ed = Ed {
            core,
            frontend: Terminal::new(ColorTheme::default_dark())?,
            data_stack: DataStack::new(),
            call_stack: CallStack::new(),
            keymaps,
        };

        // Set initial keymap
        ed.call(Word::Literal(Value::ModeName("tree".into())))?;
        ed.call(Word::PushMode)?;

        // Add an empty list to the document
        ed.call(Word::Literal(Value::LangConstruct(
            ed.core
                .lang_name_of(&DocLabel::ActiveDoc)
                .expect("no active doc")
                .to_owned(),
            "list".into(),
        )))?;
        ed.call(Word::NodeByName)?;
        ed.call(Word::Replace)?;

        ed.core.clear_messages()?;
        Ok(ed)
    }

    fn run(&mut self) -> Result<(), ShellError> {
        loop {
            if self.keymaps.active_menu.is_some() {
                self.handle_input()?;
            } else {
                if let Some(word) = self.call_stack.next() {
                    if let Err(err) = self.call(word) {
                        self.core.show_message(&format!("Error: {:?}", err))?;
                    }
                } else {
                    self.exec(MetaCommand::EndGroup)?;
                    self.handle_input()?;
                }
            }
        }
    }

    fn handle_input(&mut self) -> Result<(), ShellError> {
        let doc = self.core.active_doc()?;
        let context = FilterContext {
            required_sort: doc.self_sort(),
            self_arity: doc.self_arity_type(),
            parent_arity: doc.parent_arity_type(),
        };
        let kmap = self.keymaps.active_keymap(doc.in_tree_mode(), &context)?;

        self.update_key_hints(&kmap)?;
        self.core.redisplay(&mut self.frontend)?;
        match self.next_event(&kmap) {
            Ok(prog) => {
                self.call_stack.push(prog);
                self.keymaps.active_menu = None;
                Ok(())
            }
            Err(ShellError::KeyboardInterrupt) => Err(ShellError::KeyboardInterrupt),
            Err(err) => Ok(self.core.show_message(&format!("Error: {:?}", err))?),
        }
    }

    fn update_key_hints(&mut self, kmap: &Kmap) -> Result<(), ShellError> {
        let lang_name = self.core.lang_name_of(&DocLabel::KeyHints)?;

        let mut dict_node = self.core.new_node("dict", lang_name)?;

        for (key, prog) in self.keymaps.hints(kmap) {
            let mut key_node = self.core.new_node("key", lang_name)?;
            key_node.inner().unwrap_text().text_mut(|t| {
                t.activate();
                t.set(key);
                t.inactivate();
            });

            let mut prog_node = self.core.new_node("prog", lang_name)?;
            prog_node.inner().unwrap_text().text_mut(|t| {
                t.activate();
                t.set(prog);
                t.inactivate();
            });

            let mut entry_node = self.core.new_node("entry", &lang_name)?;
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
        self.core
            .exec_on(TreeCmd::Replace(dict_node), &DocLabel::KeyHints)?;

        let mut description_node = self
            .core
            .new_node_in_doc_lang("message", &DocLabel::KeymapName)?;
        description_node.inner().unwrap_text().text_mut(|t| {
            t.activate();
            t.set(kmap.name());
            t.inactivate();
        });
        self.core
            .exec_on(TreeCmd::Replace(description_node), &DocLabel::KeymapName)?;
        Ok(())
    }

    fn next_event(&mut self, kmap: &Kmap) -> Result<Prog<'static>, ShellError> {
        match self.frontend.next_event() {
            Some(Ok(Event::KeyEvent(Key::Ctrl('c')))) => Err(ShellError::KeyboardInterrupt),
            Some(Ok(Event::KeyEvent(key))) => self.keymaps.lookup(key, kmap),
            Some(Err(err)) => Err(err.into()),
            _ => Err(ShellError::UnknownEvent),
        }
    }

    fn call(&mut self, word: Word<'static>) -> Result<(), ShellError> {
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
                self.core.show_message(&message)?;
            }
            Word::NodeByName => {
                let (lang_name, construct_name) = self.data_stack.pop_lang_construct()?;
                let node = self.core.new_node(&construct_name, &lang_name)?;
                self.data_stack.push(Value::Tree(node));
            }
            Word::PushMode => {
                let name = self.data_stack.pop_mode_name()?;
                self.keymaps.mode_stack.push(name);
            }
            Word::PopMode => {
                self.keymaps.mode_stack.pop();
            }
            Word::ActivateMenu => {
                let name = self.data_stack.pop_menu_name()?;
                if self.keymaps.active_menu.is_some() {
                    // TODO decide how to handle this
                    panic!("Another menu is already active");
                }
                self.keymaps.active_menu = Some(name);
            }
            Word::ChildSort => {
                self.data_stack
                    .push(Value::Sort(self.core.active_doc()?.child_sort()));
            }
            Word::SelfSort => {
                self.data_stack
                    .push(Value::Sort(self.core.active_doc()?.self_sort()));
            }
            Word::SiblingSort => {
                self.data_stack
                    .push(Value::Sort(self.core.active_doc()?.sibling_sort()));
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
                self.core.add_bookmark(name, &DocLabel::ActiveDoc)?;
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

    fn exec<T>(&mut self, cmd: T) -> Result<(), ShellError>
    where
        T: Debug + Into<MetaCommand<'static>>,
    {
        self.core.exec(cmd.into())?;
        Ok(())
    }
}

fn demo_pane_notation() -> PaneNotation {
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
