use editor::{
    make_json_lang, Doc, EditorCmd, MetaCommand, NotationSets, TextCmd, TextNavCmd, TreeCmd,
    TreeNavCmd,
};
use frontends::{Event, Frontend, Key, Terminal};
use language::LanguageSet;
use pretty::{ColorTheme, DocLabel};

use crate::engine::Engine;
use crate::error::ServerError;
use crate::keymaps::{AvailableKeys, FilterContext, KeymapManager};
use crate::prog::{CallStack, DataStack, Value, Word};

use crate::data::example_keymaps;
use crate::data::example_pane_notation::make_example_pane_notation;
use crate::data::keyhint_lang::make_keyhint_lang;
use crate::data::message_lang::make_message_lang;

/// Demonstrate a basic interactive tree editor
pub struct Server<'l> {
    engine: Engine<'l>,
    frontend: Terminal,
    data_stack: DataStack<'l>,
    call_stack: CallStack<'l>,
    keymap_manager: KeymapManager<'l>,
}

impl<'l> Server<'l> {
    pub fn new(
        language_set: &'l LanguageSet,
        notation_sets: &'l NotationSets,
    ) -> Result<Self, ServerError> {
        let engine = Engine::new(
            language_set,
            notation_sets,
            make_example_pane_notation(),
            make_keyhint_lang(),
            make_message_lang(),
            make_json_lang(),
        )?;
        let mut keymap_manager = KeymapManager::new();
        keymap_manager.register_mode("tree".into(), example_keymaps::make_tree_map());
        keymap_manager.register_mode("speed_bool".into(), example_keymaps::make_speed_bool_map());
        keymap_manager.register_menu(
            "node".into(),
            example_keymaps::make_node_map(
                engine.language(engine.lang_name_of(&DocLabel::ActiveDoc)?)?,
            ),
        );
        keymap_manager.replace_text_keymap(example_keymaps::make_text_map());

        let mut ed = Server {
            engine,
            frontend: Terminal::new(ColorTheme::default_dark())?,
            data_stack: DataStack::new(),
            call_stack: CallStack::new(),
            keymap_manager,
        };

        // Set initial keymap
        ed.call(Word::Literal(Value::ModeName("tree".into())))?;
        ed.call(Word::PushMode)?;

        // ed.engine.clear_messages()?;
        Ok(ed)
    }

    pub fn run(&mut self) -> Result<(), ServerError> {
        loop {
            if self.keymap_manager.has_active_menu() {
                self.handle_input()?;
            } else {
                if let Some(word) = self.call_stack.next() {
                    if let Err(err) = self.call(word) {
                        self.engine.show_message(&format!("Error: {}", err))?;
                    }
                } else {
                    self.engine.exec(MetaCommand::EndGroup)?;
                    self.handle_input()?;
                }
            }
        }
    }

    fn handle_input(&mut self) -> Result<(), ServerError> {
        let available_keys = self
            .keymap_manager
            .get_available_keys(get_tree_context(self.engine.active_doc()?))?;

        self.update_key_hints(&available_keys)?;
        self.engine.redisplay(&mut self.frontend)?;

        match self.frontend.next_event() {
            Some(Ok(Event::KeyEvent(Key::Ctrl('c')))) => return Err(ServerError::KeyboardInterrupt),
            Some(Ok(Event::KeyEvent(key))) => Ok(key),
            Some(Err(err)) => Err(err.into()),
            _ => Err(ServerError::UnknownEvent),
        }
        .and_then(|key| {
            self.keymap_manager
                .lookup(key, &available_keys)
                .ok_or_else(|| ServerError::UnknownKey(key))
        })
        .map(|prog| {
            self.call_stack.push(prog);
            self.keymap_manager.deactivate_menu();
        })
        .or_else(|err| Ok(self.engine.show_message(&format!("Error: {}", err))?))
    }

    fn update_key_hints(&mut self, available_keys: &AvailableKeys) -> Result<(), ServerError> {
        let lang_name = self.engine.lang_name_of(&DocLabel::KeyHints)?;

        let mut keymap_node = self.engine.new_node("keymap", lang_name)?;

        for (key, prog) in self.keymap_manager.hints(available_keys) {
            let mut key_node = self.engine.new_node("key", lang_name)?;
            key_node.inner().unwrap_text().text_mut(|t| {
                t.activate();
                t.set(key);
                t.deactivate();
            });

            let mut prog_node = self.engine.new_node("prog", lang_name)?;
            prog_node.inner().unwrap_text().text_mut(|t| {
                t.activate();
                t.set(prog);
                t.deactivate();
            });

            let mut binding_node = self.engine.new_node("binding", &lang_name)?;
            binding_node
                .inner()
                .unwrap_fixed()
                .replace_child(0, key_node)
                .unwrap();
            binding_node
                .inner()
                .unwrap_fixed()
                .replace_child(1, prog_node)
                .unwrap();
            let mut inner_keymap = keymap_node.inner().unwrap_flexible();
            inner_keymap
                .insert_child(inner_keymap.num_children(), binding_node)
                .unwrap();
        }
        self.engine
            .exec_on(TreeCmd::Replace(keymap_node), &DocLabel::KeyHints)?;

        let mut description_node = self
            .engine
            .new_node_in_doc_lang("message", &DocLabel::KeymapName)?;
        description_node.inner().unwrap_text().text_mut(|t| {
            t.activate();
            t.set(available_keys.name());
            t.deactivate();
        });
        self.engine
            .exec_on(TreeCmd::Replace(description_node), &DocLabel::KeymapName)?;
        Ok(())
    }

    fn call(&mut self, word: Word<'l>) -> Result<(), ServerError> {
        Ok(match word {
            Word::Literal(value) => self.data_stack.push(value),
            Word::Apply => {
                let prog = self.data_stack.pop_quote()?;
                self.call_stack.push(prog);
            }
            Word::Swap => {
                self.data_stack.swap()?;
            }
            Word::Pop => {
                self.data_stack.pop()?;
            }
            Word::Print => {
                let message = self.data_stack.pop_string()?;
                self.engine.show_message(&message)?;
            }
            Word::NodeByName => {
                let (lang_name, construct_name) = self.data_stack.pop_lang_construct()?;
                let node = self.engine.new_node(&construct_name, &lang_name)?;
                self.data_stack.push(Value::Tree(node));
            }
            Word::PushMode => {
                let name = self.data_stack.pop_mode_name()?;
                self.keymap_manager.push_mode(name)?;
            }
            Word::PopMode => {
                self.keymap_manager.pop_mode();
            }
            Word::ActivateMenu => {
                let name = self.data_stack.pop_menu_name()?;
                if self.keymap_manager.has_active_menu() {
                    // TODO decide how to handle this
                    panic!("Another menu is already active");
                }
                self.keymap_manager.activate_menu(name)?;
            }
            Word::Remove => self.engine.exec(TreeCmd::Remove)?,
            Word::Clear => self.engine.exec(TreeCmd::Clear)?,
            Word::InsertHoleAfter => {
                self.engine.exec(TreeCmd::InsertHoleAfter)?;
            }
            Word::InsertHoleBefore => {
                self.engine.exec(TreeCmd::InsertHoleBefore)?;
            }
            Word::InsertHolePrepend => {
                self.engine.exec(TreeCmd::InsertHolePrepend)?;
            }
            Word::InsertHolePostpend => {
                self.engine.exec(TreeCmd::InsertHolePostpend)?;
            }
            Word::Replace => {
                let tree = self.data_stack.pop_tree()?;
                self.engine.exec(TreeCmd::Replace(tree))?;
            }
            Word::Left => self.engine.exec(TreeNavCmd::Left)?,
            Word::Right => self.engine.exec(TreeNavCmd::Right)?,
            Word::Parent => self.engine.exec(TreeNavCmd::Parent)?,
            Word::Child => {
                let index = self.data_stack.pop_usize()?;
                self.engine.exec(TreeNavCmd::Child(index))?;
            }
            Word::Undo => self.engine.exec(MetaCommand::Undo)?,
            Word::Redo => self.engine.exec(MetaCommand::Redo)?,
            Word::Cut => self.engine.exec(EditorCmd::Cut)?,
            Word::Copy => self.engine.exec(EditorCmd::Copy)?,
            Word::PasteSwap => self.engine.exec(EditorCmd::PasteSwap)?,
            Word::PopClipboard => self.engine.exec(EditorCmd::PopClipboard)?,
            Word::GotoBookmark => {
                let name = self.data_stack.pop_char()?;
                let mark = self.engine.get_bookmark(name)?;
                self.engine.exec(TreeNavCmd::GotoBookmark(mark))?;
            }
            Word::SetBookmark => {
                let name = self.data_stack.pop_char()?;
                self.engine.add_bookmark(name, &DocLabel::ActiveDoc)?;
            }
            Word::InsertChar => {
                let ch = self.data_stack.pop_char()?;
                self.engine.exec(TextCmd::InsertChar(ch))?;
            }
            Word::DeleteCharBackward => self.engine.exec(TextCmd::DeleteCharBackward)?,
            Word::DeleteCharForward => self.engine.exec(TextCmd::DeleteCharForward)?,
            Word::TreeMode => self.engine.exec(TextNavCmd::TreeMode)?,
            Word::TextLeft => self.engine.exec(TextNavCmd::Left)?,
            Word::TextRight => self.engine.exec(TextNavCmd::Right)?,
        })
    }
}

fn get_tree_context<'l>(doc: &Doc<'l>) -> Option<FilterContext> {
    if doc.in_tree_mode() {
        Some(FilterContext {
            sort: doc.self_sort(),
            self_arity: doc.self_arity_type(),
            parent_arity: doc.parent_arity_type(),
        })
    } else {
        None
    }
}
