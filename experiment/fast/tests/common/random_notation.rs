use fast::Notation;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use Notation::*;

struct Builder {
    next_letter: char,
    rng: StdRng,
    num_choices: usize,
}

const MAX_CHOICES: usize = 6;
const SIZE_RANGE: (usize, usize) = (6, 7);
const LITERAL_RANGE: (usize, usize) = (0, 10);
const INDENT_RANGE: (usize, usize) = (0, 10);

pub struct NotationGenerator {
    rng: StdRng,
}

impl NotationGenerator {
    pub fn new(seed: u64) -> NotationGenerator {
        NotationGenerator {
            rng: StdRng::seed_from_u64(seed),
        }
    }

    pub fn random_notation(&mut self) -> Notation {
        let size = self.rng.gen_range(SIZE_RANGE.0, SIZE_RANGE.1);
        let mut builder = Builder::new(StdRng::seed_from_u64(self.rng.gen()));
        let notation = builder.notation(size);
        self.rng = builder.rng;
        notation
    }
}

impl Builder {
    fn new(rng: StdRng) -> Builder {
        Builder {
            next_letter: 'a',
            rng,
            num_choices: MAX_CHOICES,
        }
    }

    fn letter(&mut self) -> char {
        let letter = self.next_letter;
        self.next_letter = ((self.next_letter as u8) + 1) as char;
        letter
    }

    fn notation(&mut self, size: usize) -> Notation {
        match size {
            0 => panic!("Random notation: unexpected size 0"),
            1 => {
                if self.rng.gen() {
                    self.literal()
                } else {
                    Newline
                }
            }
            2 => match self.rng.gen_range(0, 3) {
                0 => self.indent(size),
                1 => self.flat(size),
                2 => self.align(size),
                _ => unreachable!(),
            },
            _ => {
                if self.num_choices > 0 {
                    match self.rng.gen_range(0, 7) {
                        0 => self.indent(size),
                        1 => self.flat(size),
                        2 => self.align(size),
                        3 | 4 => self.concat(size),
                        5 | 6 => self.choice(size),
                        _ => unreachable!(),
                    }
                } else {
                    match self.rng.gen_range(0, 5) {
                        0 => self.indent(size),
                        1 => self.flat(size),
                        2 => self.align(size),
                        3 | 4 => self.concat(size),
                        _ => unreachable!(),
                    }
                }
            }
        }
    }

    fn literal(&mut self) -> Notation {
        let letter = self.letter();
        let len = self.rng.gen_range(LITERAL_RANGE.0, LITERAL_RANGE.1);
        let string = (0..len).map(|_| letter).collect();
        Literal(string)
    }

    fn indent(&mut self, size: usize) -> Notation {
        let indent = self.rng.gen_range(INDENT_RANGE.0, INDENT_RANGE.1);
        Indent(indent, Box::new(self.notation(size - 1)))
    }

    fn flat(&mut self, size: usize) -> Notation {
        Flat(Box::new(self.notation(size - 1)))
    }

    fn align(&mut self, size: usize) -> Notation {
        Align(Box::new(self.notation(size - 1)))
    }

    fn concat(&mut self, size: usize) -> Notation {
        let size = size - 1;
        let left_size = self.rng.gen_range(1, size);
        let right_size = size - left_size;
        Concat(
            Box::new(self.notation(left_size)),
            Box::new(self.notation(right_size)),
        )
    }

    fn choice(&mut self, size: usize) -> Notation {
        let size = size - 1;
        self.num_choices -= 1;
        let left_size = self.rng.gen_range(1, size);
        let right_size = size - left_size;
        Choice(
            Box::new(self.notation(left_size)),
            Box::new(self.notation(right_size)),
        )
    }
}
