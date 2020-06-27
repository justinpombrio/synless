// TODO:
// - functionality: seeking, multiple expand

use super::measure::{MeasuredNotation, Pos};
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
    width: usize,
    blocks: Vec<Block<'n>>,
}

#[derive(Debug)]
pub struct NextLinePrinter<'b, 'n> {
    width: usize,
    spaces: usize,
    prefix: String,
    chunks: &'b mut Vec<Chunk<'n>>,
    blocks: &'b mut Vec<Block<'n>>,
}

#[derive(Debug, Clone)]
pub struct BackwardPrinter<'n> {
    width: usize,
    blocks: Vec<Block<'n>>,
}

#[derive(Debug)]
pub struct PrevLinePrinter<'b, 'n> {
    width: usize,
    spaces: usize,
    suffix: String,
    chunks: &'b mut Vec<Chunk<'n>>,
    blocks: &'b mut Vec<Block<'n>>,
}

#[derive(Debug, Clone)]
pub struct PartialPrettyPrinter<'n> {
    width: usize,
    spaces: usize, // TODO: needed?
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
    let mut ppp = PartialPrettyPrinter {
        width,
        spaces: 0,
        prev_blocks: vec![],
        next_blocks: vec![],
        prev_chunks: vec![],
        next_chunks: vec![],
    };
    ppp.seek(pos, notation, Some(0));
    ppp.print()
}

pub fn partial_pretty_print_first<'n>(
    notation: &'n MeasuredNotation,
    width: usize,
) -> ForwardPrinter<'n> {
    let blocks = vec![Block::new(notation)];
    ForwardPrinter { width, blocks }
}

pub fn partial_pretty_print_last<'n>(
    notation: &'n MeasuredNotation,
    width: usize,
) -> BackwardPrinter<'n> {
    let blocks = vec![Block::new(notation)];
    BackwardPrinter { width, blocks }
}

impl<'n> PartialPrettyPrinter<'n> {
    fn seek(&mut self, sought: Pos, notation: &'n MeasuredNotation, indent: Option<usize>) {
        use MeasuredNotation::*;
        if sought <= notation.span().start {
            self.next_chunks.push(Chunk::Notation { indent, notation });
            return;
        } else if sought >= notation.span().end {
            self.prev_chunks.push(Chunk::Notation { indent, notation });
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
            Align(_, _inner_notation) => unimplemented!(),
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
                        notation: left,
                    });
                    self.seek(sought, right, indent);
                }
            }
            Choice(_, choice) => {
                if let Some(chosen_notation) = choice.sole_option(indent.is_none()) {
                    self.seek(sought, chosen_notation, indent);
                    return;
                }

                // Compute prefix
                let prefix_printer = PrevLinePrinter {
                    width: self.width,
                    spaces: self.spaces,
                    suffix: "".to_string(),
                    chunks: &mut self.prev_chunks,
                    blocks: &mut self.prev_blocks,
                };
                let (prefix_spaces, prefix) = prefix_printer.print();
                let prefix_len = prefix_spaces + prefix.chars().count();
                self.spaces = prefix_spaces;
                self.prev_chunks.push(Chunk::Text(prefix));

                // Compute suffix
                let suffix_printer = NextLinePrinter {
                    width: self.width,
                    spaces: 0,
                    prefix: "".to_string(),
                    chunks: &mut self.next_chunks,
                    blocks: &mut self.next_blocks,
                };
                let (suffix_spaces, suffix) = suffix_printer.print();
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

    fn print(mut self) -> (BackwardPrinter<'n>, ForwardPrinter<'n>) {
        let mut chunks = mem::take(&mut self.next_chunks);
        self.prev_chunks.reverse();
        chunks.append(&mut self.prev_chunks);
        self.next_blocks.push(Block {
            spaces: self.spaces,
            chunks,
        });
        let bpp = BackwardPrinter {
            width: self.width,
            blocks: self.prev_blocks,
        };
        let fpp = ForwardPrinter {
            width: self.width,
            blocks: self.next_blocks,
        };
        (bpp, fpp)
    }
}

impl<'n> Iterator for ForwardPrinter<'n> {
    type Item = (usize, String);

    fn next(&mut self) -> Option<(usize, String)> {
        if let Some(mut block) = self.blocks.pop() {
            let next_line_printer = NextLinePrinter {
                width: self.width,
                spaces: block.spaces,
                prefix: "".to_string(),
                chunks: &mut block.chunks,
                blocks: &mut self.blocks,
            };
            Some(next_line_printer.print())
        } else {
            None
        }
    }
}

impl<'b, 'n> NextLinePrinter<'b, 'n> {
    fn print(mut self) -> (usize, String) {
        while let Some(chunk) = self.chunks.pop() {
            match chunk {
                Chunk::Text(text) => self.prefix += &text,
                Chunk::Notation { indent, notation } => self.print_notation(indent, notation),
            }
        }
        (self.spaces, self.prefix)
    }

    fn print_notation(&mut self, indent: Option<usize>, notation: &'n MeasuredNotation) {
        use MeasuredNotation::*;
        match notation {
            Empty(_) => (),
            Literal(_, text) => self.prefix += &text,
            Newline(_) => self.newline(indent),
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
                let suffix_printer = NextLinePrinter {
                    width: self.width,
                    spaces: 0,
                    prefix: "".to_string(),
                    chunks: &mut self.chunks,
                    blocks: &mut self.blocks,
                };
                let (suffix_spaces, suffix) = suffix_printer.print();
                assert_eq!(suffix_spaces, 0);
                let prefix_len = self.spaces + self.prefix.chars().count();
                let suffix_len = suffix.chars().count();
                let chosen_notation =
                    choice.choose(indent, Some(prefix_len), Some(suffix_len), self.width);
                self.push_text(suffix);
                self.push_chunk(indent, chosen_notation);
            }
        }
    }

    fn push_chunk(&mut self, indent: Option<usize>, notation: &'n MeasuredNotation) {
        self.chunks.push(Chunk::Notation { indent, notation });
    }

    fn push_text(&mut self, text: String) {
        self.chunks.push(Chunk::Text(text));
    }

    fn newline(&mut self, indent: Option<usize>) {
        self.blocks.push(Block {
            spaces: indent.unwrap(),
            chunks: mem::take(&mut self.chunks),
        });
    }
}

impl<'n> Iterator for BackwardPrinter<'n> {
    type Item = (usize, String);

    fn next(&mut self) -> Option<(usize, String)> {
        if let Some(mut block) = self.blocks.pop() {
            let prev_line_printer = PrevLinePrinter {
                width: self.width,
                spaces: block.spaces,
                suffix: "".to_string(),
                chunks: &mut block.chunks,
                blocks: &mut self.blocks,
            };
            Some(prev_line_printer.print())
        } else {
            None
        }
    }
}

impl<'b, 'n> PrevLinePrinter<'b, 'n> {
    fn print(mut self) -> (usize, String) {
        while let Some(chunk) = self.chunks.pop() {
            match chunk {
                Chunk::Text(text) => self.suffix = text + &self.suffix,
                Chunk::Notation { indent, notation } => self.print_notation(indent, notation),
            }
        }
        (self.spaces, self.suffix)
    }

    fn print_notation(&mut self, indent: Option<usize>, notation: &'n MeasuredNotation) {
        use MeasuredNotation::*;
        match notation {
            Empty(_) => (),
            Literal(_, text) => self.suffix = text.to_string() + &self.suffix,
            Newline(_) => self.newline(indent),
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
                let prefix_printer = PrevLinePrinter {
                    width: self.width,
                    spaces: self.spaces,
                    suffix: "".to_string(),
                    chunks: &mut self.chunks,
                    blocks: &mut self.blocks,
                };
                let (prefix_spaces, prefix) = prefix_printer.print();
                let prefix_len = prefix_spaces + prefix.chars().count();
                let suffix_len = self.suffix.chars().count();
                let chosen_notation =
                    choice.choose(indent, Some(prefix_len), Some(suffix_len), self.width);
                self.spaces = prefix_spaces;
                self.push_text(prefix);
                self.push_chunk(indent, chosen_notation);
            }
        }
    }

    fn push_chunk(&mut self, indent: Option<usize>, notation: &'n MeasuredNotation) {
        self.chunks.push(Chunk::Notation { indent, notation });
    }

    fn push_text(&mut self, text: String) {
        self.chunks.push(Chunk::Text(text));
    }

    fn newline(&mut self, indent: Option<usize>) {
        self.blocks.push(Block {
            spaces: self.spaces,
            chunks: mem::take(&mut self.chunks),
        });
        self.spaces = indent.unwrap();
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
