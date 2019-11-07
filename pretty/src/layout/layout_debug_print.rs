use super::compute_layout::{Layout, LayoutElement};
use crate::geometry::Pos;
use std::fmt;

impl fmt::Debug for Layout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut printer = LayoutDebugPrinter::new();
        for element in &self.elements {
            printer.write_element(element);
        }
        for (i, line) in printer.lines.iter().enumerate() {
            for ch in line {
                write!(f, "{}", ch)?;
            }
            if i != printer.lines.len() - 1 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

struct LayoutDebugPrinter {
    lines: Vec<Vec<char>>,
}

impl LayoutDebugPrinter {
    fn new() -> LayoutDebugPrinter {
        LayoutDebugPrinter { lines: vec![] }
    }

    fn write_char(&mut self, ch: char, pos: Pos) {
        while pos.row as usize >= self.lines.len() {
            self.lines.push(vec![]);
        }
        let line = &mut self.lines[pos.row as usize];
        while pos.col as usize >= line.len() {
            line.push(' ');
        }
        line[pos.col as usize] = ch;
    }

    fn write_element(&mut self, element: &LayoutElement) {
        match element {
            LayoutElement::Literal(region, string, _) => {
                let mut pos = region.beginning();
                for ch in string.chars() {
                    self.write_char(ch, pos);
                    pos.col += 1;
                }
            }
            LayoutElement::Text(region, _) => {
                for pos in region.positions() {
                    self.write_char('0', pos);
                }
            }
            LayoutElement::Child(region, i) => {
                for pos in region.positions() {
                    let ch: char = (('0' as u8) + (*i as u8)).into();
                    self.write_char(ch, pos);
                }
            }
        }
    }
}
