use super::Notation;
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};
use Notation::*;

struct Generator {
    next_letter: char,
    rng: ThreadRng,
    num_choices: usize,
}

const MAX_CHOICES: usize = 6;
const SIZE_RANGE: (usize, usize) = (1, 20);
const LITERAL_RANGE: (usize, usize) = (0, 10);
const INDENT_RANGE: (usize, usize) = (0, 10);

pub fn random_notation() -> Notation {
    let size = thread_rng().gen_range(SIZE_RANGE.0, SIZE_RANGE.1);
    Generator::new().notation(size)
}

impl Generator {
    fn new() -> Generator {
        Generator {
            next_letter: 'a',
            rng: thread_rng(),
            num_choices: MAX_CHOICES,
        }
    }

    fn letter(&mut self) -> char {
        let letter = self.next_letter;
        self.next_letter = ((self.next_letter as u8) + 1) as char;
        letter
    }

    fn notation(&mut self, size: usize) -> Notation {
        let p: f32 = self.rng.gen();
        match size {
            0 => panic!("Random notation: unexpected size 0"),
            1 => match self.rng.gen_range(0, 7) {
                0 => self.indent(size),
                1 => self.flat(size),
                2 => self.align(size),
                3 | 4 => self.literal(),
                5 | 6 => Newline,
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
        Indent(indent, Box::new(self.notation(size)))
    }

    fn flat(&mut self, size: usize) -> Notation {
        Flat(Box::new(self.notation(size)))
    }

    fn align(&mut self, size: usize) -> Notation {
        Align(Box::new(self.notation(size)))
    }

    fn concat(&mut self, size: usize) -> Notation {
        let left_size = self.rng.gen_range(1, size);
        let right_size = size - left_size;
        Concat(
            Box::new(self.notation(left_size)),
            Box::new(self.notation(right_size)),
        )
    }

    fn choice(&mut self, size: usize) -> Notation {
        self.num_choices -= 1;
        let left_size = self.rng.gen_range(1, size);
        let right_size = size - left_size;
        Choice(
            Box::new(self.notation(left_size)),
            Box::new(self.notation(right_size)),
        )
    }
}
