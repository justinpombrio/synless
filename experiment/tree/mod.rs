//! Synless is a tree editor. Here are the trees.
mod path;
mod tree;
mod tree_ref;
mod tree_mut;
mod cursor;

pub use tree::tree_ref::TreeRef;
pub use tree::tree_mut::TreeMut;
pub use tree::tree::{Tree};
pub use tree::path::{Path, extend_path, pop_path, match_end_of_path};
pub use tree::cursor::{Cursor, Mode};
