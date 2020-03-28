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
enum Variant {
    Newline,
    Literal,
    Flat,
    Align,
    Indent,
    Concat,
    Choice,
}

impl Variant {
    fn arity(self) -> usize {
        use Variant::*;
        match self {
            Newline | Literal => 0,
            Flat | Align | Indent => 1,
            Concat | Choice => 2,
        }
    }

    fn weight(self) -> usize {
        use Variant::*;
        match self {
            Newline | Literal | Flat | Align | Indent | Choice => 1,
            Concat => 2,
        }
    }
}

const VARIANTS: &[Variant] = &[
    Variant::Newline,
    Variant::Literal,
    Variant::Flat,
    Variant::Align,
    Variant::Indent,
    Variant::Concat,
    Variant::Choice,
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
        let variants = VARIANTS
            .iter()
            .copied()
            .filter(|opt| {
                (opt.arity() == 0 && size == 1) || (opt.arity() > 0 && size >= opt.arity() + 1)
            })
            .filter(|&opt| opt != Variant::Choice || self.num_choices > 0)
            .collect::<Vec<_>>();
        let total_weight: usize = variants.iter().map(|opt| opt.weight()).sum();
        let mut selection = self.rng.gen_range(0, total_weight);
        for variant in variants {
            if selection < variant.weight() {
                return self.use_variant(variant, size);
            }
            selection -= variant.weight();
        }
        unreachable!();
    }

    fn use_variant(&mut self, variant: Variant, size: usize) -> Notation {
        match variant {
            Variant::Literal => self.literal(),
            Variant::Newline => self.newline(),
            Variant::Flat => self.flat(size),
            Variant::Align => self.align(size),
            Variant::Indent => self.indent(size),
            Variant::Concat => self.concat(size),
            Variant::Choice => self.choice(size),
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

    fn newline(&mut self) -> Notation {
        Newline
    }

    fn flat(&mut self, size: usize) -> Notation {
        Flat(Box::new(self.notation(size - 1)))
    }

    fn align(&mut self, size: usize) -> Notation {
        Align(Box::new(self.notation(size - 1)))
    }

    fn indent(&mut self, size: usize) -> Notation {
        let indent = self.rng.gen_range(self.indent_range.0, self.indent_range.1);
        Indent(indent, Box::new(self.notation(size - 1)))
    }

    fn concat(&mut self, size: usize) -> Notation {
        let (left, right) = self.bifurcate(size - 1);
        Concat(Box::new(left), Box::new(right))
    }

    fn choice(&mut self, size: usize) -> Notation {
        self.num_choices -= 1;
        let (left, right) = self.bifurcate(size - 1);
        Choice(Box::new(left), Box::new(right))
    }

    fn bifurcate(&mut self, size: usize) -> (Notation, Notation) {
        let left_size = self.rng.gen_range(1, size);
        let right_size = size - left_size;
        (self.notation(left_size), self.notation(right_size))
    }
}
