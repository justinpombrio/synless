mod json_parser;

use crate::language::{Arity, Storage};
use crate::tree::Node;
use crate::util::{bug, error, SynlessError};
use partial_pretty_printer as ppp;
use std::fmt;
use std::path::Path;

pub use json_parser::JsonParser;

pub trait Parse: fmt::Debug {
    fn name(&self) -> &str;

    fn parse(
        &mut self,
        s: &mut Storage,
        file_name: &str,
        source: &str,
    ) -> Result<Node, SynlessError>;
}

/// Convert holes in `source` from `invalid_hole_syntax` to `valid_hole_syntax`, so that they can
/// be parsed with a standard parser for the language.
pub fn preprocess(source: &str, invalid_hole_syntax: &str, valid_hole_syntax: &str) -> String {
    source.replace(invalid_hole_syntax, valid_hole_syntax)
}

/// Find every texty node within `root` that contains `hole_text` with a hole. If its parent is
/// fixed, replace it with a hole, otherwise delete it (since you can't have holes under a listy
/// parent).
pub fn postprocess(s: &mut Storage, root: Node, hole_text: &str) {
    root.walk_tree(s, |s: &mut Storage, node: Node| {
        if let Some(text) = node.text(s) {
            if text.as_str() == hole_text {
                let should_delete = if let Some(parent) = node.parent(s) {
                    matches!(parent.arity(s), Arity::Listy(_))
                } else {
                    false
                };
                if should_delete {
                    let _ = node.detach(s);
                    node.delete_root(s);
                } else {
                    let hole = Node::new_hole(s, node.language(s));
                    if !node.swap(s, hole) {
                        bug!("Failed to replace node with hole in parser postprocess()")
                    }
                }
            }
        }
    });
}

#[derive(Debug)]
pub struct ParseError {
    pub pos: Option<ppp::Pos>,
    pub file_name: String,
    pub message: String,
}

impl ParseError {
    pub fn from_ron_error(filepath: &Path, error: ron::error::SpannedError) -> ParseError {
        // Serde ron uses 1-indexed positions, with 0,0 as a sentinel value.
        // We use 0-indexed positions.
        let (row, col) = (
            error.position.line as ppp::Row,
            error.position.col as ppp::Col,
        );
        let pos = if row == 0 || col == 0 {
            None
        } else {
            Some(ppp::Pos {
                row: row - 1,
                col: col - 1,
            })
        };
        ParseError {
            file_name: filepath.to_string_lossy().into_owned(),
            pos,
            message: format!("{}", error.code),
        }
    }
}

impl From<ParseError> for SynlessError {
    fn from(error: ParseError) -> SynlessError {
        let doc = error.file_name;
        let message = error.message;
        if let Some(ppp::Pos { row, col }) = error.pos {
            let line = row + 1;
            let col = col + 1;
            error!(Parse, "In {doc} at {line}:{col}: {message}")
        } else {
            error!(Parse, "In {doc}: {message}")
        }
    }
}
