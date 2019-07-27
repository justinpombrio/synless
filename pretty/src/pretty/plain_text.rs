use std::fmt;

use super::pretty_window::PrettyWindow;
use crate::geometry::{Col, Pos, Rect, Region, Row};
use crate::style::{Shade, Style};

/// Render a document in plain text.
pub struct PlainText {
    rect: Rect,
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
    /// Construct a screen with the given width and height.
    pub fn new(size: Pos) -> PlainText {
        PlainText {
            rect: Rect::new(Pos::zero(), size),
            lines: vec![],
        }
    }

    /// Construct a screen with the given width and unbounded height.
    pub fn new_infinite_scroll(width: Col) -> PlainText {
        let size = Pos {
            col: width,
            // leave wiggle-room to avoid overflowing
            row: Row::max_value() - 1,
        };
        PlainText::new(size)
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

    fn size(&self) -> Result<Pos, Self::Error> {
        Ok(self.rect.size())
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
