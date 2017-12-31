extern crate tree_editor;

use tree_editor::Tree;
use tree_editor::Language;
use tree_editor::KeyMap;
use tree_editor::Editor;

fn main() {
    let language = Language::example_language();
    let keymap = KeyMap::example_keymap(&language);
    let mut tree = Tree::new_hole();
    let mut editor = Editor::new(&language, keymap, &mut tree);
    editor.run();
}
