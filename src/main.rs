extern crate synless;

use synless::Tree;
use synless::Language;
use synless::KeyMap;
use synless::Editor;

fn main() {
    let language = Language::example_language();
    let keymap = KeyMap::example_keymap(&language);
    let mut tree = Tree::new_hole();
    let mut editor = Editor::new(&language, keymap, &mut tree);
    editor.run();
}
