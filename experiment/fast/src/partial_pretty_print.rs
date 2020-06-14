// TODO:
// - functionality: seeking, multiple expand

use super::measure::{MeasuredNotation, Pos, Span};
use std::iter::Iterator;
use std::mem;

#[derive(Debug, Clone)]
struct Block<'n> {
    spaces: usize,
    chunks: Vec<Chunk<'n>>,
}

#[derive(Debug, Clone)]
enum Chunk<'n> {
    Text(String),
    Notation {
        indent: Option<usize>, // None means flat
        notation: &'n MeasuredNotation,
    },
}

#[derive(Debug, Clone)]
pub struct ForwardPrinter<'n> {
    // Persistent:
    width: usize,
    prefix: String,
    blocks: Vec<Block<'n>>,
    // Temporary:
    spaces: usize,
    chunks: Vec<Chunk<'n>>,
}

#[derive(Debug, Clone)]
pub struct BackwardPrinter<'n> {
    // Persistent:
    width: usize,
    suffix: String,
    blocks: Vec<Block<'n>>,
    // Temporary:
    spaces: usize,
    chunks: Vec<Chunk<'n>>,
}

#[derive(Debug, Clone)]
pub struct PartialPrettyPrinter<'n> {
    width: usize,
    spaces: usize,
    prev_blocks: Vec<Block<'n>>,
    next_blocks: Vec<Block<'n>>,
    prev_chunks: Vec<Chunk<'n>>,
    next_chunks: Vec<Chunk<'n>>,
}

pub fn partial_pretty_print<'n>(
    notation: &'n MeasuredNotation,
    width: usize,
    pos: Pos,
) -> (BackwardPrinter<'n>, ForwardPrinter<'n>) {
    let mut ppp = PartialPrettyPrinter::new(notation, width, pos);
    ppp.print()
}

pub fn partial_pretty_print_first<'n>(
    notation: &'n MeasuredNotation,
    width: usize,
) -> ForwardPrinter<'n> {
    let blocks = vec![Block::new(notation)];
    ForwardPrinter::new(width, blocks)
}

pub fn partial_pretty_print_last<'n>(
    notation: &'n MeasuredNotation,
    width: usize,
) -> BackwardPrinter<'n> {
    let blocks = vec![Block::new(notation)];
    BackwardPrinter::new(width, blocks)
}

impl<'n> PartialPrettyPrinter<'n> {
    pub fn new(notation: &'n MeasuredNotation, width: usize, pos: Pos) -> PartialPrettyPrinter<'n> {
        let mut ppp = PartialPrettyPrinter {
            width,
            spaces: 0,
            prev_blocks: vec![],
            next_blocks: vec![],
            prev_chunks: vec![],
            next_chunks: vec![],
        };
        ppp.seek(pos, notation, Some(0));
        ppp
    }

    fn seek(&mut self, sought: Pos, notation: &'n MeasuredNotation, indent: Option<usize>) {
        use MeasuredNotation::*;
        if sought <= notation.span().start {
            self.next_chunks.push(Chunk::Notation { indent, notation });
            return;
        } else if sought >= notation.span().end {
            self.next_chunks.push(Chunk::Notation { indent, notation });
            return;
        }
        match notation {
            Empty(_) | Literal(_, _) | Newline(_) => unreachable!(), // pos, not span
            Indent(_, i, inner_notation) => {
                self.seek(sought, inner_notation, indent.map(|j| j + i));
            }
            Flat(_, inner_notation) => {
                self.seek(sought, inner_notation, None);
            }
            Align(_, inner_notation) => unimplemented!(),
            Concat(_, left, right, _) => {
                if sought <= right.span().start {
                    self.next_chunks.push(Chunk::Notation {
                        indent,
                        notation: right,
                    });
                    self.seek(sought, left, indent);
                } else {
                    self.prev_chunks.push(Chunk::Notation {
                        indent,
                        notation: right,
                    });
                    self.seek(sought, right, indent);
                }
            }
            Choice(_, choice) => {
                if let Some(chosen_notation) = choice.sole_option(indent.is_none()) {
                    self.seek(sought, chosen_notation, indent);
                    return;
                }

                self.move_chunks_to_blocks();

                // Compute prefix
                let mut prefix_printer =
                    BackwardPrinter::new(self.width, mem::take(&mut self.prev_blocks));
                let (prefix_spaces, prefix) = prefix_printer.next().unwrap();
                self.prev_blocks = prefix_printer.blocks();
                let prefix_len = prefix_spaces + prefix.chars().count();
                self.spaces = prefix_spaces;
                self.prev_chunks.push(Chunk::Text(prefix));

                // Compute suffix
                let mut suffix_printer =
                    ForwardPrinter::new(self.width, mem::take(&mut self.next_blocks));
                let (suffix_spaces, suffix) = suffix_printer.next().unwrap();
                self.next_blocks = suffix_printer.blocks();
                assert_eq!(suffix_spaces, 0);
                let suffix_len = suffix.chars().count();
                self.next_chunks.push(Chunk::Text(suffix));

                // Choose a notation
                let chosen_notation =
                    choice.choose(indent, Some(prefix_len), Some(suffix_len), self.width);
                self.seek(sought, chosen_notation, indent);
            }
        }
    }

    fn move_chunks_to_blocks(&mut self) {
        self.prev_blocks.push(Block {
            spaces: self.spaces,
            chunks: mem::take(&mut self.prev_chunks),
        });
        self.next_blocks.push(Block {
            spaces: 0,
            chunks: mem::take(&mut self.next_chunks),
        });
    }

    fn print(mut self) -> (BackwardPrinter<'n>, ForwardPrinter<'n>) {
        self.move_chunks_to_blocks();
        let bpp = BackwardPrinter::new(self.width, mem::take(&mut self.prev_blocks));
        let fpp = ForwardPrinter::new(self.width, mem::take(&mut self.next_blocks));
        (bpp, fpp)
    }
}

impl<'n> Iterator for ForwardPrinter<'n> {
    type Item = (usize, String);

    fn next(&mut self) -> Option<(usize, String)> {
        if let Some(block) = self.blocks.pop() {
            self.spaces = block.spaces;
            self.chunks = block.chunks;
            while let Some(chunk) = self.chunks.pop() {
                match chunk {
                    Chunk::Text(text) => self.prefix += &text,
                    Chunk::Notation { indent, notation } => self.print_notation(indent, notation),
                }
            }
            Some((self.spaces, self.prefix.split_off(0)))
        } else {
            None
        }
    }
}

impl<'n> ForwardPrinter<'n> {
    fn new(width: usize, blocks: Vec<Block<'n>>) -> ForwardPrinter<'n> {
        ForwardPrinter {
            width,
            blocks,
            prefix: "".to_string(),
            spaces: 0,
            chunks: vec![],
        }
    }

    fn blocks(self) -> Vec<Block<'n>> {
        self.blocks
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
                self.blocks.push(Block {
                    spaces: 0,
                    chunks: mem::take(&mut self.chunks),
                });
                let mut suffix_printer =
                    ForwardPrinter::new(self.width, mem::take(&mut self.blocks));
                let (suffix_spaces, suffix) = suffix_printer.next().unwrap();
                let blocks = suffix_printer.blocks();
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

impl<'n> Iterator for BackwardPrinter<'n> {
    type Item = (usize, String);

    fn next(&mut self) -> Option<(usize, String)> {
        if let Some(block) = self.blocks.pop() {
            self.spaces = block.spaces;
            self.chunks = block.chunks;
            while let Some(chunk) = self.chunks.pop() {
                match chunk {
                    Chunk::Text(text) => self.suffix = text.to_owned() + &self.suffix,
                    Chunk::Notation { indent, notation } => self.print_notation(indent, notation),
                }
            }
            Some((self.spaces, self.suffix.split_off(0)))
        } else {
            None
        }
    }
}

impl<'n> BackwardPrinter<'n> {
    fn new(width: usize, blocks: Vec<Block<'n>>) -> BackwardPrinter<'n> {
        BackwardPrinter {
            width,
            blocks,
            suffix: "".to_string(),
            spaces: 0,
            chunks: vec![],
        }
    }

    fn push_chunk(&mut self, indent: Option<usize>, notation: &'n MeasuredNotation) {
        self.chunks.push(Chunk::Notation { indent, notation });
    }

    fn push_text(&mut self, text: String) {
        self.chunks.push(Chunk::Text(text));
    }

    fn blocks(self) -> Vec<Block<'n>> {
        self.blocks
    }

    fn print_notation(&mut self, indent: Option<usize>, notation: &'n MeasuredNotation) {
        use MeasuredNotation::*;
        match notation {
            Empty(_) => (),
            Literal(_, text) => self.suffix = text.to_owned() + &self.suffix,
            Newline(pos) => {
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
                self.blocks.push(Block {
                    spaces: self.spaces,
                    chunks: mem::take(&mut self.chunks),
                });
                let mut prefix_printer =
                    BackwardPrinter::new(self.width, mem::take(&mut self.blocks));
                let (prefix_spaces, prefix) = prefix_printer.next().unwrap();
                let blocks = prefix_printer.blocks();
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
