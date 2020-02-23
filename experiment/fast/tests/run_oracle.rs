/*
mod common;

use common::{oracular_pretty_print, NotationGenerator};
use fast::{pretty_print, Notation};

const WIDTH_RANGE: (usize, usize) = (5, 20);
const NUM_TESTS: usize = 10000;
const SEED: u64 = 20;

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

#[test]
#[ignore]
fn run_oracle() {
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
            error.width,
        );
        assert!(false);
    }
}
*/
