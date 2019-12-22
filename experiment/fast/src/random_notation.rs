use super::Notation;
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
    fn new(seed: u64) -> NotationGenerator {
        NotationGenerator {
            rng: StdRng::seed_from_u64(seed),
        }
    }

    fn random_notation(&mut self) -> Notation {
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

#[cfg(test)]
mod tests {
    use super::NotationGenerator;
    use crate::notation::Notation;
    use crate::oracular_pretty_print::oracular_pretty_print;
    use crate::pretty_print::pretty_print;

    enum PPResult {
        Ok,
        Invalid,
        Error(PPError),
    }

    struct PPError {
        notation: Notation,
        width: usize,
        actual: Vec<String>,
        oracular: Vec<String>,
    }

    fn expand_line(indent: usize, line: String) -> String {
        format!("{:indent$}{}", "", line, indent = indent)
    }

    fn expand_lines(lines: Vec<(usize, String)>) -> Vec<String> {
        lines.into_iter().map(|(i, s)| expand_line(i, s)).collect()
    }

    fn try_pretty_print(notation: Notation) -> PPResult {
        let valid_notation = match notation.clone().validate() {
            Ok(valid) => valid,
            Err(_) => return PPResult::Invalid,
        };
        let measured_notation = valid_notation.measure();
        for width in WIDTH_RANGE.0..WIDTH_RANGE.1 {
            let oracle_lines = oracular_pretty_print(&notation, width);
            let actual_lines = pretty_print(&measured_notation, width);
            if actual_lines != oracle_lines {
                return PPResult::Error(PPError {
                    notation,
                    width,
                    actual: expand_lines(actual_lines),
                    oracular: expand_lines(oracle_lines),
                });
            }
        }
        PPResult::Ok
    }

    const WIDTH_RANGE: (usize, usize) = (5, 20);
    const NUM_TESTS: usize = 10000;
    const SEED: u64 = 20;

    #[test]
    fn oracle_tests() {
        let mut first_error = None;
        let mut num_invalid = 0;
        let mut num_errors = 0;
        let mut generator = NotationGenerator::new(SEED);
        for _ in 0..NUM_TESTS {
            let note = generator.random_notation();
            match try_pretty_print(note) {
                PPResult::Ok => (),
                PPResult::Invalid => {
                    num_invalid += 1;
                }
                PPResult::Error(error) => {
                    // WLOG this is first.
                    first_error = Some(error);
                    num_errors += 1;
                }
            }
        }
        eprintln!(
            "Tested {} notations. {} were invalid. {} were printed incorrectly.",
            NUM_TESTS, num_invalid, num_errors
        );
        if let Some(error) = first_error {
            eprintln!(
                "PRETTY PRINTER PRODUCED:\n{}\n\nBUT ORACLE SAYS IT SHOULD BE:\n{}\n\nNOTATION:\n{:#?}\nWIDTH:{}",
                error.actual.join("\n"),
                error.oracular.join("\n"),
                error.notation,
//                error.notation.validate().unwrap().measure(),
                error.width,
            );
            assert!(false);
        }
    }
}
