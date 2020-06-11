// TODO:
// - functionality: last lines, seeking, multiple expand

use super::measure::MeasuredNotation;
use std::iter::Iterator;
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

pub struct FirstLineIter<'n> {
    blocks: Vec<Block<'n>>,
    width: usize,
}

struct LastLinePrinter<'n> {
    spaces: usize,
    suffix: String,
    chunks: Vec<Chunk<'n>>,
    blocks: Vec<Block<'n>>,
    width: usize,
}

pub struct LastLineIter<'n> {
    blocks: Vec<Block<'n>>,
    width: usize,
}

pub fn partial_pretty_print_first<'n>(
    notation: &'n MeasuredNotation,
    width: usize,
) -> FirstLineIter<'n> {
    let blocks = vec![Block::new(notation)];
    FirstLineIter { blocks, width }
}

pub fn partial_pretty_print_last<'n>(
    notation: &'n MeasuredNotation,
    width: usize,
) -> LastLineIter<'n> {
    let blocks = vec![Block::new(notation)];
    LastLineIter { blocks, width }
}

impl<'n> Iterator for FirstLineIter<'n> {
    type Item = (usize, String);

    fn next(&mut self) -> Option<(usize, String)> {
        if self.blocks.is_empty() {
            return None;
        } else {
            let blocks = mem::take(&mut self.blocks);
            let printer = FirstLinePrinter::new(self.width, blocks);
            let (spaces, line, new_blocks) = printer.print();
            self.blocks = new_blocks;
            Some((spaces, line))
        }
    }
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

    fn print(mut self) -> (usize, String, Vec<Block<'n>>) {
        while let Some(chunk) = self.chunks.pop() {
            match chunk {
                Chunk::Text(text) => self.prefix += &text,
                Chunk::Notation { indent, notation } => self.print_notation(indent, notation),
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

    fn print_notation(&mut self, indent: Option<usize>, notation: &'n MeasuredNotation) {
        use MeasuredNotation::*;
        match notation {
            Empty(_) => (),
            Literal(_, text) => self.prefix += &text,
            Newline(_) => {
                self.blocks.push(Block {
                    spaces: indent.unwrap(),
                    chunks: mem::take(&mut self.chunks),
                });
            }
            Indent(_, inner_indent, inner_notation) => {
                let full_indent = indent.map(|i| i + inner_indent);
                self.push_chunk(full_indent, inner_notation);
            }
            Flat(_, inner_notation) => {
                self.push_chunk(None, inner_notation);
            }
            Align(_, _) => unimplemented!(),
            Concat(_, left, right, _) => {
                self.push_chunk(indent, right);
                self.push_chunk(indent, left);
            }
            Choice(_, choice) => {
                if let Some(chosen_notation) = choice.sole_option(indent.is_none()) {
                    self.push_chunk(indent, chosen_notation);
                    return;
                }
                let suffix_printer = FirstLinePrinter {
                    spaces: 0,
                    prefix: "".to_string(),
                    chunks: mem::take(&mut self.chunks),
                    blocks: mem::take(&mut self.blocks),
                    width: self.width,
                };
                let (suffix_spaces, suffix, blocks) = suffix_printer.print();
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

impl<'n> Iterator for LastLineIter<'n> {
    type Item = (usize, String);

    fn next(&mut self) -> Option<(usize, String)> {
        if self.blocks.is_empty() {
            return None;
        } else {
            let blocks = mem::take(&mut self.blocks);
            let printer = LastLinePrinter::new(self.width, blocks);
            let (spaces, line, new_blocks) = printer.print();
            self.blocks = new_blocks;
            Some((spaces, line))
        }
    }
}

impl<'n> LastLinePrinter<'n> {
    fn new(width: usize, blocks: Vec<Block<'n>>) -> LastLinePrinter<'n> {
        let mut blocks = blocks;
        assert!(blocks.len() >= 1);
        let block = blocks.pop().unwrap();
        LastLinePrinter {
            spaces: block.spaces,
            suffix: "".to_string(),
            chunks: block.chunks,
            blocks,
            width,
        }
    }

    fn push_chunk(&mut self, indent: Option<usize>, notation: &'n MeasuredNotation) {
        self.chunks.push(Chunk::Notation { indent, notation });
    }

    fn push_text(&mut self, text: String) {
        self.chunks.push(Chunk::Text(text));
    }

    fn print(mut self) -> (usize, String, Vec<Block<'n>>) {
        while let Some(chunk) = self.chunks.pop() {
            match chunk {
                Chunk::Text(text) => self.suffix = text.to_owned() + &self.suffix,
                Chunk::Notation { indent, notation } => self.print_notation(indent, notation),
            }
        }
        (self.spaces, self.suffix, self.blocks)
    }

    fn print_notation(&mut self, indent: Option<usize>, notation: &'n MeasuredNotation) {
        use MeasuredNotation::*;
        match notation {
            Empty(_) => (),
            Literal(_, text) => self.suffix = text.to_owned() + &self.suffix,
            Newline(_) => {
                self.blocks.push(Block {
                    spaces: self.spaces,
                    chunks: mem::take(&mut self.chunks),
                });
                self.spaces = indent.unwrap();
            }
            Indent(_, inner_indent, inner_notation) => {
                let full_indent = indent.map(|i| i + inner_indent);
                self.push_chunk(full_indent, inner_notation);
            }
            Flat(_, inner_notation) => {
                self.push_chunk(None, inner_notation);
            }
            Align(_, _) => unimplemented!(),
            Concat(_, left, right, _) => {
                self.push_chunk(indent, left);
                self.push_chunk(indent, right);
            }
            Choice(_, choice) => {
                if let Some(chosen_notation) = choice.sole_option(indent.is_none()) {
                    self.push_chunk(indent, chosen_notation);
                    return;
                }
                let prefix_printer = LastLinePrinter {
                    spaces: self.spaces,
                    suffix: "".to_string(),
                    chunks: mem::take(&mut self.chunks),
                    blocks: mem::take(&mut self.blocks),
                    width: self.width,
                };
                let (prefix_spaces, prefix, blocks) = prefix_printer.print();
                let prefix_len = prefix_spaces + prefix.chars().count();
                let suffix_len = self.suffix.chars().count();
                let chosen_notation =
                    choice.choose(indent, Some(prefix_len), Some(suffix_len), self.width);
                self.spaces = prefix_spaces;
                self.blocks = blocks;
                self.push_text(prefix);
                self.push_chunk(indent, chosen_notation);
            }
        }
    }
}

impl<'n> Block<'n> {
    fn new(notation: &'n MeasuredNotation) -> Block<'n> {
        Block {
            spaces: 0,
            chunks: vec![Chunk::Notation {
                indent: Some(0),
                notation,
            }],
        }
    }
}
