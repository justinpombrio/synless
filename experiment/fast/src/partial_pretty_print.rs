use super::measure::{MeasuredNotation, Requirement};
use MeasuredNotation::*;

struct Block<'a>(Vec<Part<'a>>);

enum Part<'a> {
    String(String),
    Notation {
        indent: usize,
        suffix_req: Requirement,
        notation: &'a MeasuredNotation,
    },
}

fn partially_expand<'a>(
    indent: usize,
    suffix_req: Requirement,
    notation: &'a MeasuredNotation,
) -> Vec<Block<'a>> {
    let mut expander = PartialExpander::new();
    expander.expand(indent, suffix_req, notation);
    expander.finish()
}

/// Points between Parts
struct PartPos {
    block: usize,
    part: usize,
}

struct PartialExpander<'a> {
    blocks: Vec<Block<'a>>,
    parts: Vec<Part<'a>>,
    cursor_start: Option<PartPos>,
    cursor_end: Option<PartPos>,
}

impl<'a> PartialExpander<'a> {
    fn new() -> PartialExpander<'a> {
        PartialExpander {
            blocks: vec![],
            parts: vec![],
            cursor_start: None,
            cursor_end: None,
        }
    }

    fn finish(mut self) -> Vec<Block<'a>> {
        let parts = self.parts.drain(..).collect();
        self.blocks.push(Block(parts));
        self.blocks
    }

    fn expand(&mut self, indent: usize, suffix_req: Requirement, notation: &'a MeasuredNotation) {
        match notation {
            Literal(lit) => self.parts.push(Part::String(lit.to_string())),
            Newline => {
                let parts = self.parts.drain(..).collect();
                self.blocks.push(Block(parts));
            }
            Indent(i, note) => self.expand(indent + i, suffix_req, note),
            Concat(left, right, right_req) => {
                self.expand(indent, right_req.concat(suffix_req), left);
                self.expand(indent, suffix_req, right);
            }
            Flat(_) | Align(_) | Choice(_, _) => {
                self.parts.push(Part::Notation {
                    indent,
                    suffix_req,
                    notation,
                });
            }
        }
    }
}
