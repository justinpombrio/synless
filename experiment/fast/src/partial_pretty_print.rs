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
            let (suffix, mut blocks) = remaining.expand_first_line(width);
            let prefix_len = prefix.as_ref().map(|s| s.chars().count()).unwrap_or(0);
            let suffix_len = suffix.chars().count();
            let mut middle_block = Block { chunks: vec![] };
            /*
            let mut first_block = if blocks.is_empty() {
                Block { chunks: vec![] }
            } else {
                blocks.remove(0)
            };
            */
            middle_block.prepend_text(suffix);
            let chosen_notation = choice.choose(indent, Some(prefix_len), Some(suffix_len), width);
            let (line, mut left_blocks) =
                expand_first_line(prefix, indent, chosen_notation, middle_block, width);
            left_blocks.append(&mut blocks);
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

/*
enum Block<'n> {
    Line {
        indent: usize,
        text: String,
    },
    Partial {
        notation: &'n MeasuredNotation,
        indent: usize,
        prefix: String,
        suffix: String,
    },
    ChoosyLeft {
        left: &'n MeasuredNotation,
        right: &'n MeasuredNotation,
        indent: usize,
        prefix: String,
        suffix: String,
    },
    ChoosyRight {
        left: &'n MeasuredNotation,
        right: &'n MeasuredNotation,
        indent: usize,
        prefix: String,
        suffix: String,
    },
}

impl<'n> Block<'n> {
    fn line(indent: usize, text: String) -> Block {
        Block { indent, text }
    }

    fn expect_line(self) -> (usize, String) {
        match self {
            Line { indent, text } => (indent, text),
            _ => panic!("Block::expect_line - wasn't a line"),
        }
    }
}

struct PartialPrettyPrinter {
    width: usize,
}

#[derive(Clone, Copy, Debug)]
enum Direction {
    FirstLine
    LastLine,
    Flat,
}

impl PartialPrettyPrinter {
    fn expand_flat<'n>(
        &self,
        notation: &'n MeasuredNotation,
        prefix: String,
        suffix: String,
    ) -> (usize, String) {
        let lines = self.expand(notation, 0, prefix, suffix, Direction::Flat);
        assert_eq!(lines.len(), 1);
        lines.pop().unwrap().expect_line()
    }

    fn expand_first_line<'n>(
        &self,
        notation: &'n MeasuredNotation,
        indent: usize,
        prefix: String,
        suffix: String,
    ) -> ((usize, String), Vec<Block<'n>>) {
        let lines = self.expand(notation, indent, prefix, suffix, Direction::FirstLine);
        let (i, line) = lines.remove(0).expect_line();
        ((i, line), lines)
    }

    fn expand_last_line<'n>(
        &self,
        notation: &'n MeasuredNotation,
        indent: usize,
        prefix: String,
        suffix: String,
    ) -> (Vec<Block<'n>>, (usize, String)) {
        let lines = self.expand(notation, indent, prefix, suffix, Direction::LastLine);
        let (i, line) = lines.pop().expect_line();
        (lines, (i, line))
    }

    fn expand<'n>(
        &self,
        notation: &'n MeasuredNotation,
        indent: usize,
        prefix: String,
        suffix: String,
        direction: Direction,
    ) -> Vec<Block<'n>> {
        match notation {
            Empty => vec![Block::line(0, prefix + suffix)],
            Literal(text) => vec![Block::line(0, prefix + text + suffix)],
            Newline => {
                assert!(direction != Direction::Flat, "newline in flat (ppp)");
                vec![Block::line(0, prefix), Block::line(indent, suffix)]
            }
            Indent(inner_indent, notation) =>
                self.expand(notation, indent + inner_indent, prefix, suffix, direction),
            Flat(notation) =>
                self.expand(notation, 0, prefix, suffix, Direction::Flat),
            Align(_) => unimplemented!("ppp:align"),
            Concat(left, right, info) => {
                match expand_last_line(right, indent, "".to_string(), suffix) {}
            }
            Choice(choice) => {
                let indent_arg = match direction {
                    Direction::Flat => None,
                    _ => Some(indent),
                };
                let prefix_len = Some(prefix.chars().count());
                let suffix_len = Some(suffix.chars().count());
                let notation = choice.choose(indent_arg, prefix_len, suffix_len, self.width);
                self.expand(notation, indent, prefix, suffix)
            }
        }
    }

    fn expand_block<'n>(
        &self,
        block: Block<'n>,
        direction: Direction,
    ) -> Vec<Block<'n>> {
        match block {
            Line { .. } => vec![block],
            Partial {
                notation,
                indent,
                prefix,
                suffix,
            } => self.expand(notation, indent, prefix, suffix),
            ChoosyLeft {
                left,
                right,
                indent,
                prefix,
                suffix,
            } => {
                match direction {
                    Flat => {
                        let (_, right) = self.expand(right, 0, "".to_string(), suffix, Direction::Flat).pop().unwrap().expect_line();
                        let (i, left) = self.expand(left, 0, prefix, right, Direction::Flat).pop().unwrap().expect_line();

                    }
                }
                let right_lines = self.expand(right, indent, "".to_string(), suffix, Direction::FirstLine);
                let (_, middle_line) = right_lines.remove(0).expect_line();
                let left_lines = self.expand
            }
        }
    }
}

fn expand_last_line_of_block<'n>(block: Block<'n>) -> Vec<Block<'n>> {
    use MeasuredNotation::*;

    match block {
        Line { .. } => vec![block],
        Partial {
            indent,
            prefix,
            suffix,
            notation,
        } => expand_last_line(indent, prefix, suffix, notation),
    }
}

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
*/
