use std::fmt;

use construct::Construct;
use tree::{Path, Tree, TreeRef, TreeMut};

use self::Mode::*;

/// Indicates whether the cursor is currently editing a tree or text.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    TreeMode,
    TextMode
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &TreeMode => write!(f, "Tree"),
            &TextMode => write!(f, "Text")
        }
    }
}

/// The cursor navigates and edits a document.
pub struct Cursor<'t, 'l : 't> {
    char_index: Option<usize>,
    tree:  TreeMut<'t, 'l>
}

impl<'t, 'l : 't> Cursor<'t, 'l> {

    pub fn new(tree: &'t mut Tree<'l>) -> Cursor<'t, 'l> {
        Cursor{
            char_index: None,
            tree: tree.as_mut()
        }
    }

    // Info //

    /// The root of the document.
    pub fn root(&'t self) -> TreeRef<'t, 'l> {
        self.tree.as_ref().root()
    }

    /// The tree at the current cursor location.
    pub fn as_ref(&'t self) -> TreeRef<'t, 'l> {
        self.tree.as_ref()
    }

    /// The current cursor location.
    pub fn path(&self) -> Path {
        self.tree.path().clone()
    }

    /// If editing text, the character index, else None.
    pub fn char_index(&self) -> Option<usize> {
        self.char_index
    }

    // Tree Navigation //

    /// Attempt to move right (i.e., to the next tree sibling).
    ///
    /// This will fail if the cursor is in text mode,
    /// or if there is no right sibling.
    ///
    /// Returns `true` if movement was successful, or `false` otherwise.
    pub fn right(&mut self) -> bool {
        if self.mode() == TreeMode && self.tree.has_parent() {
            let i = self.tree.index();
            if i + 1 < self.tree.num_siblings() {
                self.tree.goto_parent();
                self.tree.goto_child(i + 1);
                return true;
            }
        }
        false
    }

    /// Attempt to move left (i.e., to the previous tree sibling).
    ///
    /// This will fail if the cursor is in text mode,
    /// or if there is no left sibling.
    ///
    /// Returns `true` if movement was successful, or `false` otherwise.
    pub fn left(&mut self) -> bool {
        if self.mode() == TreeMode && self.tree.has_parent() {
            let i = self.tree.index();
            if i > 0 {
                self.tree.goto_parent();
                self.tree.goto_child(i - 1);
                return true;
            }
        }
        false
    }

    /// Attempt to move up (i.e., to the parent node).
    ///
    /// This will fail if the cursor is in text mode,
    /// or if it is at the root of the document.
    ///
    /// Returns `true` if movement was successful, or `false` otherwise.
    pub fn up(&mut self) -> bool {
        if self.mode() == TreeMode && self.tree.has_parent() {
            let index = self.tree.goto_parent();
            *self.tree.breadcrumb_mut() = index;
            return true;
        }
        false
    }

    /// Attempt to move down (i.e., to a child).
    ///
    /// This will go to the child that was last visited.
    /// It will fail if the cursor is in text mode,
    /// or if it is at a tree that has no children.
    ///
    /// Returns `true` if movement was successful, or `false` otherwise.
    pub fn down(&mut self) -> bool {
        if self.mode() == TreeMode && self.tree.is_foresty()
            && self.tree.num_children() > 0
        {
            let index = self.tree.breadcrumb();
            self.tree.goto_child(index);
            return true;
        }
        false
    }

    // Text Navigation //

    /// Attempt to move right one character.
    ///
    /// This will fail if the cursor is in tree mode,
    /// or if it is at the end of the text.
    ///
    /// Returns `true` if movement was successful, or `false` otherwise.
    pub fn right_char(&mut self) -> bool {
        if let Some(i) = self.char_index {
            let text = self.tree.text();
            if i < text.chars().count() {
                self.char_index = Some(i + 1);
                return true;
            }
        }
        false
    }

    /// Attempt to move left one character.
    ///
    /// This will fail if the cursor is in tree mode,
    /// or if it is at the beginning of the text.
    ///
    /// Returns `true` if movement was successful, or `false` otherwise.
    pub fn left_char(&mut self) -> bool {
        if let Some(i) = self.char_index {
            if i > 0 {
                self.char_index = Some(i - 1);
                return true;
            }
        }
        false
    }

    // Modes //

    /// Is the cursor currently selecting a tree, or text?
    pub fn mode(&self) -> Mode {
        if self.char_index.is_some() {
            TextMode
        } else {
            TreeMode
        }
    }

    /// Attempt to enter text mode.
    ///
    /// This will go to the last selected character position in the text
    /// (or the end if it has not been selected yet).
    /// It will fail if the cursor is already in text mode,
    /// or if the tree selected is not texty.
    ///
    /// Returns `true` if the mode change was successful, or `false` otherwise.
    pub fn enter_text(&mut self) -> bool {
        if self.mode() == TreeMode && self.tree.is_texty() {
            self.char_index = Some(self.tree.breadcrumb());
            return true;
        }
        false
    }

    /// Attempt to exit text mode.
    ///
    /// This will fail if the cursor is already in tree mode.
    ///
    /// Returns `true` if the mode change was successful, or `false` otherwise.
    pub fn exit_text(&mut self) -> bool {
        if self.mode() == TextMode {
            *self.tree.breadcrumb_mut() = self.char_index.unwrap();
            self.char_index = None;
            return true;
        }
        false
    }

    // Tree Operations

    /// Add a child to the end of an extendable tree.
    ///
    /// This will fail if the cursor is in text mode.
    ///
    /// Returns `true` if successful.
    pub fn add_child(&mut self) -> bool {
        if self.tree.is_foresty() && self.tree.node().is_extendable() {
            self.tree.forest_mut().push(Tree::new_hole());
            self.update();
            return true;
        }
        false
    }

    /// Replace the selected tree with a new empty tree.
    ///
    /// This will fail if the cursor is in text mode.
    ///
    /// Returns the tree that was replaced.
    pub fn replace_tree(&mut self, construct: &'l Construct) -> Option<Tree<'l>> {
        if self.mode() == TreeMode {
            let tree = self.tree.replace(Tree::new(construct));
            self.update();
            return Some(tree);
        }
        None
    }

    /// Delete the selected tree.
    ///
    /// This will fail if the cursor is in text mode.
    ///
    /// Returns the tree that was deleted.
    pub fn delete_tree(&mut self) -> Option<Tree<'l>> {
        if self.mode() == TreeMode {
            let tree = self.tree.replace(Tree::new_hole());
            self.update();
            return Some(tree);
        }
        None
    }

    // Text Operations

    /// Insert a character after the selected position.
    ///
    /// This will fail if the cursor is in tree mode.
    ///
    /// Returns `true` if successful.
    pub fn insert_char(&mut self, ch: char) -> bool {
        if let Some(i) = self.char_index {
            self.tree.text_mut().insert(i, ch);
            self.char_index = Some(i + 1);
            self.update();
            return true;
        }
        false
    }

    /// Delete the character before the selected position.
    ///
    /// This will fail if the cursor is in tree mode.
    ///
    /// Returns the deleted character.
    pub fn delete_char(&mut self) -> Option<char> {
        if let Some(i) = self.char_index {
            if i >= 1 {
                let ch = self.tree.text_mut().remove(i - 1);
                self.char_index = Some(i - 1);
                self.update();
                return Some(ch)
            }
        }
        None
    }

    /////////////
    // PRIVATE //
    /////////////

    fn update(&mut self) {
        self.tree.update();
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use construct::{TEST_TEXT, TEST_FOREST};
    use tree::Tree;

    fn t(children: Vec<Tree<'static>>) -> Tree<'static> {
        Tree::new_forest(&TEST_FOREST, children)
    }

    fn bt(k: usize) -> Tree<'static> {
        // What happens when you can make a doorway that
        // connects the universe to a copy of itself,
        // and then one of you makes n doorways?
        t((0..k).map(|i| bt(i)).collect())
    }

    fn tt() -> Tree<'static> {
        Tree::new_forest(&TEST_FOREST, vec!(
            Tree::new_text(&TEST_TEXT, "hi"),
            Tree::new_text(&TEST_TEXT, "hey")))
    }

    #[test]
    fn test_tree_navigation() {
        let mut tree = bt(3);
        let mut doc = Cursor::new(&mut tree);
        
        assert_eq!(doc.path(), vec!());
        // Down
        assert_eq!(doc.down(), true);
        assert_eq!(doc.path(), vec!(0));
        // Down too far
        assert_eq!(doc.down(), false);
        assert_eq!(doc.path(), vec!(0));
        // Right
        assert_eq!(doc.right(), true);
        assert_eq!(doc.path(), vec!(1));
        // Right too far
        assert_eq!(doc.right(), true);
        assert_eq!(doc.right(), false);
        assert_eq!(doc.path(), vec!(2));
        // Left
        assert_eq!(doc.left(), true);
        assert_eq!(doc.path(), vec!(1));
        // Left too far
        assert_eq!(doc.left(), true);
        assert_eq!(doc.left(), false);
        assert_eq!(doc.path(), vec!(0));
        // Something to remember
        assert_eq!(doc.right(), true);
        assert_eq!(doc.right(), true);
        assert_eq!(doc.down(), true);
        assert_eq!(doc.path(), vec!(2, 0));
        assert_eq!(doc.right(), true);
        assert_eq!(doc.path(), vec!(2, 1));
        // Up
        assert_eq!(doc.up(), true);
        assert_eq!(doc.path(), vec!(2));
        assert_eq!(doc.up(), true);
        assert_eq!(doc.path(), vec!());
        // Up too far
        assert_eq!(doc.up(), false);
        assert_eq!(doc.path(), vec!());
        // Direct memory
        assert_eq!(doc.down(), true);
        assert_eq!(doc.down(), true);
        assert_eq!(doc.path(), vec!(2, 1));
        assert_eq!(doc.down(), true);
        assert_eq!(doc.path(), vec!(2, 1, 0));
        // Indirect memory
        assert_eq!(doc.up(), true);
        assert_eq!(doc.up(), true);
        assert_eq!(doc.left(), true);
        assert_eq!(doc.up(), true);
        assert_eq!(doc.down(), true);
        assert_eq!(doc.down(), true);
        assert_eq!(doc.path(), vec!(1, 0));
        assert_eq!(doc.up(), true);
        assert_eq!(doc.right(), true);
        assert_eq!(doc.down(), true);
        assert_eq!(doc.down(), true);
        assert_eq!(doc.path(), vec!(2, 1, 0));
    }

    #[test]
    fn test_text_navigation() {
        let mut tree = tt();
        let mut doc = Cursor::new(&mut tree);
        
        // Cannot enter text
        assert_eq!(doc.mode(), TreeMode);
        assert_eq!(doc.enter_text(), false);
        assert_eq!(doc.mode(), TreeMode);
        assert_eq!(doc.char_index(), None);

        // Can enter text
        assert_eq!(doc.down(), true);
        assert_eq!(doc.right(), true);
        assert_eq!(doc.char_index(), None);
        assert_eq!(doc.enter_text(), true);
        assert_eq!(doc.mode(), TextMode);
        assert_eq!(doc.char_index(), Some(3));

        // Can navigate
        assert_eq!(doc.left_char(), true);
        assert_eq!(doc.left_char(), true);
        assert_eq!(doc.left_char(), true);
        assert_eq!(doc.left_char(), false);
        assert_eq!(doc.right_char(), true);
        assert_eq!(doc.right_char(), true);
        assert_eq!(doc.left_char(), true);
        assert_eq!(doc.right_char(), true);
        assert_eq!(doc.right_char(), true);
        assert_eq!(doc.right_char(), false);
        assert_eq!(doc.left_char(), true);
        assert_eq!(doc.left_char(), true);
        assert_eq!(doc.char_index(), Some(1));
        
        // Can exit text
        assert_eq!(doc.exit_text(), true);
        assert_eq!(doc.mode(), TreeMode);
        assert_eq!(doc.char_index(), None);

        // Character position is remembered
        assert_eq!(doc.enter_text(), true);
        assert_eq!(doc.char_index(), Some(1));

        // Character position is not infectious
        assert_eq!(doc.exit_text(), true);
        assert_eq!(doc.left(), true);
        assert_eq!(doc.enter_text(), true);
        assert_eq!(doc.char_index(), Some(2));
    }

    #[test]
    fn test_text_ops() {
        // Tests for: insert char, delete char
        let mut tree = tt();
        let mut doc = Cursor::new(&mut tree);
        
        doc.down();
        doc.right();
        doc.enter_text();
        assert_eq!(doc.delete_char(), Some('y'));
        doc.left_char();
        doc.left_char();
        assert_eq!(doc.delete_char(), None);
        assert_eq!(doc.insert_char('l'), true);
        doc.right_char();
        doc.right_char();
        assert_eq!(doc.insert_char('r'), true);
        doc.exit_text();
        assert_eq!(doc.as_ref().text(), "lher");
    }

    #[test]
    fn test_tree_ops() {
        // Tests for: add child, replace tree, delete tree
        let mut tree = bt(3);
        let mut doc = Cursor::new(&mut tree);

        assert_eq!(doc.add_child(), true);
        doc.down();
        doc.right();
        doc.right();
        doc.right();
        assert_eq!(doc.as_ref().len(), 0);
        doc.left();
        let opt_tree = doc.delete_tree();
        assert!(opt_tree.is_some());
        assert_eq!(opt_tree.unwrap().len(), 2);
        assert_eq!(doc.as_ref().len(), 0);
        doc.left();
        let opt_tree = doc.replace_tree(&TEST_TEXT);
        assert!(opt_tree.is_some());
        assert_eq!(opt_tree.unwrap().len(), 1);
        assert_eq!(&doc.as_ref().node().construct().name, "TEST_TEXT");

        let mut tree = tt();
        let mut doc = Cursor::new(&mut tree);
        doc.down();
        assert_eq!(doc.add_child(), false);
    }

    #[test]
    fn test_as_ref() {
        let mut tree = tt();
        let mut doc = Cursor::new(&mut tree);

        doc.down();
        doc.right();
        assert_eq!(doc.root().path(), &vec!());
        assert_eq!(doc.as_ref().path(), &vec!(1));
    }
}
