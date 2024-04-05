mod json_parser;

use crate::language::{LanguageError, Storage};
use crate::tree::Node;
use partial_pretty_printer as ppp;
use std::fmt;

pub use json_parser::JsonParser;

#[derive(Debug)]
pub struct ParseError {
    pub pos: Option<ppp::Pos>,
    pub message: String,
}

pub trait Parse: fmt::Debug {
    fn name(&self) -> &str;
    fn parse(&mut self, s: &mut Storage, source: &str) -> Result<Node, ParseError>;
}

impl From<LanguageError> for ParseError {
    fn from(err: LanguageError) -> ParseError {
        ParseError {
            pos: None,
            message: format!("{}", err),
        }
    }
}
