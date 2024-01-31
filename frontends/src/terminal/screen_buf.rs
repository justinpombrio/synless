use super::TermError;
use partial_pretty_printer::{pane::PrettyWindow, Pos, ShadedStyle, Size, Width};
use std::mem;

/// Represents a screen full of characters. It buffers changes to the
/// characters, and can produce a set of instructions for efficiently updating
/// the screen to reflect those changes.
#[derive(Debug)]
pub struct ScreenBuf {
    /// This should always contain the number of lines and cols requested by the
    /// user (eg. 0-by-5), even if `cells` is empty.
    size: Size,
    /// Grid of characters covering the screen.
    new_buffer: Buffer,
    /// Previous buffer, if any.
    old_buffer: Option<Buffer>,
}

#[derive(Debug)]
struct Buffer(Vec<Vec<CharCell>>);

/// Represents a single character on a screen, with style properties.
#[derive(Clone, Copy, Debug, PartialEq)]
struct CharCell {
    ch: char,
    style: ShadedStyle,
}

/// Instructions for how to update a terminal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScreenOp {
    /// Print a character at the current cursor position, and advance the cursor.
    Print(char),
    /// Set a persistent style that will apply to anything printed, until a new style is applied.
    Style(ShadedStyle),
    /// Set the cursor position.
    Goto(Pos),
}

/// An iterator that produces instructions for updating a screen to match changes in a ScreenBuf.
pub struct ScreenBufIter<'a> {
    size: Size,
    new_buffer: &'a Buffer,
    old_buffer: Option<Buffer>,
    /// The last style applied to the screen. It will persist until a new style is applied.
    screen_style: Option<ShadedStyle>,
    /// The screen's cursor position. The next printed char will appear at this position.
    screen_pos: Option<Pos>,
    /// Which cell the iterator is considering (NOT the position of the
    /// terminal's cursor). None means we're past the end / done iterating.
    buffer_pos: Option<Pos>,
}

fn char_width(_ch: char) -> Width {
    // TODO: Full-width char support
    1
}

impl Buffer {
    fn new(size: Size) -> Buffer {
        // TODO: Make this a 1d vector, using multiplication to access
        Buffer(vec![
            vec![CharCell::default(); size.width as usize];
            size.height as usize
        ])
    }

    fn get(&self, pos: Pos) -> Result<CharCell, TermError> {
        self.0
            .get(pos.row as usize)
            .and_then(|line| line.get(pos.col as usize))
            .copied()
            .ok_or(TermError::OutOfBounds)
    }

    fn get_mut(&mut self, pos: Pos) -> Result<&mut CharCell, TermError> {
        self.0
            .get_mut(pos.row as usize)
            .and_then(|line| line.get_mut(pos.col as usize))
            .ok_or(TermError::OutOfBounds)
    }
}

impl ScreenBuf {
    /// Create a new ScreenBuf with the given number of rows and columns of character cells
    pub fn new(size: Size) -> Self {
        ScreenBuf {
            new_buffer: Buffer::new(size),
            old_buffer: None,
            size,
        }
    }

    /// Get `ScreenOp` instructions that describe all changes to the screen buffer since the last time this method was called.
    pub fn drain_changes(&mut self) -> ScreenBufIter {
        // Swap buffers
        let old_buffer = self.old_buffer.take();
        let new_buffer = mem::replace(&mut self.new_buffer, Buffer::new(self.size));
        self.old_buffer = Some(new_buffer);

        ScreenBufIter {
            size: self.size,
            old_buffer,
            new_buffer: self.old_buffer.as_ref().unwrap(),
            screen_style: None,
            screen_pos: None,
            buffer_pos: Some(Pos::zero()),
        }
    }

    /// Clear the screen buffer and change the number of rows and columns of character cells
    pub fn resize(&mut self, size: Size) {
        self.new_buffer = Buffer::new(size);
        self.old_buffer = None;
        self.size = size;
    }
}

impl PrettyWindow for ScreenBuf {
    type Error = TermError;

    /// Return the current size of the screen buffer, without checking the
    /// actual size of the terminal window (which might have changed recently).
    fn size(&self) -> Result<Size, Self::Error> {
        Ok(self.size)
    }

    /// No newlines allowed. If the string doesn't fit between the starting
    /// column position and the right edge of the screen, it's truncated and
    /// and an OutOfBounds error is returned.
    fn print(&mut self, mut pos: Pos, string: &str, style: ShadedStyle) -> Result<(), Self::Error> {
        for ch in string.chars() {
            let cell = self.new_buffer.get_mut(pos)?;
            cell.ch = ch;
            cell.style = style;
            pos.col += char_width(ch);
        }
        Ok(())
    }

    fn fill(
        &mut self,
        mut pos: Pos,
        ch: char,
        len: Width,
        style: ShadedStyle,
    ) -> Result<(), Self::Error> {
        // TODO: Make sure this method gets deleted.
        // This impl is broken if `ch` is full width.
        for _ in 0..len {
            let cell = self.new_buffer.get_mut(pos)?;
            cell.ch = ch;
            cell.style = style;
            pos.col += char_width(ch);
        }
        Ok(())
    }
}

impl Default for CharCell {
    fn default() -> Self {
        CharCell {
            ch: ' ',
            style: ShadedStyle::plain(),
        }
    }
}

impl<'a> ScreenBufIter<'a> {
    fn next_pos(&self, pos: Pos, char_width: Width) -> Option<Pos> {
        if pos.col + char_width >= self.size.width {
            // At the last column of a line
            if pos.row + 1 >= self.size.height {
                // At the last line too, that's the last position!
                None
            } else {
                // Go to start of next line
                Some(Pos {
                    row: pos.row + 1,
                    col: 0,
                })
            }
        } else {
            // Go forward 1 character
            Some(Pos {
                row: pos.row,
                col: pos.col + char_width,
            })
        }
    }
}

impl<'a> Iterator for ScreenBufIter<'a> {
    type Item = ScreenOp;

    fn next(&mut self) -> Option<ScreenOp> {
        loop {
            let pos = match self.buffer_pos {
                None => return None,
                Some(pos) => pos,
            };
            let new_cell = self.new_buffer.get(pos).unwrap();
            let old_cell = self
                .old_buffer
                .as_ref()
                .map(|old_buffer| old_buffer.get(pos).unwrap());
            let is_dirty = old_cell
                .map(|old_cell| old_cell != new_cell)
                .unwrap_or(true);

            if is_dirty {
                // 1. Update position, if needed
                if self.screen_pos != Some(pos) {
                    self.screen_pos = Some(pos);
                    return Some(ScreenOp::Goto(pos));
                }
                // 2. Update style, if needed
                if self.screen_style != Some(new_cell.style) {
                    self.screen_style = Some(new_cell.style);
                    return Some(ScreenOp::Style(new_cell.style));
                }
                // 3. Write char
                let width = char_width(new_cell.ch);
                self.screen_pos.as_mut().unwrap().col += width;
                self.buffer_pos = self.next_pos(pos, width);
                return Some(ScreenOp::Print(new_cell.ch));
            } else if let Some(next_pos) = self.next_pos(pos, char_width(new_cell.ch)) {
                self.buffer_pos = Some(next_pos);
            } else {
                return None;
            }
        }
    }
}

#[cfg(test)]
mod screen_buf_tests {
    use super::*;
    use partial_pretty_printer::{Color, Pos, Shade, ShadedStyle, Size, Style};

    fn assert_out_of_bounds(result: Result<(), TermError>) {
        match result {
            Err(TermError::OutOfBounds) => (),
            x => panic!("expected OutOfBounds error, got {:?}", x),
        }
    }

    fn assert_resized(buf: &mut ScreenBuf, size: Size, good_pos: &[Pos], bad_pos: &[Pos]) {
        buf.resize(size);
        assert_eq!(buf.size().unwrap(), size);
        for &pos in good_pos {
            buf.print(pos, "x", ShadedStyle::plain())
                .unwrap_or_else(|_| panic!("pos {} out-of-bounds of buf with size {}", pos, size));
        }
        for &pos in bad_pos {
            assert_out_of_bounds(buf.print(pos, "x", ShadedStyle::plain()));
        }
    }

    #[test]
    fn test_resize() {
        let c0r0 = Pos { col: 0, row: 0 };
        let c0r1 = Pos { col: 0, row: 1 };
        let c1r0 = Pos { col: 1, row: 0 };
        let c1r1 = Pos { col: 1, row: 1 };
        let c5r8 = Pos { col: 5, row: 8 };
        let c5r7 = Pos { col: 5, row: 7 };
        let c4r7 = Pos { col: 4, row: 7 };
        let c4r8 = Pos { col: 4, row: 8 };

        let mut buf = ScreenBuf::new(Size {
            height: 1,
            width: 1,
        });
        assert_eq!(
            buf.size().unwrap(),
            Size {
                height: 1,
                width: 1,
            },
        );
        assert_resized(
            &mut buf,
            Size {
                width: 1,
                height: 0,
            },
            &[],
            &[c0r0, c1r0, c0r1],
        );
        assert_resized(
            &mut buf,
            Size {
                width: 0,
                height: 1,
            },
            &[],
            &[c0r0, c1r0, c0r1],
        );
        assert_resized(
            &mut buf,
            Size {
                width: 1,
                height: 1,
            },
            &[c0r0],
            &[c1r0, c0r1, c1r1],
        );
        assert_resized(
            &mut buf,
            Size {
                width: 5,
                height: 8,
            },
            &[c0r0, c1r0, c0r1, c1r1, c4r7],
            &[c5r8, c4r8, c5r7],
        );
    }

    #[test]
    fn test_simple() {
        let style1 = ShadedStyle::new(Style::color(Color::Base09), Shade::background());
        let mut buf = ScreenBuf::new(Size {
            width: 3,
            height: 2,
        });

        let pos = Pos { col: 2, row: 0 };
        buf.print(pos, "x", style1).unwrap();
        let mut actual_ops: Vec<_> = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos::zero()),
                ScreenOp::Style(ShadedStyle::plain()),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Style(style1),
                ScreenOp::Print('x'),
                ScreenOp::Goto(Pos { row: 1, col: 0 }),
                ScreenOp::Style(ShadedStyle::plain()),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
            ]
        );
        actual_ops = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(pos),
                ScreenOp::Style(ShadedStyle::plain()),
                ScreenOp::Print(' '),
            ]
        );
    }

    #[test]
    fn test_no_change() {
        let style1 = ShadedStyle::new(Style::color(Color::Base09), Shade::background());
        let mut buf = ScreenBuf::new(Size {
            width: 3,
            height: 2,
        });

        let pos = Pos { col: 2, row: 0 };
        buf.print(pos, "x", style1).unwrap();
        let mut actual_ops: Vec<_> = buf.drain_changes().collect();

        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos::zero()),
                ScreenOp::Style(ShadedStyle::plain()),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Style(style1),
                ScreenOp::Print('x'),
                ScreenOp::Goto(Pos { row: 1, col: 0 }),
                ScreenOp::Style(ShadedStyle::plain()),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
            ]
        );

        // Print same thing as before
        buf.print(pos, "x", style1).unwrap();
        actual_ops = buf.drain_changes().collect();
        assert_eq!(actual_ops, vec![]);
    }

    #[test]
    fn test_shorten() {
        let style1 = ShadedStyle::new(Style::color(Color::Base09), Shade::background());

        let mut buf = ScreenBuf::new(Size {
            width: 3,
            height: 1,
        });

        buf.print(Pos::zero(), "xyz", style1).unwrap();
        let mut actual_ops: Vec<_> = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos::zero()),
                ScreenOp::Style(style1),
                ScreenOp::Print('x'),
                ScreenOp::Print('y'),
                ScreenOp::Print('z'),
            ]
        );

        buf.print(Pos::zero(), "xy", style1).unwrap();
        actual_ops = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos { col: 2, row: 0 }),
                ScreenOp::Style(ShadedStyle::plain()),
                ScreenOp::Print(' '),
            ]
        );

        buf.print(Pos::zero(), "x", style1).unwrap();
        actual_ops = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos { col: 1, row: 0 }),
                ScreenOp::Style(ShadedStyle::plain()),
                ScreenOp::Print(' '),
            ]
        );

        buf.print(Pos::zero(), "xy", style1).unwrap();
        actual_ops = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos { col: 1, row: 0 }),
                ScreenOp::Style(style1),
                ScreenOp::Print('y'),
            ]
        );
    }

    #[test]
    fn test_complex() {
        let style1 = ShadedStyle::new(Style::color(Color::Base09), Shade::background());
        let style2 = ShadedStyle::new(Style::color(Color::Base0C), Shade::background());

        let mut buf = ScreenBuf::new(Size {
            width: 3,
            height: 4,
        });

        buf.print(Pos { col: 1, row: 0 }, "fo", style1).unwrap();
        buf.print(Pos { col: 0, row: 1 }, "oba", style1).unwrap();
        buf.print(Pos { col: 0, row: 2 }, "r", style1).unwrap();

        buf.print(Pos { col: 0, row: 1 }, "OB", style2).unwrap();

        buf.print(Pos { col: 2, row: 3 }, "$", ShadedStyle::plain())
            .unwrap();

        let actual_ops: Vec<_> = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos::zero()),
                ScreenOp::Style(ShadedStyle::plain()),
                ScreenOp::Print(' '),
                ScreenOp::Style(style1),
                ScreenOp::Print('f'),
                ScreenOp::Print('o'),
                ScreenOp::Goto(Pos { row: 1, col: 0 }),
                ScreenOp::Style(style2),
                ScreenOp::Print('O'),
                ScreenOp::Print('B'),
                ScreenOp::Style(style1),
                ScreenOp::Print('a'),
                ScreenOp::Goto(Pos { row: 2, col: 0 }),
                ScreenOp::Print('r'),
                ScreenOp::Style(ShadedStyle::plain()),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Goto(Pos { row: 3, col: 0 }),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Print('$'),
            ]
        );

        let actual_ops: Vec<_> = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos { row: 0, col: 1 }),
                ScreenOp::Style(ShadedStyle::plain()),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Goto(Pos { row: 1, col: 0 }),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Goto(Pos { row: 2, col: 0 }),
                ScreenOp::Print(' '),
                ScreenOp::Goto(Pos { row: 3, col: 2 }),
                ScreenOp::Print(' '),
            ]
        );

        buf.print(Pos { col: 2, row: 3 }, "!", ShadedStyle::plain())
            .unwrap();

        let actual_ops: Vec<_> = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos { col: 2, row: 3 }),
                ScreenOp::Style(ShadedStyle::plain()),
                ScreenOp::Print('!'),
            ]
        );
    }
}
