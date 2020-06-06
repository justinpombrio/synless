// TODO:
// - general cleanup
// - don't pass width around everywhere
// - don't stick spaces in a string
// - functionality: last lines, seeking, multiple expand

use super::measure::MeasuredNotation;

pub struct PartialPrettyPrinter<'n> {
    blocks: Vec<Block<'n>>,
    width: usize,
}

struct Block<'n> {
    chunks: Vec<Chunk<'n>>,
}

enum Chunk<'n> {
    Text(String),
    Notation {
        indent: Option<usize>, // None means flat
        notation: &'n MeasuredNotation,
    },
}

impl<'n> PartialPrettyPrinter<'n> {
    pub fn new(notation: &'n MeasuredNotation, width: usize) -> PartialPrettyPrinter<'n> {
        let chunk = Chunk::Notation {
            indent: Some(0),
            notation,
        };
        let block = Block {
            chunks: vec![chunk],
        };
        PartialPrettyPrinter {
            blocks: vec![block],
            width,
        }
    }

    // TODO: Handle multiple blocks; multiple expansions
    pub fn first_lines(mut self, num_lines: usize) -> Vec<String> {
        assert_eq!(self.blocks.len(), 1);
        let block = self.blocks.pop().unwrap();
        block.expand_first_lines(num_lines, self.width).0
    }
}

fn expand_first_line<'n>(
    prefix: Option<String>,
    indent: Option<usize>, // None means flat
    notation: &'n MeasuredNotation,
    mut remaining: Block<'n>,
    width: usize,
) -> (String, Vec<Block<'n>>) {
    use MeasuredNotation::*;

    match notation {
        Empty => {
            if let Some(prefix) = prefix {
                remaining.prepend_text(prefix);
            }
            remaining.expand_first_line(width)
        }
        Literal(text) => {
            remaining.prepend_text(text.to_string());
            if let Some(prefix) = prefix {
                remaining.prepend_text(prefix);
            }
            remaining.expand_first_line(width)
        }
        Newline => {
            let spaces = " ".repeat(indent.unwrap());
            remaining.prepend_text(spaces);
            (prefix.unwrap_or_default(), vec![remaining])
        }
        Indent(inner_indent, inner_notation) => {
            let full_indent = indent.map(|i| i + inner_indent);
            expand_first_line(prefix, full_indent, inner_notation, remaining, width)
        }
        Flat(inner_notation) => expand_first_line(prefix, None, inner_notation, remaining, width),
        Align(_) => unimplemented!(),
        Concat(left, right, _) => {
            remaining.prepend_notation(indent, right);
            expand_first_line(prefix, indent, left, remaining, width)
        }
        Choice(choice) => {
            if let Some(chosen_notation) = choice.sole_option(indent.is_none()) {
                return expand_first_line(prefix, indent, chosen_notation, remaining, width);
            }
            let (suffix, mut right_blocks) = remaining.expand_first_line(width);
            let prefix_len = prefix.as_ref().map(|s| s.chars().count()).unwrap_or(0);
            let suffix_len = suffix.chars().count();
            let chosen_notation = choice.choose(indent, Some(prefix_len), Some(suffix_len), width);
            let middle_block = Block { chunks: vec![Chunk::Text(suffix)] };
            let (line, mut left_blocks) =
                expand_first_line(prefix, indent, chosen_notation, middle_block, width);
            left_blocks.append(&mut right_blocks);
            (line, left_blocks)
        }
    }
}

impl<'n> Block<'n> {
    fn prepend_text(&mut self, prefix: String) {
        self.chunks.insert(0, Chunk::Text(prefix));
    }

    fn prepend_notation(&mut self, indent: Option<usize>, notation: &'n MeasuredNotation) {
        let chunk = Chunk::Notation { indent, notation };
        self.chunks.insert(0, chunk);
    }

    fn expand_first_line(self, width: usize) -> (String, Vec<Block<'n>>) {
        let mut iter = self.chunks.into_iter();
        let mut prefix = None;
        while let Some(chunk) = iter.next() {
            match chunk {
                Chunk::Text(text) => {
                    prefix = match prefix {
                        None => Some(text),
                        Some(prefix) => Some(prefix + &text),
                    };
                }
                Chunk::Notation { indent, notation } => {
                    let remaining = Block {
                        chunks: iter.collect(),
                    };
                    return expand_first_line(prefix, indent, notation, remaining, width);
                }
            }
        }
        // No notation found, it's just one simple line.
        let line = prefix.unwrap_or_default();
        (line, vec![])
    }

    fn expand_first_lines(self, num_lines: usize, width: usize) -> (Vec<String>, Vec<Block<'n>>) {
        let mut lines = vec![];
        let mut blocks = vec![self];
        for _ in 0..num_lines {
            if blocks.is_empty() {
                return (lines, vec![]);
            }
            let block = blocks.remove(0);
            let (next_line, next_blocks) = block.expand_first_line(width);
            lines.push(next_line);
            blocks = next_blocks.into_iter().chain(blocks.into_iter()).collect();
        }
        (lines, blocks)
    }
}
