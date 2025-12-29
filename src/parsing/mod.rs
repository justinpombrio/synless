mod json_parser;
mod json_tokenizer;

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

/// An error while parsing a file. If a location is given, labels the line it's on. For example:
///
/// ```
/// # use synless::parsing::ParseError;
/// # use partial_pretty_printer::Pos;
/// let error_without_location =
///     ParseError::without_location("git-references.md".to_owned(), "Bad writing.".to_owned());
/// assert_eq!(
///     error_without_location.to_string(),
///     "In git-references.md: Bad writing."
/// );
/// ```
///
/// ```
/// # use synless::parsing::ParseError;
/// # use partial_pretty_printer::Pos;
/// let file_contents = "\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\nOh shit, git!\n\n";
/// let error_with_location = ParseError::with_location(
///     "git-references.md".to_owned(),
///     "Profanity is unprofessional.".to_owned(),
///     file_contents,
///     Pos { row: 16, col: 3 },
///     "profane".to_owned(),
/// );
/// assert_eq!(
///     error_with_location.to_string(),
///     [
///         "In git-references.md at 17:4: Profanity is unprofessional.",
///         "17 |Oh shit, git!",
///         "       ^ profane",
///     ]
///     .join("\n"),
/// );
/// ```
#[derive(Debug)]
pub struct ParseError {
    file_name: String,
    message: String,
    location: Option<ParseErrorLocation>,
}

#[derive(Debug)]
pub struct ParseErrorLocation {
    pos: ppp::Pos,
    line: String,
    label: String,
}

impl ParseError {
    pub fn without_location(file_name: String, message: String) -> ParseError {
        ParseError {
            file_name,
            message,
            location: None,
        }
    }

    pub fn with_location(
        file_name: String,
        message: String,
        file_contents: &str,
        pos: ppp::Pos,
        label: String,
    ) -> ParseError {
        let line = file_contents
            .lines()
            .nth(pos.row as usize)
            .unwrap_or("")
            .to_owned();
        ParseError {
            file_name,
            message,
            location: Some(ParseErrorLocation { pos, line, label }),
        }
    }

    pub fn from_ron_error(
        filepath: &Path,
        file_contents: &str,
        error: ron::error::SpannedError,
    ) -> ParseError {
        // Serde ron uses 1-indexed positions, with 0,0 as a sentinel value.
        // We use 0-indexed positions.
        let pos = if error.position.line == 0 || error.position.col == 0 {
            None
        } else {
            Some(ppp::Pos {
                row: error.position.line as ppp::Row - 1,
                col: error.position.col as ppp::Col - 1,
            })
        };

        let file_name = filepath.to_string_lossy().into_owned();
        let message = format!("{}", error.code);
        match pos {
            None => ParseError::without_location(file_name, message),
            Some(pos) => {
                ParseError::with_location(file_name, message, file_contents, pos, "here".to_owned())
            }
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let file = &self.file_name;
        let message = &self.message;
        match &self.location {
            None => write!(f, "In {file}: {message}"),
            Some(loc) => {
                let row = loc.pos.row + 1;
                let col = loc.pos.col + 1;
                writeln!(f, "In {file} at {row}:{col}: {message}")?;
                let line = &loc.line;
                let spacing = row.to_string().len() + col as usize;
                let label = &loc.label;
                writeln!(f, "{row} |{line}")?;
                write!(f, "{:>spacing$} ^ {label}", "", spacing = spacing)
            }
        }
    }
}

impl From<ParseError> for SynlessError {
    fn from(error: ParseError) -> SynlessError {
        error!(Parse, "{}", error)
    }
}
