#![feature(slice_patterns)]
#![feature(box_patterns)]
// TODO: This is silencing errors in the dependencies
#![allow(intra_doc_link_resolution_failure)]
extern crate rustbox;
#[macro_use]
extern crate lazy_static;
extern crate uuid;

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


pub mod common;    // commonly used; not listed as dep
pub mod style;     // commonly used; not listed as dep

pub mod frontends; // dep: rustbox
pub mod syntax;    // dep:
pub mod forest;    // dep:
pub mod language;  // dep:
/*
pub mod tree;      // dep: syntax
pub mod render;    // dep: syntax, doc, terminal
pub mod editor;    // dep: language, tree, doc, terminal, render

// API:
pub use language::Language;
pub use tree::Tree;
pub use editor::KeyMap;
pub use editor::Editor;
pub use style::ColorTheme;
*/
