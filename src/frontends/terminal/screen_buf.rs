use super::TermError;
use crate::style::ConcreteStyle;
use partial_pretty_printer::{pane::PrettyWindow, Height, Pos, Size, Width};
use std::mem;

// TODO: ScreenBuf thinks you don't need to re-print a space over the second half of a deleted
// full-width character. Is that true? How do terminals work? (search "reprinted")

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
    /// The style to use for CharCells that haven't been written to.
    blank_style: ConcreteStyle,
}

#[derive(Debug)]
struct Buffer {
    cells: Vec<CharCell>,
    size: Size,
}

/// Represents a single character on a screen, with style properties.
#[derive(Clone, Debug, PartialEq)]
struct CharCell {
    ch: char,
    style: ConcreteStyle,
    width: Width,
}

/// Instructions for how to update a terminal.
#[derive(Clone, Debug, PartialEq)]
pub enum ScreenOp {
    /// Print a character at the current cursor position, and advance the cursor.
    Print(char),
    /// Set a persistent style that will apply to anything printed, until a new style is applied.
    Style(ConcreteStyle),
    /// Set the cursor position.
    Goto(Pos),
}

/// An iterator that produces instructions for updating a screen to match changes in a ScreenBuf.
pub struct ScreenBufIter<'a> {
    size: Size,
    new_buffer: &'a Buffer,
    old_buffer: Option<Buffer>,
    /// The last style applied to the screen. It will persist until a new style is applied.
    screen_style: Option<ConcreteStyle>,
    /// The screen's cursor position. The next printed char will appear at this position.
    screen_pos: Option<Pos>,
    /// Which cell the iterator is considering (NOT the position of the
    /// terminal's cursor). None means we're past the end / done iterating.
    buffer_pos: Option<Pos>,
}

impl Buffer {
    fn new(size: Size, blank_style: ConcreteStyle) -> Buffer {
        let blank_cell = CharCell {
            ch: ' ',
            style: blank_style,
            width: 1,
        };
        Buffer {
            cells: vec![blank_cell; (size.width as usize) * (size.height as usize)],
            size,
        }
    }

    fn index(&self, pos: Pos) -> Result<usize, TermError> {
        if pos.col >= self.size.width || pos.row >= self.size.height {
            Err(TermError::OutOfBounds)
        } else {
            Ok((pos.row as usize) * (self.size.width as usize) + (pos.col as usize))
        }
    }

    fn get(&self, pos: Pos) -> Result<CharCell, TermError> {
        Ok(self.cells[self.index(pos)?].clone())
    }

    fn get_mut(&mut self, pos: Pos) -> Result<&mut CharCell, TermError> {
        let i = self.index(pos)?;
        Ok(&mut self.cells[i])
    }
}

impl ScreenBuf {
    /// Create a new ScreenBuf with the given number of rows and columns of character cells
    pub fn new(size: Size, blank_style: ConcreteStyle) -> Self {
        ScreenBuf {
            new_buffer: Buffer::new(size, blank_style),
            old_buffer: None,
            size,
            blank_style,
        }
    }

    /// Get `ScreenOp` instructions that describe all changes to the screen buffer since the last time this method was called.
    pub fn drain_changes(&mut self) -> ScreenBufIter {
        // Swap buffers
        let old_buffer = self.old_buffer.take();
        let new_buffer = mem::replace(
            &mut self.new_buffer,
            Buffer::new(self.size, self.blank_style),
        );
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
        self.new_buffer = Buffer::new(size, self.blank_style);
        self.old_buffer = None;
        self.size = size;
    }

    /// Return the current size of the screen buffer, without checking the
    /// actual size of the terminal window (which might have changed recently).
    pub fn size(&self) -> Size {
        self.size
    }

    /// Display a character at the given window position in the given style. `full_width` indicates
    /// whether the character is 1 (`false`) or 2 (`true`) columns wide. The character is guaranteed
    /// to fit in the window and not overlap or overwrite any other characters.
    pub fn display_char(
        &mut self,
        ch: char,
        pos: Pos,
        style: ConcreteStyle,
        width: Width,
    ) -> Result<(), TermError> {
        let cell = self.new_buffer.get_mut(pos)?;
        cell.ch = ch;
        cell.style = style;
        cell.width = width;
        Ok(())
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
                self.screen_pos.as_mut().unwrap().col += new_cell.width;
                self.buffer_pos = self.next_pos(pos, new_cell.width);
                return Some(ScreenOp::Print(new_cell.ch));
            } else if let Some(next_pos) = self.next_pos(pos, new_cell.width) {
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
    use crate::style::{ConcreteStyle, Rgb};
    use partial_pretty_printer::{Pos, Size};

    const STYLE_DEFAULT: ConcreteStyle = ConcreteStyle {
        fg_color: Rgb {
            red: 255,
            green: 255,
            blue: 255,
        },
        bg_color: Rgb {
            red: 0,
            green: 0,
            blue: 0,
        },
        bold: false,
        underlined: false,
    };

    const STYLE_RED: ConcreteStyle = ConcreteStyle {
        fg_color: Rgb {
            red: 255,
            green: 0,
            blue: 0,
        },
        bg_color: Rgb {
            red: 50,
            green: 0,
            blue: 0,
        },
        bold: false,
        underlined: false,
    };

    const STYLE_GREEN: ConcreteStyle = ConcreteStyle {
        fg_color: Rgb {
            red: 0,
            green: 255,
            blue: 0,
        },
        bg_color: Rgb {
            red: 0,
            green: 50,
            blue: 0,
        },
        bold: false,
        underlined: false,
    };

    fn new_buf(width: Width, height: Height) -> ScreenBuf {
        ScreenBuf::new(Size { width, height }, STYLE_DEFAULT)
    }

    // All characters must be the same width, char_width.
    fn display(
        buf: &mut ScreenBuf,
        s: &str,
        mut pos: Pos,
        style: ConcreteStyle,
        char_width: Width,
    ) {
        for ch in s.chars() {
            buf.display_char(ch, pos, style, char_width).unwrap();
            pos.col += char_width;
        }
    }

    fn assert_out_of_bounds(result: Result<(), TermError>) {
        match result {
            Err(TermError::OutOfBounds) => (),
            x => panic!("expected OutOfBounds error, got {:?}", x),
        }
    }

    fn assert_resized(buf: &mut ScreenBuf, size: Size, good_pos: &[Pos], bad_pos: &[Pos]) {
        buf.resize(size);
        assert_eq!(buf.size(), size);
        for &pos in good_pos {
            buf.display_char('x', pos, STYLE_DEFAULT, 1)
                .unwrap_or_else(|_| panic!("pos {} out-of-bounds of buf with size {}", pos, size));
        }
        for &pos in bad_pos {
            assert_out_of_bounds(buf.display_char('x', pos, STYLE_DEFAULT, 1));
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

        let mut buf = new_buf(1, 1);
        assert_eq!(
            buf.size(),
            Size {
                width: 1,
                height: 1,
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
        let mut buf = new_buf(3, 2);
        let pos = Pos { col: 2, row: 0 };
        buf.display_char('x', pos, STYLE_RED, 1).unwrap();
        let mut actual_ops: Vec<_> = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos::zero()),
                ScreenOp::Style(STYLE_DEFAULT),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Style(STYLE_RED),
                ScreenOp::Print('x'),
                ScreenOp::Goto(Pos { row: 1, col: 0 }),
                ScreenOp::Style(STYLE_DEFAULT),
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
                ScreenOp::Style(STYLE_DEFAULT),
                ScreenOp::Print(' '),
            ]
        );
    }

    #[test]
    fn test_no_change() {
        let mut buf = new_buf(3, 2);
        let pos = Pos { col: 2, row: 0 };
        buf.display_char('x', pos, STYLE_RED, 1).unwrap();
        let mut actual_ops: Vec<_> = buf.drain_changes().collect();

        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos::zero()),
                ScreenOp::Style(STYLE_DEFAULT),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Style(STYLE_RED),
                ScreenOp::Print('x'),
                ScreenOp::Goto(Pos { row: 1, col: 0 }),
                ScreenOp::Style(STYLE_DEFAULT),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
            ]
        );

        // Print same thing as before
        buf.display_char('x', pos, STYLE_RED, 1).unwrap();
        actual_ops = buf.drain_changes().collect();
        assert_eq!(actual_ops, vec![]);
    }

    #[test]
    fn test_shorten() {
        let mut buf = new_buf(3, 1);
        display(&mut buf, "xyz", Pos::zero(), STYLE_RED, 1);
        let mut actual_ops: Vec<_> = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos::zero()),
                ScreenOp::Style(STYLE_RED),
                ScreenOp::Print('x'),
                ScreenOp::Print('y'),
                ScreenOp::Print('z'),
            ]
        );

        display(&mut buf, "xy", Pos::zero(), STYLE_RED, 1);
        actual_ops = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos { col: 2, row: 0 }),
                ScreenOp::Style(STYLE_DEFAULT),
                ScreenOp::Print(' '),
            ]
        );

        display(&mut buf, "x", Pos::zero(), STYLE_RED, 1);
        actual_ops = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos { col: 1, row: 0 }),
                ScreenOp::Style(STYLE_DEFAULT),
                ScreenOp::Print(' '),
            ]
        );

        display(&mut buf, "xy", Pos::zero(), STYLE_RED, 1);
        actual_ops = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos { col: 1, row: 0 }),
                ScreenOp::Style(STYLE_RED),
                ScreenOp::Print('y'),
            ]
        );
    }

    #[test]
    fn test_full_width() {
        let mut buf = new_buf(8, 1);
        display(&mut buf, "1234567", Pos { col: 1, row: 0 }, STYLE_RED, 1);
        let actual_ops = buf.drain_changes().collect::<Vec<_>>();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos::zero()),
                ScreenOp::Style(STYLE_DEFAULT),
                ScreenOp::Print(' '),
                ScreenOp::Style(STYLE_RED),
                ScreenOp::Print('1'),
                ScreenOp::Print('2'),
                ScreenOp::Print('3'),
                ScreenOp::Print('4'),
                ScreenOp::Print('5'),
                ScreenOp::Print('6'),
                ScreenOp::Print('7'),
            ]
        );

        buf.display_char('一', Pos::zero(), STYLE_RED, 2);
        buf.display_char('二', Pos { col: 2, row: 0 }, STYLE_RED, 2);
        buf.display_char('*', Pos { col: 4, row: 0 }, STYLE_RED, 1);
        buf.display_char('三', Pos { col: 5, row: 0 }, STYLE_RED, 2);
        let actual_ops = buf.drain_changes().collect::<Vec<_>>();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos::zero()),
                ScreenOp::Style(STYLE_RED),
                ScreenOp::Print('一'),
                ScreenOp::Print('二'),
                ScreenOp::Print('*'),
                ScreenOp::Print('三'),
                ScreenOp::Style(STYLE_DEFAULT),
                ScreenOp::Print(' '),
            ]
        );

        buf.display_char('3', Pos { col: 3, row: 0 }, STYLE_RED, 1);
        buf.display_char('5', Pos { col: 5, row: 0 }, STYLE_RED, 1);
        let actual_ops = buf.drain_changes().collect::<Vec<_>>();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos::zero()),
                ScreenOp::Style(STYLE_DEFAULT),
                ScreenOp::Print(' '),
                // Should the space be "reprinted", or should there be the goto instead?
                // ScreenOp::Print(' '),
                ScreenOp::Goto(Pos { col: 2, row: 0 }),
                ScreenOp::Print(' '),
                ScreenOp::Style(STYLE_RED),
                ScreenOp::Print('3'),
                ScreenOp::Style(STYLE_DEFAULT),
                ScreenOp::Print(' '),
                ScreenOp::Style(STYLE_RED),
                ScreenOp::Print('5'),
                // Should the space be "reprinted"?
                // ScreenOp::Style(STYLE_DEFAULT),
                // ScreenOp::Print(' '),
            ]
        );
    }

    #[test]
    fn test_replace_full_width_with_space() {
        let mut buf = new_buf(2, 1);
        buf.display_char('一', Pos::zero(), STYLE_DEFAULT, 2);
        buf.drain_changes();
        buf.display_char(' ', Pos::zero(), STYLE_DEFAULT, 1);
        let actual_ops = buf.drain_changes().collect::<Vec<_>>();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos::zero()),
                ScreenOp::Style(STYLE_DEFAULT),
                ScreenOp::Print(' '),
                // Should the space be "reprinted"?
                // ScreenOp::Print(' '),
            ]
        );
    }

    #[test]
    fn test_complex() {
        let mut buf = new_buf(3, 4);
        display(&mut buf, "fo", Pos { col: 1, row: 0 }, STYLE_RED, 1);
        display(&mut buf, "oba", Pos { col: 0, row: 1 }, STYLE_RED, 1);
        display(&mut buf, "r", Pos { col: 0, row: 2 }, STYLE_RED, 1);
        display(&mut buf, "OB", Pos { col: 0, row: 1 }, STYLE_GREEN, 1);
        display(&mut buf, "$", Pos { col: 2, row: 3 }, STYLE_DEFAULT, 1);
        let actual_ops: Vec<_> = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos::zero()),
                ScreenOp::Style(STYLE_DEFAULT),
                ScreenOp::Print(' '),
                ScreenOp::Style(STYLE_RED),
                ScreenOp::Print('f'),
                ScreenOp::Print('o'),
                ScreenOp::Goto(Pos { row: 1, col: 0 }),
                ScreenOp::Style(STYLE_GREEN),
                ScreenOp::Print('O'),
                ScreenOp::Print('B'),
                ScreenOp::Style(STYLE_RED),
                ScreenOp::Print('a'),
                ScreenOp::Goto(Pos { row: 2, col: 0 }),
                ScreenOp::Print('r'),
                ScreenOp::Style(STYLE_DEFAULT),
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
                ScreenOp::Style(STYLE_DEFAULT),
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

        buf.display_char('!', Pos { col: 2, row: 3 }, STYLE_DEFAULT, 1)
            .unwrap();
        let actual_ops: Vec<_> = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos { col: 2, row: 3 }),
                ScreenOp::Style(STYLE_DEFAULT),
                ScreenOp::Print('!'),
            ]
        );
    }
}
