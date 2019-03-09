use std::{iter, mem, vec};

use crate::ast::{Ast, AstKind};
use crate::command::{Command, DocCmd, NavCmd};

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
}

impl<'l> IntoIterator for UndoGroup<'l> {
    type Item = Command<'l>;
    type IntoIter = iter::Rev<vec::IntoIter<Command<'l>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.commands.into_iter().rev()
    }
}

pub struct Doc<'l> {
    name: String,
    recent: UndoGroup<'l>,
    undo_stack: Vec<UndoGroup<'l>>,
    redo_stack: Vec<UndoGroup<'l>>,
    ast: Ast<'l>,
}

impl<'l> Doc<'l> {
    fn take_recent(&mut self) -> UndoGroup<'l> {
        mem::replace(&mut self.recent, UndoGroup::new())
    }

    fn undo(&mut self) -> bool {
        // TODO: Think about error handling if self.recent is none-empty.
        self.recent.clear(); // It _should_ be empty, but just in case.
        match self.undo_stack.pop() {
            None => false, // Undo stack is empty
            Some(group) => {
                if !self.execute(group) {
                    return false;
                }
                let recent = self.take_recent();
                self.redo_stack.push(recent);
                true
            }
        }
    }

    fn redo(&mut self) -> bool {
        // TODO: Think about error handling if self.recent is none-empty.
        self.recent.clear(); // It _should_ be empty, but just in case.
        match self.redo_stack.pop() {
            None => false, // Redo stack is empty
            Some(group) => {
                if !self.execute(group) {
                    return false;
                }
                let recent = self.take_recent();
                self.undo_stack.push(recent);
                true
            }
        }
    }

    fn reset(&mut self) -> bool {
        let recent = self.take_recent();
        let ok = self.execute(recent);
        self.recent.clear();
        ok
    }

    fn execute<I>(&mut self, cmds: I) -> bool
    where
        I: IntoIterator<Item = Command<'l>>,
    {
        let mut all_ok = true;
        for cmd in cmds {
            all_ok &= match cmd {
                Command::Nav(cmd) => self.execute_nav(cmd),
                Command::Doc(cmd) => self.execute_doc(cmd),
                Command::Ed(_) => panic!("Doc::execute - unexpected editor command"),
            }
        }
        all_ok
    }

    fn execute_nav(&mut self, cmd: NavCmd) -> bool {
        let undos = match cmd {
            NavCmd::Left => {
                let i = self.ast.index();
                if i == 0 {
                    return false;
                }
                self.ast.goto_sibling(i - 1);
                vec![NavCmd::Right.into()]
            }
            NavCmd::Right => {
                let i = self.ast.index();
                let n = self.ast.num_siblings();
                if i + 1 >= n {
                    return false;
                }
                self.ast.goto_sibling(i + 1);
                vec![NavCmd::Left.into()]
            }
            NavCmd::Parent => {
                if self.ast.at_root() {
                    return false;
                }
                let i = self.ast.index();
                self.ast.goto_parent();
                vec![NavCmd::Child(i).into()]
            }
            NavCmd::Child(i) => match self.ast.inner() {
                AstKind::Text(_) => return false,
                AstKind::Fixed(mut ast) => {
                    if i >= ast.num_children() {
                        return false;
                    }
                    ast.goto_child(i);
                    vec![NavCmd::Parent.into()]
                }
                AstKind::Flexible(mut ast) => {
                    if i >= ast.num_children() {
                        return false;
                    }
                    ast.goto_child(i);
                    vec![NavCmd::Parent.into()]
                }
            },
        };
        self.recent.append(UndoGroup {
            contains_edit: false,
            commands: undos,
        });
        true
    }

    fn execute_doc(&mut self, cmd: DocCmd) -> bool {
        let undos = match cmd {
            DocCmd::Undo => {
                self.undo(); // TODO: warn if false
                vec![]
            }
            DocCmd::Redo => {
                self.redo(); // TODO: warn if false
                vec![]
            }
            _ => unimplemented!(),
        };
        self.recent.append(UndoGroup {
            contains_edit: true,
            commands: undos,
        });
        true
    }
}
