use crate::language::{Construct, Storage};
use crate::tree::Node;
use crate::util::{error, SynlessError};
use regex::{self, Regex};

#[derive(thiserror::Error, Debug)]
pub enum SearchError {
    #[error("{}", .0)]
    InvalidRegex(regex::Error),
}

impl From<SearchError> for SynlessError {
    fn from(error: SearchError) -> SynlessError {
        error!(Edit, "{}", error)
    }
}

#[derive(Debug)]
pub struct Search {
    pattern: SearchPattern,
    pub highlight: bool,
}

#[derive(Debug)]
enum SearchPattern {
    /// Matches nodes of the given construct.
    Construct(Construct),
    /// Matches nodes that are identical to the given node, including children.
    Node(Node),
    /// Matches texty nodes whose text contains the given substring.
    Substring(String),
    /// Matches texty nodes whose text matches the given regex.
    Regex(Regex),
}

impl Search {
    pub fn new_construct(construct: Construct) -> Search {
        Search {
            pattern: SearchPattern::Construct(construct),
            highlight: true,
        }
    }

    pub fn new_node(node: Node) -> Search {
        Search {
            pattern: SearchPattern::Node(node),
            highlight: true,
        }
    }

    pub fn new_substring(substring: String) -> Search {
        Search {
            pattern: SearchPattern::Substring(substring),
            highlight: true,
        }
    }

    pub fn new_regex(regex_pattern: &str) -> Result<Search, SearchError> {
        let regex = Regex::new(regex_pattern).map_err(SearchError::InvalidRegex)?;
        Ok(Search {
            pattern: SearchPattern::Regex(regex),
            highlight: true,
        })
    }

    pub fn matches(&self, s: &Storage, node: Node) -> bool {
        match &self.pattern {
            SearchPattern::Construct(construct) => node.construct(s) == *construct,
            SearchPattern::Node(expected_node) => expected_node.equals(s, node),
            SearchPattern::Substring(substring) => node
                .text(s)
                .map(|text| text.as_str().contains(substring))
                .unwrap_or(false),
            SearchPattern::Regex(regex) => node
                .text(s)
                .map(|text| regex.is_match(text.as_str()))
                .unwrap_or(false),
        }
    }

    pub fn delete(self, s: &mut Storage) {
        use SearchPattern as P;

        match self.pattern {
            P::Node(node) => node.delete_root(s),
            P::Construct(_) | P::Substring(_) | P::Regex(_) => (),
        }
    }
}
