mod doc;
mod doc_command;
mod doc_set;
mod engine;

use partial_pretty_printer as ppp;

#[derive(Debug, Clone)]
pub struct Settings {
    source_width: ppp::Width,
    max_doc_width: ppp::Width,
    focus_height: f32,
}

impl Settings {
    fn default() -> Settings {
        Settings {
            source_width: 100,
            max_doc_width: 80,
            focus_height: 0.5,
        }
    }
}
