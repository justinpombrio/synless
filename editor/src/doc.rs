use std::{iter, mem, vec};

use crate::ast::{Ast, AstKind, AstRef};
use crate::command::{Command, CommandGroup, EditorCmd, TextCmd, TextNavCmd, TreeCmd, TreeNavCmd};

#[derive(Debug)]
pub enum DocError<'l> {
    NotInTextMode,
    NotInTreeMode,
    NothingToUndo,
    NothingToRedo,
    WrongSort(Ast<'l>),
    CannotPaste,
    CannotMove,
    CannotRemoveNode,
    CannotDeleteChar,
}

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

    fn with_edit(commands: Vec<Command<'l>>) -> Self {
        UndoGroup {
            contains_edit: true,
            commands,
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

/// A stack containing Asts that have been cut or copied.
pub struct Clipboard<'l>(Vec<Ast<'l>>);

impl<'l> Clipboard<'l> {
    /// Construct a new, empty clipboard stack.
    pub fn new() -> Self {
        Clipboard(Vec::new())
    }

    /// Push the given tree onto the clipboard stack.
    pub fn push(&mut self, new_ast: Ast<'l>) {
        self.0.push(new_ast);
    }

    /// Pop a tree off the top of the clipboard stack, returning None if the
    /// clipboard is empty.
    pub fn pop(&mut self) -> Option<Ast<'l>> {
        self.0.pop()
    }
}

#[derive(Clone, Copy)]
pub enum Mode {
    Tree,
    /// byte-index of text cursor
    Text(usize),
}

impl Mode {
    pub fn is_tree_mode(&self) -> bool {
        match self {
            Mode::Tree => true,
            _ => false,
        }
    }

    fn text_pos(&self) -> Option<usize> {
        match self {
            Mode::Text(pos) => Some(*pos),
            _ => None,
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ast_ref<'f>(&'f self) -> AstRef<'f, 'l> {
        self.ast.ast_ref()
    }

    pub fn in_tree_mode(&self) -> bool {
        self.mode.is_tree_mode()
    }

    fn take_recent(&mut self) -> UndoGroup<'l> {
        mem::replace(&mut self.recent, UndoGroup::new())
    }

    fn undo(&mut self, clipboard: &mut Clipboard<'l>) -> Result<(), DocError<'l>> {
        assert!(self.recent.commands.is_empty());
        assert!(!self.recent.contains_edit);

        // Find the most recent group that contains an edit
        let index = self
            .undo_stack
            .iter()
            .rposition(|group| group.contains_edit);

        if let Some(index) = index {
            // Undo everything up to and including that group
            for _ in 0..(self.undo_stack.len() - index) {
                let group = self
                    .undo_stack
                    .pop()
                    .expect("undo stack shouldn't be empty!");
                self.execute_group(group, clipboard)
                    .expect("Failed to undo");
                let recent = self.take_recent();
                self.redo_stack.push(recent);
            }
            Ok(())
        } else {
            Err(DocError::NothingToUndo) // There are no edits on the undo stack
        }
    }

    fn redo(&mut self, clipboard: &mut Clipboard<'l>) -> Result<(), DocError<'l>> {
        assert!(self.recent.commands.is_empty());
        assert!(!self.recent.contains_edit);

        // Find the most recent group that contains an edit
        let index = self
            .redo_stack
            .iter()
            .rposition(|group| group.contains_edit);

        if let Some(index) = index {
            // Redo everything up to and including that group
            for _ in 0..(self.redo_stack.len() - index) {
                let group = self
                    .redo_stack
                    .pop()
                    .expect("redo stack shouldn't be empty!");
                self.execute_group(group, clipboard)
                    .expect("Failed to redo");
                let recent = self.take_recent();
                self.undo_stack.push(recent);
            }
            Ok(())
        } else {
            Err(DocError::NothingToRedo) // There are no edits on the redo stack
        }
    }

    pub fn execute(
        &mut self,
        cmds: CommandGroup<'l>,
        clipboard: &mut Clipboard<'l>,
    ) -> Result<(), DocError<'l>> {
        match cmds {
            CommandGroup::Undo => self.undo(clipboard),
            CommandGroup::Redo => self.redo(clipboard),
            CommandGroup::Group(group) => {
                let result = self.execute_group(group, clipboard);
                let undos = self.take_recent();
                if !undos.is_empty() {
                    self.undo_stack.push(undos);
                }
                self.redo_stack.clear();
                result
            }
        }
    }

    fn execute_group<I>(
        &mut self,
        cmds: I,
        clipboard: &mut Clipboard<'l>,
    ) -> Result<(), DocError<'l>>
    where
        I: IntoIterator<Item = Command<'l>>,
    {
        for cmd in cmds {
            let undos = match cmd {
                Command::Ed(cmd) => self.execute_ed(cmd, clipboard)?,
                Command::Tree(cmd) => self.execute_tree(cmd)?,
                Command::TreeNav(cmd) => self.execute_tree_nav(cmd)?,
                Command::Text(cmd) => self.execute_text(cmd)?,
                Command::TextNav(cmd) => self.execute_text_nav(cmd)?,
            };
            self.recent.append(undos);
        }
        Ok(())
    }

    fn execute_ed(
        &mut self,
        cmd: EditorCmd,
        clipboard: &mut Clipboard<'l>,
    ) -> Result<UndoGroup<'l>, DocError<'l>> {
        if !self.mode.is_tree_mode() {
            return Err(DocError::NotInTreeMode);
        }
        match cmd {
            EditorCmd::Cut => {
                let (undos, ast) = self.remove(true)?;
                clipboard.push(ast.expect("failed to cut"));
                Ok(UndoGroup::with_edit(undos))
            }
            EditorCmd::Copy => {
                clipboard.push(self.ast.clone());
                Ok(UndoGroup::new())
            }
            EditorCmd::PasteAfter => {
                if let Some(tree) = clipboard.pop() {
                    // TODO if the insert fails, we'll lose the tree forever...
                    self.execute_tree(TreeCmd::InsertAfter(tree))
                } else {
                    // TODO should we return an error if the clipboard is empty?
                    Ok(UndoGroup::new())
                }
            }
            EditorCmd::PasteBefore => {
                if let Some(tree) = clipboard.pop() {
                    self.execute_tree(TreeCmd::InsertBefore(tree))
                } else {
                    Ok(UndoGroup::new())
                }
            }
            EditorCmd::PastePrepend => {
                if let Some(tree) = clipboard.pop() {
                    self.execute_tree(TreeCmd::InsertPrepend(tree))
                } else {
                    Ok(UndoGroup::new())
                }
            }
            EditorCmd::PastePostpend => {
                if let Some(tree) = clipboard.pop() {
                    self.execute_tree(TreeCmd::InsertPostpend(tree))
                } else {
                    Ok(UndoGroup::new())
                }
            }
            EditorCmd::PasteReplace => {
                if let Some(tree) = clipboard.pop() {
                    self.execute_tree(TreeCmd::Replace(tree))
                } else {
                    Ok(UndoGroup::new())
                }
            }
        }
    }

    fn execute_tree(&mut self, cmd: TreeCmd<'l>) -> Result<UndoGroup<'l>, DocError<'l>> {
        if !self.mode.is_tree_mode() {
            return Err(DocError::NotInTreeMode);
        }
        let undos = match cmd {
            TreeCmd::Replace(new_ast) => {
                let i = self.ast.goto_parent(); // child index
                match self.ast.inner() {
                    AstKind::Fixed(mut fixed) => {
                        let result = fixed.replace_child(i, new_ast);
                        fixed.goto_child(i);
                        let old_ast =
                            result.map_err(|rejected_ast| DocError::WrongSort(rejected_ast))?;
                        vec![TreeCmd::Replace(old_ast).into()]
                    }
                    AstKind::Flexible(mut flexible) => {
                        let result = flexible.replace_child(i, new_ast);
                        flexible.goto_child(i);
                        let old_ast =
                            result.map_err(|rejected_ast| DocError::WrongSort(rejected_ast))?;
                        vec![TreeCmd::Replace(old_ast).into()]
                    }
                    _ => panic!("how can a parent not be fixed or flexible?"),
                }
            }
            TreeCmd::Remove => {
                let (undos, _ast) = self.remove(false)?;
                undos
            }
            TreeCmd::InsertBefore(new_ast) => self.insert_sibling(true, new_ast)?,
            TreeCmd::InsertAfter(new_ast) => self.insert_sibling(false, new_ast)?,
            TreeCmd::InsertPrepend(new_ast) => self.insert_child_at_edge(true, new_ast)?,
            TreeCmd::InsertPostpend(new_ast) => self.insert_child_at_edge(false, new_ast)?,
        };
        Ok(UndoGroup {
            contains_edit: true,
            commands: undos,
        })
    }

    fn execute_tree_nav(&mut self, cmd: TreeNavCmd) -> Result<UndoGroup<'l>, DocError<'l>> {
        if !self.mode.is_tree_mode() {
            return Err(DocError::NotInTreeMode);
        }
        let undos = match cmd {
            TreeNavCmd::Left => {
                let i = self.ast.index();
                if i == 0 {
                    return Err(DocError::CannotMove);
                }
                self.ast.goto_sibling(i - 1);
                vec![TreeNavCmd::Right.into()]
            }
            TreeNavCmd::Right => {
                let i = self.ast.index();
                let n = self.ast.num_siblings();
                if i + 1 >= n {
                    return Err(DocError::CannotMove);
                }
                self.ast.goto_sibling(i + 1);
                vec![TreeNavCmd::Left.into()]
            }
            TreeNavCmd::Parent => {
                if self.ast.is_parent_at_root() {
                    // User should never be able to select the root node
                    return Err(DocError::CannotMove);
                }
                let i = self.ast.goto_parent();
                vec![TreeNavCmd::Child(i).into()]
            }
            TreeNavCmd::Child(i) => match self.ast.inner() {
                AstKind::Text(mut text) => {
                    // Enter text mode
                    self.mode = Mode::Text(i);
                    text.text_mut(|t| t.activate());
                    vec![TextNavCmd::TreeMode.into()]
                }
                AstKind::Fixed(mut ast) => {
                    if i >= ast.num_children() {
                        return Err(DocError::CannotMove);
                    }
                    ast.goto_child(i);
                    vec![TreeNavCmd::Parent.into()]
                }
                AstKind::Flexible(mut ast) => {
                    if i >= ast.num_children() {
                        return Err(DocError::CannotMove);
                    }
                    ast.goto_child(i);
                    vec![TreeNavCmd::Parent.into()]
                }
            },
        };
        Ok(UndoGroup {
            contains_edit: false,
            commands: undos,
        })
    }

    fn execute_text(&mut self, cmd: TextCmd) -> Result<UndoGroup<'l>, DocError<'l>> {
        let char_index = self.mode.text_pos().ok_or(DocError::NotInTextMode)?;
        let mut ast = self.ast.inner().unwrap_text();
        let undos = match cmd {
            TextCmd::InsertChar(character) => {
                ast.text_mut(|t| t.insert(char_index, character));
                self.mode = Mode::Text(char_index + 1);
                vec![TextCmd::DeleteCharBackward.into()]
            }
            TextCmd::DeleteCharForward => {
                let text_len = ast.text(|t| t.num_chars());
                if char_index == text_len {
                    return Err(DocError::CannotDeleteChar);
                }
                let c = ast.text_mut(|t| t.delete(char_index));
                vec![TextCmd::InsertChar(c).into()]
            }
            TextCmd::DeleteCharBackward => {
                if char_index == 0 {
                    return Err(DocError::CannotDeleteChar);
                }
                let c = ast.text_mut(|t| t.delete(char_index - 1));
                self.mode = Mode::Text(char_index - 1);
                vec![TextCmd::InsertChar(c).into()]
            }
        };
        Ok(UndoGroup {
            contains_edit: true,
            commands: undos,
        })
    }

    fn execute_text_nav(&mut self, cmd: TextNavCmd) -> Result<UndoGroup<'l>, DocError<'l>> {
        let char_index = self.mode.text_pos().ok_or(DocError::NotInTextMode)?;
        let mut ast = self.ast.inner().unwrap_text();
        let undos = match cmd {
            TextNavCmd::Left => {
                if char_index == 0 {
                    return Err(DocError::CannotMove);
                }
                self.mode = Mode::Text(char_index - 1);
                vec![TextNavCmd::Right.into()]
            }
            TextNavCmd::Right => {
                if char_index >= ast.text(|t| t.num_chars()) {
                    return Err(DocError::CannotMove);
                }
                self.mode = Mode::Text(char_index + 1);
                vec![TextNavCmd::Left.into()]
            }
            TextNavCmd::TreeMode => {
                // Exit text mode
                ast.text_mut(|t| t.inactivate());
                self.mode = Mode::Tree;
                vec![TreeNavCmd::Child(char_index).into()]
            }
        };
        Ok(UndoGroup {
            contains_edit: false,
            commands: undos,
        })
    }

    /// If `at_start` is true, insert the new ast as the first child of this
    /// node. Otherwise, insert it as the last child. If the insertion is
    /// successful, return the list of commands needed to undo it. Otherwise,
    /// return `Err`.
    fn insert_child_at_edge(
        &mut self,
        at_start: bool,
        new_ast: Ast<'l>,
    ) -> Result<Vec<Command<'l>>, DocError<'l>> {
        match self.ast.inner() {
            AstKind::Flexible(mut flexible) => {
                let original_num_children = flexible.num_children();
                let index = if at_start { 0 } else { original_num_children };
                flexible
                    .insert_child(index, new_ast)
                    .map_err(|rejected_ast| DocError::WrongSort(rejected_ast))?;
                flexible.goto_child(index);
                let mut undo = Vec::new();
                if original_num_children != 0 {
                    // If there are still children left after removing this
                    // one, we won't automatically go back up to the parent.
                    // So do that here.
                    undo.push(TreeNavCmd::Parent.into());
                }
                undo.push(TreeCmd::Remove.into());
                Ok(undo)
            }
            _ => Err(DocError::WrongSort(new_ast)),
        }
    }

    /// If `before` is true, insert the new ast immediately to the left of this
    /// this node. Otherwise, insert it immediately to the right. If the
    /// insertion is successful, return the list of commands needed to undo it.
    /// Otherwise, return `Err`.
    fn insert_sibling(
        &mut self,
        before: bool,
        new_ast: Ast<'l>,
    ) -> Result<Vec<Command<'l>>, DocError<'l>> {
        let i = self.ast.goto_parent();
        let insertion_index = if before { i } else { i + 1 };
        match self.ast.inner() {
            AstKind::Fixed(mut fixed) => {
                // Oops, go back, we can't insert something into a fixed node.
                fixed.goto_child(i);
                Err(DocError::WrongSort(new_ast))
            }
            AstKind::Flexible(mut flexible) => {
                let result = flexible
                    .insert_child(insertion_index, new_ast)
                    .map_err(|rejected_ast| DocError::WrongSort(rejected_ast));
                // Go back to the node we started on, before possibly
                // early-returning an error.
                flexible.goto_child(insertion_index);
                result?;
                if before && i != 0 {
                    Ok(vec![TreeNavCmd::Right.into(), TreeCmd::Remove.into()])
                } else {
                    Ok(vec![TreeCmd::Remove.into()])
                }
            }
            _ => panic!("how can a parent not be fixed or flexible?"),
        }
    }

    /// Used for both cutting and deleting. If return_original is true, cut the
    /// selected Ast, return the original Ast, and return Undo commands
    /// containing a _copy_ of the Ast. Otherwise, return None and use the
    /// original Ast in the Undo commands.
    fn remove(
        &mut self,
        return_original: bool,
    ) -> Result<(Vec<Command<'l>>, Option<Ast<'l>>), DocError<'l>> {
        let i = self.ast.goto_parent();
        match self.ast.inner() {
            AstKind::Fixed(mut fixed) => {
                // Oops, go back, we can't delete something from a fixed node.
                fixed.goto_child(i);
                Err(DocError::CannotRemoveNode)
            }
            AstKind::Flexible(mut flexible) => {
                let old_ast = flexible.remove_child(i);
                let (undo_ast, returned_ast) = if return_original {
                    (old_ast.clone(), Some(old_ast))
                } else {
                    (old_ast, None)
                };
                let num_children = flexible.num_children();
                let undos = if num_children == 0 {
                    // Stay at the childless parent
                    vec![TreeCmd::InsertPrepend(undo_ast).into()]
                } else if i == 0 {
                    // We removed the first child, so to undo must insert before.
                    flexible.goto_child(0);
                    vec![TreeCmd::InsertBefore(undo_ast).into()]
                } else {
                    // Go to the left.
                    flexible.goto_child(i - 1);
                    vec![TreeCmd::InsertAfter(undo_ast).into()]
                };
                Ok((undos, returned_ast))
            }
            _ => panic!("how can a parent not be fixed or flexible?"),
        }
    }
}
