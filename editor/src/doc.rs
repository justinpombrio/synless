use std::{iter, mem, vec};

use crate::ast::{Ast, AstKind, AstRef};
use crate::command::{Command, CommandGroup, EditorCmd, TextCmd, TextNavCmd, TreeCmd, TreeNavCmd};

#[derive(Debug)]
pub struct UndoGroup<'l> {
    contains_edit: bool,
    commands: Vec<Command<'l>>,
}

impl<'l> UndoGroup<'l> {
    fn new() -> UndoGroup<'l> {
        UndoGroup {
            contains_edit: false,
            commands: vec![],
        }
    }

    fn append(&mut self, mut other: UndoGroup<'l>) {
        self.contains_edit |= other.contains_edit;
        self.commands.append(&mut other.commands);
    }

    fn clear(&mut self) {
        self.contains_edit = false;
        self.commands = vec![];
    }

    fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

impl<'l> IntoIterator for UndoGroup<'l> {
    type Item = Command<'l>;
    type IntoIter = iter::Rev<vec::IntoIter<Command<'l>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.commands.into_iter().rev()
    }
}

enum Mode {
    Tree,
    /// byte-index of text cursor
    Text(usize),
}

impl Mode {
    fn is_tree_mode(&self) -> bool {
        match self {
            Mode::Tree => true,
            _ => false,
        }
    }

    fn unwrap_text_pos(&self) -> usize {
        match self {
            Mode::Text(pos) => *pos,
            _ => panic!("tried to execute text command in tree mode"),
        }
    }
}

pub struct Doc<'l> {
    name: String,
    recent: UndoGroup<'l>,
    undo_stack: Vec<UndoGroup<'l>>,
    redo_stack: Vec<UndoGroup<'l>>,
    ast: Ast<'l>,
    mode: Mode,
}

impl<'l> Doc<'l> {
    pub fn new(name: &str, ast: Ast<'l>) -> Self {
        Doc {
            name: name.to_owned(),
            recent: UndoGroup::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            ast: ast,
            mode: Mode::Tree,
        }
    }

    pub fn ast_ref<'f>(&'f self) -> AstRef<'f, 'l> {
        self.ast.borrow()
    }

    fn take_recent(&mut self) -> UndoGroup<'l> {
        mem::replace(&mut self.recent, UndoGroup::new())
    }

    fn undo(&mut self) -> bool {
        assert!(self.recent.commands.is_empty());
        assert!(!self.recent.contains_edit);
        match self.undo_stack.pop() {
            None => false, // Undo stack is empty
            Some(group) => {
                if !self.execute_group(group) {
                    panic!("Failed to undo")
                }
                let recent = self.take_recent();
                self.redo_stack.push(recent);
                true
            }
        }
    }

    fn redo(&mut self) -> bool {
        assert!(self.recent.commands.is_empty());
        assert!(!self.recent.contains_edit);
        match self.redo_stack.pop() {
            None => false, // Redo stack is empty
            Some(group) => {
                if !self.execute_group(group) {
                    panic!("Failed to redo")
                }
                let recent = self.take_recent();
                self.undo_stack.push(recent);
                true
            }
        }
    }

    fn reset(&mut self) -> bool {
        let recent = self.take_recent();
        let ok = self.execute_group(recent);
        self.recent.clear();
        ok
    }

    pub fn execute(&mut self, cmds: CommandGroup<'l>) -> bool {
        match cmds {
            CommandGroup::Undo => self.undo(),
            CommandGroup::Redo => self.redo(),
            CommandGroup::Group(group) => {
                let all_ok = self.execute_group(group);
                let undos = self.take_recent();
                if !undos.is_empty() {
                    self.undo_stack.push(undos);
                }
                self.redo_stack.clear();
                all_ok
            }
        }
    }

    fn execute_group<I>(&mut self, cmds: I) -> bool
    where
        I: IntoIterator<Item = Command<'l>>,
    {
        let mut all_ok = true;
        for cmd in cmds {
            all_ok &= match cmd {
                Command::Ed(cmd) => self.execute_ed(cmd),
                Command::Tree(cmd) => self.execute_tree(cmd),
                Command::TreeNav(cmd) => self.execute_tree_nav(cmd),
                Command::Text(cmd) => self.execute_text(cmd),
                Command::TextNav(cmd) => self.execute_text_nav(cmd),
            }
        }
        all_ok
    }

    fn execute_ed(&mut self, _cmd: EditorCmd) -> bool {
        unimplemented!();
    }

    fn execute_tree(&mut self, cmd: TreeCmd<'l>) -> bool {
        if !self.mode.is_tree_mode() {
            // return false;
            panic!("tried to execute tree command in text mode")
        }
        let undos = match cmd {
            TreeCmd::Replace(new_ast) => {
                let i = self.ast.goto_parent();
                match self.ast.inner() {
                    AstKind::Fixed(mut fixed) => {
                        let old_ast = fixed.replace_child(i, new_ast);
                        fixed.goto_child(i);
                        vec![TreeCmd::Replace(old_ast).into()]
                    }
                    AstKind::Flexible(mut flexible) => {
                        let old_ast = flexible.replace_child(i, new_ast);
                        flexible.goto_child(i);
                        vec![TreeCmd::Replace(old_ast).into()]
                    }
                    _ => panic!("how can a parent not be fixed or flexible?"),
                }
            }
            TreeCmd::Remove => {
                let i = self.ast.goto_parent();
                match self.ast.inner() {
                    AstKind::Fixed(mut fixed) => {
                        // Oops, go back, we can't delete something from a fixed node.
                        fixed.goto_child(i);
                        return false;
                    }
                    AstKind::Flexible(mut flexible) => {
                        let old_ast = flexible.remove_child(i);
                        let num_children = flexible.num_children();
                        if num_children == 0 {
                            // Stay at the childless parent
                            vec![TreeCmd::InsertPrepend(old_ast).into()]
                        } else if i == 0 {
                            // We removed the first child, so to undo must insert before.
                            flexible.goto_child(0);
                            vec![TreeCmd::InsertBefore(old_ast).into()]
                        } else {
                            // Go to the left.
                            flexible.goto_child(i - 1);
                            vec![TreeCmd::InsertAfter(old_ast).into()]
                        }
                    }
                    _ => panic!("how can a parent not be fixed or flexible?"),
                }
            }
            TreeCmd::InsertBefore(new_ast) => match self.insert_sibling(true, new_ast) {
                Ok(undo) => undo,
                Err(()) => return false,
            },
            TreeCmd::InsertAfter(new_ast) => match self.insert_sibling(false, new_ast) {
                Ok(undo) => undo,
                Err(()) => return false,
            },
            TreeCmd::InsertPrepend(new_ast) => match self.insert_child_at_edge(true, new_ast) {
                Ok(undo) => undo,
                Err(()) => return false,
            },
            TreeCmd::InsertPostpend(new_ast) => match self.insert_child_at_edge(false, new_ast) {
                Ok(undo) => undo,
                Err(()) => return false,
            },
        };
        self.recent.append(UndoGroup {
            contains_edit: true,
            commands: undos,
        });
        true
    }

    fn execute_tree_nav(&mut self, cmd: TreeNavCmd) -> bool {
        if !self.mode.is_tree_mode() {
            // TODO: once there's scripting or user-defined keybindings,
            // this needs to be a gentler error.
            panic!("tried to execute tree navigation command in text mode")
        }
        let undos = match cmd {
            TreeNavCmd::Left => {
                let i = self.ast.index();
                if i == 0 {
                    return false;
                }
                self.ast.goto_sibling(i - 1);
                vec![TreeNavCmd::Right.into()]
            }
            TreeNavCmd::Right => {
                let i = self.ast.index();
                let n = self.ast.num_siblings();
                if i + 1 >= n {
                    return false;
                }
                self.ast.goto_sibling(i + 1);
                vec![TreeNavCmd::Left.into()]
            }
            TreeNavCmd::Parent => {
                if self.ast.is_parent_at_root() {
                    // User should never be able to select the root node
                    return false;
                }
                let i = self.ast.goto_parent();
                vec![TreeNavCmd::Child(i).into()]
            }
            TreeNavCmd::Child(i) => match self.ast.inner() {
                AstKind::Text(mut text) => {
                    // Enter text mode
                    self.mode = Mode::Text(i);
                    text.text_mut().as_mut().activate();
                    vec![TextNavCmd::TreeMode.into()]
                }
                AstKind::Fixed(mut ast) => {
                    if i >= ast.num_children() {
                        return false;
                    }
                    ast.goto_child(i);
                    vec![TreeNavCmd::Parent.into()]
                }
                AstKind::Flexible(mut ast) => {
                    if i >= ast.num_children() {
                        return false;
                    }
                    ast.goto_child(i);
                    vec![TreeNavCmd::Parent.into()]
                }
            },
        };
        self.recent.append(UndoGroup {
            contains_edit: false,
            commands: undos,
        });
        true
    }

    fn execute_text(&mut self, cmd: TextCmd) -> bool {
        let mut ast = self.ast.inner().unwrap_text();
        let char_index = self.mode.unwrap_text_pos();
        let undos = match cmd {
            TextCmd::InsertChar(character) => {
                ast.text_mut().as_mut().insert(char_index, character);
                self.mode = Mode::Text(char_index + 1);
                vec![TextCmd::DeleteCharBackward.into()]
            }
            TextCmd::DeleteCharForward => {
                if let Some(c) = ast.text_mut().as_mut().delete_forward(char_index) {
                    vec![TextCmd::InsertChar(c).into()]
                } else {
                    return false;
                }
            }
            TextCmd::DeleteCharBackward => {
                if let Some(c) = ast.text_mut().as_mut().delete_backward(char_index) {
                    self.mode = Mode::Text(char_index - 1);
                    vec![TextCmd::InsertChar(c).into()]
                } else {
                    return false;
                }
            }
        };
        self.recent.append(UndoGroup {
            contains_edit: true,
            commands: undos,
        });
        true
    }

    fn execute_text_nav(&mut self, cmd: TextNavCmd) -> bool {
        let char_index = self.mode.unwrap_text_pos();
        let mut ast = self.ast.inner().unwrap_text();
        let undos = match cmd {
            TextNavCmd::Left => {
                if char_index == 0 {
                    return false;
                }
                self.mode = Mode::Text(char_index - 1);
                vec![TextNavCmd::Right.into()]
            }
            TextNavCmd::Right => {
                if char_index >= ast.text().as_text_ref().num_chars() {
                    return false;
                }
                self.mode = Mode::Text(char_index + 1);
                vec![TextNavCmd::Left.into()]
            }
            TextNavCmd::TreeMode => {
                // Exit text mode
                ast.text_mut().as_mut().inactivate();
                self.mode = Mode::Tree;
                vec![TreeNavCmd::Child(char_index).into()]
            }
        };
        self.recent.append(UndoGroup {
            contains_edit: false,
            commands: undos,
        });
        true
    }

    /// If `at_start` is true, insert the new ast as the first child of this
    /// node. Otherwise, insert it as the last child. If the insertion is
    /// successful, return the list of commands needed to undo it. Otherwise,
    /// return `Err`.
    fn insert_child_at_edge(
        &mut self,
        at_start: bool,
        new_ast: Ast<'l>,
    ) -> Result<Vec<Command<'l>>, ()> {
        match self.ast.inner() {
            AstKind::Flexible(mut flexible) => {
                let original_num_children = flexible.num_children();
                let index = if at_start { 0 } else { original_num_children };
                flexible.insert_child(index, new_ast);
                flexible.goto_child(index);
                let mut undo = vec![TreeCmd::Remove.into()];
                if original_num_children != 0 {
                    // If there are still children left after removing this
                    // one, we won't automatically go back up to the parent.
                    // So do that here.
                    undo.push(TreeNavCmd::Parent.into());
                }
                Ok(undo)
            }
            _ => Err(()),
        }
    }

    /// If `before` is true, insert the new ast immediately to the left of this
    /// this node. Otherwise, insert it immediately to the right. If the
    /// insertion is successful, return the list of commands needed to undo it.
    /// Otherwise, return `Err`.
    fn insert_sibling(&mut self, before: bool, new_ast: Ast<'l>) -> Result<Vec<Command<'l>>, ()> {
        let i = self.ast.goto_parent();
        let insertion_index = if before { i } else { i + 1 };
        match self.ast.inner() {
            AstKind::Fixed(mut fixed) => {
                // Oops, go back, we can't insert something into a fixed node.
                fixed.goto_child(i);
                Err(())
            }
            AstKind::Flexible(mut flexible) => {
                flexible.insert_child(insertion_index, new_ast);
                flexible.goto_child(insertion_index);
                if before && i != 0 {
                    Ok(vec![TreeNavCmd::Right.into(), TreeCmd::Remove.into()])
                } else {
                    Ok(vec![TreeCmd::Remove.into()])
                }
            }
            _ => panic!("how can a parent not be fixed or flexible?"),
        }
    }
}
