use editor::NotationSet;

mod data;
mod engine;
mod error;
mod keymaps;
mod prog;
mod server;

use editor::NotationSets;
use language::LanguageSet;
use server::Server;

fn main() {
    let language_set = LanguageSet::new();
    let notation_sets = NotationSets::new();
    match Server::new(&language_set, &notation_sets) {
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
