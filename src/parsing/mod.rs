mod json_parser;

use crate::language::Storage;
use crate::tree::Node;
use crate::util::{error, SynlessError};
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
