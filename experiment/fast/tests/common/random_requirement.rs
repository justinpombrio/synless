use fast::{AlignedMultiLine, MultiLine, Requirement};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

const LINE_RANGE: (usize, usize) = (0, 10);

pub struct RequirementGenerator {
    master_rng: StdRng,
}

impl RequirementGenerator {
    fn new(seed: u64) -> RequirementGenerator {
        RequirementGenerator {
            master_rng: StdRng::seed_from_u64(seed),
        }
    }

    fn random_requirement(&mut self) -> Requirement {
        let mut rng = StdRng::seed_from_u64(self.master_rng.gen());
        let mut req = Requirement {
            single_line: None,
            multi_line: None,
            aligned: None,
        };

        if rng.gen() {
            req.single_line = Some(rng.gen_range(LINE_RANGE.0, LINE_RANGE.1));
        }
        if rng.gen() {
            req.multi_line = Some(MultiLine {
                first: rng.gen_range(LINE_RANGE.0, LINE_RANGE.1),
                middle: rng.gen_range(LINE_RANGE.0, LINE_RANGE.1),
                last: rng.gen_range(LINE_RANGE.0, LINE_RANGE.1),
            });
        }
        if rng.gen() {
            req.aligned = Some(AlignedMultiLine {
                middle: rng.gen_range(LINE_RANGE.0, LINE_RANGE.1),
                last: rng.gen_range(LINE_RANGE.0, LINE_RANGE.1),
            });
        }
        req
    }
}

#[cfg(test)]
mod tests {
    use super::RequirementGenerator;

    const WIDTH_RANGE: (usize, usize) = (5, 20);
    const NUM_TESTS: usize = 10000;
    const SEED: u64 = 20;

    // #[test]
    // fn test_rand_req() {
    //     let mut generator = RequirementGenerator::new(SEED);
    //     let req = generator.random_requirement();
    //     println!("{:?}", req);
    // }
}
