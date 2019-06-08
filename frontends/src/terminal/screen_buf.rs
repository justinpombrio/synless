use pretty::{Pos, Rect, Region};
use pretty::{Shade, Style};

use super::Error;

/// Represents a screen full of characters. It buffers changes to the
/// characters, and can produce a set of instructions for efficiently updating
/// the screen to reflect those changes.
#[derive(Debug)]
pub struct ScreenBuf {
    cells: Vec<Vec<CharCell>>,
    // This should always contain the number of rows and cols requested by the
    // user (eg. 0-by-5), even if `cells` is empty.
    size: Pos,
}

/// Represents a single character on a screen, with style properties.
/// It keeps track of whether it has changed since the last time the ScreenBuf was redisplayed.
#[derive(Clone, Copy, Debug)]
struct CharCell {
    ch: char,
    shade: Shade,
    style: Style, // TODO except background is ignored?
    needs_redisplay: bool,
}

/// Instructions for how to update a terminal.
#[derive(Clone, Copy)]
pub enum ScreenOp {
    Print(char),
    Apply(Style),
    Goto(Pos),
}

/// An iterator that produces instructions for updating a screen to match changes in a ScreenBuf.
pub struct ScreenBufIter<'a> {
    buf: &'a mut ScreenBuf,
    current_style: Option<Style>,
    cursor_pos: Option<Pos>,
}

impl ScreenBuf {
    pub fn new() -> Self {
        ScreenBuf {
            cells: Vec::new(),
            size: Pos::zero(),
        }
    }

    /// Get `ScreenOp` instructions that describe all changes to the screen buffer since the last time this method was called.
    pub fn drain_changes(&mut self) -> ScreenBufIter {
        ScreenBufIter {
            buf: self,
            current_style: None,
            cursor_pos: None,
        }
    }

    pub fn resize(&mut self, size: Pos) {
        self.cells = Vec::new();
        let mut row = Vec::new();
        row.resize_with(size.col as usize, Default::default);
        self.cells.resize(size.row as usize, row);
        self.size = size;
    }

    pub fn clear(&mut self) {
        self.resize(self.size);
    }

    pub fn size(&self) -> Pos {
        self.size
    }

    pub fn write_str(&mut self, pos: Pos, s: &str, style: Style) -> Result<(), Error> {
        let mut char_pos = pos;
        for ch in s.chars() {
            self.set_char_with_style(char_pos, ch, style)?;
            char_pos = self.next_pos(char_pos).ok_or(Error::OutOfBounds)?;
        }
        Ok(())
    }

    pub fn shade_region(&mut self, region: Region, shade: Shade) -> Result<(), Error> {
        let mut shade_rect = |rect: Rect| {
            for r in rect.rows.0..rect.rows.1 {
                for c in rect.cols.0..rect.cols.1 {
                    let pos = Pos { row: r, col: c };
                    self.set_shade(pos, shade)?;
                }
            }
            Ok(())
        };
        shade_rect(region.body())?;
        shade_rect(region.last_line())
    }

    pub fn set_style(&mut self, pos: Pos, style: Style) -> Result<(), Error> {
        self.get_mut(pos)?.set_style(style);
        Ok(())
    }

    fn set_char_with_style(&mut self, pos: Pos, ch: char, style: Style) -> Result<(), Error> {
        let cell = self.get_mut(pos)?;
        cell.set_char(ch);
        cell.set_style(style);
        Ok(())
    }

    fn set_shade(&mut self, pos: Pos, shade: Shade) -> Result<(), Error> {
        self.get_mut(pos)?.set_shade(shade);
        Ok(())
    }

    fn get(&self, pos: Pos) -> Result<CharCell, Error> {
        self.cells
            .get(pos.row as usize)
            .and_then(|row| row.get(pos.col as usize))
            .map(|cell| *cell)
            .ok_or(Error::OutOfBounds)
    }

    fn get_mut(&mut self, pos: Pos) -> Result<&mut CharCell, Error> {
        self.cells
            .get_mut(pos.row as usize)
            .and_then(|row| row.get_mut(pos.col as usize))
            .ok_or(Error::OutOfBounds)
    }

    fn next_pos(&self, old_pos: Pos) -> Option<Pos> {
        let size = self.size();
        if old_pos.col >= (size.col - 1) {
            // At the last column of a row
            if old_pos.row >= (size.row - 1) {
                // At the last row too, that's the last position on the the screen!
                None
            } else {
                // Go to start of next row
                Some(Pos {
                    row: old_pos.row + 1,
                    col: 0,
                })
            }
        } else {
            // Go forward 1 column
            Some(Pos {
                row: old_pos.row,
                col: old_pos.col + 1,
            })
        }
    }
}

impl CharCell {
    fn set_char(&mut self, ch: char) {
        if self.ch != ch {
            self.needs_redisplay = true;
            self.ch = ch;
        }
    }

    fn set_style(&mut self, style: Style) {
        if self.style != style {
            self.needs_redisplay = true;
            self.style = style;
        }
    }

    fn set_shade(&mut self, shade: Shade) {
        if self.shade != shade {
            self.needs_redisplay = true;
            self.shade = shade;
        }
    }

    fn get_shaded_style(&self) -> Style {
        let mut style = self.style;
        style.shade = self.shade;
        style
    }
}

impl Default for CharCell {
    fn default() -> Self {
        CharCell {
            ch: ' ',
            shade: Shade::default(),
            style: Style::default(),
            needs_redisplay: true,
        }
    }
}

impl<'a> Iterator for ScreenBufIter<'a> {
    type Item = ScreenOp;
    fn next(&mut self) -> Option<ScreenOp> {
        // Start where we left off last time `next()` was called, or at the beginning of the buffer.
        let mut pos = self.cursor_pos.unwrap_or(Pos::zero());

        // Look for the next cell that needs to be redisplayed.
        let mut steps = 0;
        while !self.buf.get(pos).unwrap().needs_redisplay {
            pos = self.buf.next_pos(pos)?;
            steps += 1;
        }

        // Check if we need to explicitly jump the cursor to this position.
        let cursor_pos_was_unknown = self.cursor_pos.replace(pos).is_none();
        if steps > 1 || cursor_pos_was_unknown {
            return Some(ScreenOp::Goto(pos));
        }
        let pos = pos; // immutable now

        // Check if it has a different style than the last character we printed.
        let style = self.buf.get(pos).unwrap().get_shaded_style();
        let style_changed = match self.current_style {
            None => true,
            Some(s) => s != style,
        };

        if style_changed {
            self.current_style = Some(style);
            return Some(ScreenOp::Apply(style));
        }

        // Finally, print the character itself and mark that it's been redisplayed.
        self.buf.get_mut(pos).unwrap().needs_redisplay = false;
        Some(ScreenOp::Print(self.buf.get(pos).unwrap().ch))
    }
}
