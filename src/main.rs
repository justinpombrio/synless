extern crate synless;

use synless::ColorTheme;
use synless::Tree;
use synless::Language;
use synless::KeyMap;
use synless::Editor;

fn main() {
    let language = Language::example_language();
    let keymap = KeyMap::example_keymap(&language);
    let theme = ColorTheme::colorful_hexagon();
    let mut tree = Tree::new_hole();
    let mut editor = Editor::new(&language, keymap, theme, &mut tree);
    editor.run();
}
