use crate::ast::Ast;
use crate::command::{Command, NavCmd, DocCmd};


pub struct UndoStack<'l> {
    groups: Vec<UndoGroup<'l>>
}

pub struct UndoGroup<'l> {
    contains_edit: bool,
    commands: Vec<Command<'l>>
}

pub struct Doc<'l> {
    name: String,
    undo: UndoStack<'l>,
    ast: Ast<'l>
}

impl<'l> Doc<'l> {
    fn navigate(cmd: NavCmd) -> Option<UndoGroup<'l>> {
        let undos = match cmd {
            NavCmd::Left => {
                let i = self.ast.index();
                if i == 0 {
                    return None;
                }
                self.ast.goto_parent();
                self.ast.goto_child(i - 1);
                vec!(NavCmd::Right)
            }
            NavCmd::Right => {
                let i = self.ast.index();
                let n = self.ast.num_siblings();
                if i + 1 >= n {
                    return None;
                }
                self.ast.goto_parent();
                self.ast.goto_child(i + 1);
                vec!(NavCmd::Left)
            }
            NavCmd::Parent => {
                if self.ast.at_root() {
                    return None;
                }
                let i = self.ast.index();
                self.ast.goto_parent();
                vec!(NavCmd::Child(i))
            }
            NavCmd::Child(i) => {
                if self.arity().is_none() // at leaf
                    || self.arity().is_text() // at text wrapper node
                    || i >= self.ast.num_children() // out of bounds
                {
                    return None;
                }
                self.ast.goto_child(i);
                vec!(NavCmd::Parent)
            }
        };
        Some(UndoGroup {
            contains_edit: false,
            commands: undos
        })
    }
}
