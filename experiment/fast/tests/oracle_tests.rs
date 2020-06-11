mod common;

use common::{oracular_pretty_print, NotationGenerator, NotationGeneratorConfig};
use fast::{partial_pretty_print_first, partial_pretty_print_last, pretty_print, Notation};

// Tests passed with:
// - NUM_TESTS = 10_000_000 & SEED = 28
const NUM_TESTS: usize = 10000;
const SEED: u64 = 28;

const MAX_CHOICES: usize = 5;
const SIZE_RANGE: (usize, usize) = (1, 50);
const WIDTH_RANGE: (usize, usize) = (1, 25);
const LITERAL_RANGE: (usize, usize) = (0, 10);
const INDENT_RANGE: (usize, usize) = (0, 10);
const NUM_PARTIAL_LINES_RANGE: (usize, usize) = (1, 5);

enum PPResult {
    Ok,
    Invalid,
    Error(PPError),
}

enum Mode {
    PrettyPrint,
    PartialPrettyPrintFirst(usize),
    PartialPrettyPrintLast(usize),
}

struct PPError {
    notation: Notation,
    width: usize,
    actual: Vec<String>,
    oracular: Vec<String>,
    mode: Mode,
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
        let oracle_lines = expand_lines(oracular_pretty_print(&notation, width));
        // Test the regular printer
        let actual_lines = expand_lines(pretty_print(&measured_notation, width));
        if actual_lines != oracle_lines {
            return PPResult::Error(PPError {
                notation,
                width,
                actual: actual_lines,
                oracular: oracle_lines,
                mode: Mode::PrettyPrint,
            });
        }
        // Test the partial pretty printer, printing the first lines
        let range = NUM_PARTIAL_LINES_RANGE.clone();
        for num_partial_lines in range.0..range.1 {
            let actual_lines_iter = partial_pretty_print_first(&measured_notation, width);
            let actual_lines = expand_lines(actual_lines_iter.take(num_partial_lines).collect());
            let oracle_lines = oracle_lines
                .iter()
                .take(num_partial_lines)
                .map(|s| s.to_string())
                .collect();
            if actual_lines != oracle_lines {
                return PPResult::Error(PPError {
                    notation,
                    width,
                    actual: actual_lines,
                    oracular: oracle_lines,
                    mode: Mode::PartialPrettyPrintFirst(num_partial_lines),
                });
            }
        }
        // Test the partial pretty pritner, printing the last lines
        let range = NUM_PARTIAL_LINES_RANGE.clone();
        for num_partial_lines in range.0..range.1 {
            let actual_lines_iter = partial_pretty_print_last(&measured_notation, width);
            let mut actual_lines =
                expand_lines(actual_lines_iter.take(num_partial_lines).collect());
            actual_lines.reverse();
            let oracle_lines = oracle_lines
                .iter()
                .rev()
                .take(num_partial_lines)
                .rev()
                .map(|s| s.to_string())
                .collect();
            if actual_lines != oracle_lines {
                return PPResult::Error(PPError {
                    notation,
                    width,
                    actual: actual_lines,
                    oracular: oracle_lines,
                    mode: Mode::PartialPrettyPrintLast(num_partial_lines),
                });
            }
        }
    }
    PPResult::Ok
}

#[test]
fn run_oracle() {
    let mut error = None;
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
            PPResult::Error(err) => {
                error = Some(err);
                num_errors += 1;
            }
        }
    }
    eprintln!(
        "Tested {} notations. {} were invalid. {} were printed incorrectly.",
        NUM_TESTS, num_invalid, num_errors
    );
    if let Some(error) = error {
        let printer = match error.mode {
            Mode::PrettyPrint => "PRETTY PRINTER".to_string(),
            Mode::PartialPrettyPrintFirst(num_lines) => {
                format!("PARTIAL PRETTY PRINTING OF THE FIRST {} LINES", num_lines)
            }
            Mode::PartialPrettyPrintLast(num_lines) => {
                format!("PARTIAL PRETTY PRINTING OF THE LAST {} LINES", num_lines)
            }
        };
        eprintln!(
            "{} PRODUCED:\n{}\nBUT ORACLE SAYS IT SHOULD BE:\n{}\nNOTATION:\n{:#?}\nWIDTH:{}",
            printer,
            error.actual.join("\n"),
            error.oracular.join("\n"),
            error.notation,
            error.width,
        );
        assert!(false);
    }
}
