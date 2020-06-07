// TODO:
// - functionality: last lines, seeking, multiple expand

use super::measure::MeasuredNotation;
use std::mem;

struct Block<'n> {
    spaces: usize,
    chunks: Vec<Chunk<'n>>,
}

enum Chunk<'n> {
    Text(String),
    Notation {
        indent: Option<usize>, // None means flat
        notation: &'n MeasuredNotation,
    },
}

struct FirstLinePrinter<'n> {
    spaces: usize,
    prefix: String,
    chunks: Vec<Chunk<'n>>,
    blocks: Vec<Block<'n>>,
    width: usize,
}

pub fn partial_pretty_print_first<'n>(
    notation: &'n MeasuredNotation,
    num_lines: usize,
    width: usize,
) -> Vec<(usize, String)> {
    let mut blocks = vec![Block {
        spaces: 0,
        chunks: vec![Chunk::Notation {
            indent: Some(0),
            notation,
        }],
    }];
    let mut lines = vec![];
    for _ in 0..num_lines {
        if blocks.is_empty() {
            break;
        }
        let printer = FirstLinePrinter::new(width, blocks);
        let (spaces, line, new_blocks) = printer.expand();
        lines.push((spaces, line));
        blocks = new_blocks;
    }
    lines
}

impl<'n> FirstLinePrinter<'n> {
    fn new(width: usize, blocks: Vec<Block<'n>>) -> FirstLinePrinter<'n> {
        let mut blocks = blocks;
        assert!(blocks.len() >= 1);
        let block = blocks.pop().unwrap();
        FirstLinePrinter {
            spaces: block.spaces,
            prefix: "".to_string(),
            chunks: block.chunks,
            blocks,
            width,
        }
    }

    fn expand(mut self) -> (usize, String, Vec<Block<'n>>) {
        while let Some(chunk) = self.chunks.pop() {
            match chunk {
                Chunk::Text(text) => self.prefix += &text,
                Chunk::Notation { indent, notation } => self.expand_notation(indent, notation),
            }
        }
        (self.spaces, self.prefix, self.blocks)
    }

    fn push_chunk(&mut self, indent: Option<usize>, notation: &'n MeasuredNotation) {
        self.chunks.push(Chunk::Notation { indent, notation });
    }

    fn push_text(&mut self, text: String) {
        self.chunks.push(Chunk::Text(text));
    }

    fn expand_notation(&mut self, indent: Option<usize>, notation: &'n MeasuredNotation) {
        use MeasuredNotation::*;
        match notation {
            Empty => (),
            Literal(text) => self.prefix += &text,
            Newline => {
                self.blocks.push(Block {
                    spaces: indent.unwrap(),
                    chunks: mem::replace(&mut self.chunks, vec![]),
                });
            }
            Indent(inner_indent, inner_notation) => {
                let full_indent = indent.map(|i| i + inner_indent);
                self.push_chunk(full_indent, inner_notation);
            }
            Flat(inner_notation) => {
                self.push_chunk(None, inner_notation);
            }
            Align(_) => unimplemented!(),
            Concat(left, right, _) => {
                self.push_chunk(indent, right);
                self.push_chunk(indent, left);
            }
            Choice(choice) => {
                if let Some(chosen_notation) = choice.sole_option(indent.is_none()) {
                    self.push_chunk(indent, chosen_notation);
                    return;
                }
                let suffix_printer = FirstLinePrinter {
                    spaces: 0,
                    prefix: "".to_string(),
                    chunks: mem::replace(&mut self.chunks, vec![]),
                    blocks: mem::replace(&mut self.blocks, vec![]),
                    width: self.width,
                };
                let (suffix_spaces, suffix, blocks) = suffix_printer.expand();
                assert_eq!(suffix_spaces, 0);
                let prefix_len = self.spaces + self.prefix.chars().count();
                let suffix_len = suffix.chars().count();
                let chosen_notation =
                    choice.choose(indent, Some(prefix_len), Some(suffix_len), self.width);
                self.blocks = blocks;
                self.push_text(suffix);
                self.push_chunk(indent, chosen_notation);
            }
        }
    }
}
