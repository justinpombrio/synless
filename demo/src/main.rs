use editor::NotationSet;

mod core_editor;
mod data;
mod error;
mod keymap;
mod prog;
mod shell_editor;

use error::ShellError;
use shell_editor::ShellEditor;

fn main() -> Result<(), ShellError> {
    let mut ed = ShellEditor::new()?;
    let err = ed.run();
    drop(ed);
    println!("Error: {:?}", err);
    println!("Exited alternate screen. Your cursor should be visible again.");
    Ok(())
}
