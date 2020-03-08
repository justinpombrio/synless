mod common;

use common::{oracular_pretty_print, NotationGenerator, NotationGeneratorConfig};
use fast::{pretty_print, Notation};

// Tested with:
// - NUM_TESTS = 10_000_000 & SEED = 28
const NUM_TESTS: usize = 100000;
const SEED: u64 = 28;

const MAX_CHOICES: usize = 5;
const SIZE_RANGE: (usize, usize) = (1, 50);
const WIDTH_RANGE: (usize, usize) = (1, 25);
const LITERAL_RANGE: (usize, usize) = (0, 10);
const INDENT_RANGE: (usize, usize) = (0, 10);

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
    match notation.clone().validate() {
        Ok(()) => (),
        Err(_) => return PPResult::Invalid,
    };
    let measured_notation = notation.measure();
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

#[test]
fn run_oracle() {
    let mut first_error = None;
    let mut num_invalid = 0;
    let mut num_errors = 0;
    let config = NotationGeneratorConfig {
        max_choices: MAX_CHOICES,
        size_range: SIZE_RANGE,
        literal_range: LITERAL_RANGE,
        indent_range: INDENT_RANGE,
    };
    let mut generator = NotationGenerator::new(SEED, config);
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
            error.width,
        );
        assert!(false);
    }
}
