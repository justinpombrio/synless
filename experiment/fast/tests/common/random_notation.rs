use fast::Notation;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use Notation::*;

#[derive(Clone)]
pub struct NotationGeneratorConfig {
    pub max_choices: usize,
    pub size_range: (usize, usize),
    pub literal_range: (usize, usize),
    pub indent_range: (usize, usize),
}

pub struct NotationGenerator {
    rng: StdRng,
    config: NotationGeneratorConfig,
}

impl NotationGenerator {
    pub fn new(seed: u64, config: NotationGeneratorConfig) -> NotationGenerator {
        NotationGenerator {
            rng: StdRng::seed_from_u64(seed),
            config,
        }
    }

    pub fn random_notation(&mut self) -> Notation {
        let size = self
            .rng
            .gen_range(self.config.size_range.0, self.config.size_range.1);
        let mut builder = Builder::new(StdRng::seed_from_u64(self.rng.gen()), &self.config);
        let notation = builder.notation(size);
        self.rng = builder.rng;
        notation
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Option {
    Literal,
    Flat,
    Align,
    Nest,
    Concat,
    Choice,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OptionInfo {
    option: Option,
    arity: usize,
    weight: usize,
}

const OPTIONS: &[OptionInfo] = &[
    OptionInfo {
        option: Option::Literal,
        arity: 0,
        weight: 1,
    },
    OptionInfo {
        option: Option::Flat,
        arity: 1,
        weight: 1,
    },
    OptionInfo {
        option: Option::Align,
        arity: 1,
        weight: 1,
    },
    OptionInfo {
        option: Option::Nest,
        arity: 1,
        weight: 1,
    },
    OptionInfo {
        option: Option::Concat,
        arity: 2,
        weight: 2,
    },
    OptionInfo {
        option: Option::Choice,
        arity: 2,
        weight: 1,
    },
];

struct Builder {
    next_letter: char,
    rng: StdRng,
    num_choices: usize,
    literal_range: (usize, usize),
    indent_range: (usize, usize),
}

impl Builder {
    fn new(rng: StdRng, config: &NotationGeneratorConfig) -> Builder {
        Builder {
            next_letter: 'a',
            rng,
            num_choices: config.max_choices,
            literal_range: config.literal_range,
            indent_range: config.indent_range,
        }
    }

    fn letter(&mut self) -> char {
        let letter = self.next_letter;
        self.next_letter = ((self.next_letter as u8) + 1) as char;
        letter
    }

    fn notation(&mut self, size: usize) -> Notation {
        let options = OPTIONS
            .iter()
            .filter(|opt| (opt.arity == 0 && size == 1) || (opt.arity > 0 && size >= opt.arity + 1))
            .filter(|opt| opt.option != Option::Choice || self.num_choices > 0)
            .collect::<Vec<_>>();
        let total_weight: usize = options.iter().map(|opt| opt.weight).sum();
        let mut selection = self.rng.gen_range(0, total_weight);
        for option in options {
            if selection < option.weight {
                return self.use_option(option.option, size);
            }
            selection -= option.weight;
        }
        unreachable!();
    }

    fn use_option(&mut self, option: Option, size: usize) -> Notation {
        match option {
            Option::Literal => self.literal(),
            Option::Flat => self.flat(size),
            Option::Align => self.align(size),
            Option::Nest => self.nest(size),
            Option::Concat => self.concat(size),
            Option::Choice => self.choice(size),
        }
    }

    fn literal(&mut self) -> Notation {
        let letter = self.letter();
        let len = self
            .rng
            .gen_range(self.literal_range.0, self.literal_range.1);
        let string = (0..len).map(|_| letter).collect();
        Literal(string)
    }

    fn flat(&mut self, size: usize) -> Notation {
        Flat(Box::new(self.notation(size - 1)))
    }

    fn align(&mut self, size: usize) -> Notation {
        Align(Box::new(self.notation(size - 1)))
    }

    fn nest(&mut self, size: usize) -> Notation {
        let indent = self.rng.gen_range(self.indent_range.0, self.indent_range.1);
        Nest(indent, Box::new(self.notation(size - 1)))
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
