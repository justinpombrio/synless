// TODO: dis-entangle these files. Each of them depends on all the others!

mod plain_text;
mod pretty_doc;
mod pretty_print;
mod pretty_window;
mod viewport;

// TODO: temp
pub use plain_text::PlainText;
pub use pretty_doc::PrettyDocument;
pub use pretty_window::PrettyWindow;

/*
pub use plain_text::PlainText;
pub use pretty_doc::{DocPosSpec, PrettyDocument};
pub use pretty_window::PrettyWindow;
*/
