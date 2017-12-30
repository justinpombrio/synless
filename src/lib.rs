#![feature(slice_patterns, advanced_slice_patterns)]
#![feature(box_patterns)]
extern crate rustbox;
#[macro_use]
extern crate lazy_static;

#[allow(unused_macros)]
macro_rules! debug {
    ($($arg:tt)*) => ({
        use std::io::Write;
        match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr (file handle closed?): {}", x),
        }
    })
}

// TODO: Make terminal, doc private
// TODO: Test weird unicode chars. (Look for `.chars()`)
// TODO: Audit uses of pub
// TODO: Test render::render


#[macro_use]
pub mod coord;     // commonly used; not listed as dep
pub mod style;     // commonly used; not listed as dep

pub mod terminal;  // dep:
pub mod syntax;    // dep:
pub mod language;  // dep: syntax
pub mod tree;      // dep: syntax
pub mod doc;       // dep: tree, syntax
mod write;         // dep: doc, syntax
pub mod render;    // dep: syntax, doc, terminal
pub mod editor;    // dep: tree, doc, terminal, render


// API:
pub use language::Language;
pub use tree::Tree;
pub use editor::Editor;
