use super::measure::MeasuredNotation;
use MeasuredNotation::*;

struct Block<'a>(Vec<Part<'a>>);

enum Part<'a> {
    String(String),
    Notation {
        indent: usize,
        suffix_len: usize,
        notation: &'a MeasuredNotation,
    },
}

fn partially_expand<'a>(
    indent: usize,
    suffix_len: usize,
    notation: &'a MeasuredNotation,
) -> Vec<Block<'a>> {
    let mut expander = PartialExpander::new();
    expander.expand(indent, suffix_len, notation);
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

    fn expand(&mut self, indent: usize, suffix_len: usize, notation: &'a MeasuredNotation) {
        match notation {
            Literal(lit) => self.parts.push(Part::String(lit.to_string())),
            Newline => {
                let parts = self.parts.drain(..).collect();
                self.blocks.push(Block(parts));
            }
            Indent(i, note) => self.expand(indent + i, suffix_len, note),
            Concat(left, right, right_req) => {
                self.expand(indent, right_req.suffix_len(suffix_len), left);
                self.expand(indent, suffix_len, right);
            }
            Flat(_) | Nest(_, _) | Choice(_, _) => {
                self.parts.push(Part::Notation {
                    indent,
                    suffix_len,
                    notation,
                });
            }
        }
    }
}
