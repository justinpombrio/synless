use editor::NotationSet;

mod data;
mod engine;
mod error;
mod keymaps;
mod prog;
mod shell_editor;

use editor::NotationSets;
use language::LanguageSet;
use shell_editor::ShellEditor;

fn main() {
    let language_set = LanguageSet::new();
    let notation_sets = NotationSets::new();
    match ShellEditor::new(&language_set, &notation_sets) {
        Ok(mut ed) => {
            let result = ed.run();
            drop(ed);
            if let Err(err) = result {
                println!("Error: {}", err);
            }
        }
        Err(err) => println!("Failed to create editor: {}", err),
    };
}
