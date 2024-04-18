// TODO: temporary
#![allow(unused)]

mod engine;
mod frontends;
mod keymap;
mod language;
mod pretty_doc;
mod runtime;
mod style;
mod tree;
mod util;

pub mod parsing;

pub use engine::{DocName, Engine, Settings};
pub use frontends::Terminal;
pub use keymap::KeyProg;
pub use language::{
    AritySpec, ConstructSpec, GrammarSpec, LanguageSpec, NotationSetSpec, SortSpec, Storage,
};
pub use pretty_doc::DocRef;
pub use runtime::Runtime;
pub use style::ColorTheme;
pub use tree::{Location, Node};
pub use util::{Log, LogEntry, LogLevel, SynlessBug, SynlessError};
