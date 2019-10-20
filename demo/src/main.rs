use editor::NotationSet;

mod data;
mod engine;
mod error;
mod keymaps;
mod prog;
mod shell_editor;

use error::ShellError;
use shell_editor::ShellEditor;

fn main() -> Result<(), ShellError> {
    let mut ed = ShellEditor::new()?;
    let result = ed.run();
    drop(ed);
    if let Err(err) = result {
        println!("Error: {}", err);
    }
    println!("Exited alternate screen. Your cursor should be visible again.");
    Ok(())
}
