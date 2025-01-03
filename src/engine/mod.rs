mod command;
mod doc;
mod doc_set;
mod engine;
mod search;

use partial_pretty_printer as ppp;
use std::default::Default;

pub use command::{
    BookmarkCommand, ClipboardCommand, SearchCommand, TextEdCommand, TextNavCommand, TreeEdCommand,
    TreeNavCommand,
};
pub use doc_set::{DocDisplayLabel, DocName};
pub use engine::Engine;
pub use search::Search;

#[derive(Debug, Clone)]
pub struct Settings {
    max_source_width: ppp::Width,
    max_display_width: ppp::Width,
    focus_height: f32,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            max_source_width: 100,
            max_display_width: 120,
            focus_height: 0.25,
        }
    }
}
