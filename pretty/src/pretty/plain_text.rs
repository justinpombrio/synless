use std::fmt;

use super::pretty_window::PrettyWindow;
use crate::geometry::{Bound, Col, Pos, Region};
use crate::style::{Shade, Style};

/// Render a document in plain text.
pub struct PlainText {
    bound: Bound,
    lines: Vec<Vec<char>>,
}

impl fmt::Display for PlainText {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, line) in self.lines.iter().enumerate() {
            for ch in line {
                write!(f, "{}", ch)?;
            }
            if i + 1 != self.lines.len() {
                write!(f, "\n")?;
            }
        }
        Ok(())
    }
}

impl PlainText {
    /// Construct a screen with the given width and unbounded height.
    pub fn new(width: usize) -> PlainText {
        PlainText {
            bound: Bound::infinite_scroll(width as Col),
            lines: vec![],
        }
    }

    /// Construct a screen with the given width, height, and indent.
    pub fn new_bounded(bound: Bound) -> PlainText {
        PlainText {
            bound,
            lines: vec![],
        }
    }

    fn get_mut_line(&mut self, row: usize) -> &mut Vec<char> {
        if self.lines.len() < row + 1 {
            self.lines.resize(row + 1, vec![]);
        }
        &mut self.lines[row as usize]
    }

    fn get_mut_slice(&mut self, row: usize, col: usize, len: usize) -> &mut [char] {
        let line = self.get_mut_line(row);
        if line.len() < col + len {
            line.resize(col + len, ' ');
        }
        &mut line[col..col + len]
    }
}

impl PrettyWindow for PlainText {
    type Error = fmt::Error;

    fn bound(&self) -> Result<Bound, Self::Error> {
        Ok(self.bound)
    }

    fn print(&mut self, pos: Pos, text: &str, _style: Style) -> Result<(), Self::Error> {
        let slice = self.get_mut_slice(pos.row as usize, pos.col as usize, text.chars().count());
        for (i, ch) in text.chars().enumerate() {
            slice[i] = ch;
        }
        Ok(())
    }

    fn shade(&mut self, _region: Region, _shade: Shade) -> Result<(), Self::Error> {
        Ok(())
    }

    fn highlight(&mut self, _pos: Pos, _style: Style) -> Result<(), Self::Error> {
        Ok(())
    }
}
