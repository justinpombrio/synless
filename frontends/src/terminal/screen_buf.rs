use super::TermError;
use partial_pretty_printer::{pane::PrettyWindow, Pos, ShadedStyle, Size, Width};

/// Represents a screen full of characters. It buffers changes to the
/// characters, and can produce a set of instructions for efficiently updating
/// the screen to reflect those changes.
#[derive(Debug)]
pub struct ScreenBuf {
    /// Grid of characters covering the screen.
    cells: Vec<Vec<DoubleCharCell>>,
    /// This should always contain the number of lines and cols requested by the
    /// user (eg. 0-by-5), even if `cells` is empty.
    size: Size,
}

/// Represents a single character on a screen, with style properties.
#[derive(Clone, Copy, Debug, PartialEq)]
struct CharCell {
    ch: char,
    style: ShadedStyle,
}

/// Stores both the new unprinted state of a character, and the old state that was last printed to the screen.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
struct DoubleCharCell {
    new: CharCell,
    /// None if unknown (eg. because it's never been printed or the screen was just resized)
    old: Option<CharCell>,
}

/// Instructions for how to update a terminal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScreenOp {
    /// Print a character at the current cursor position, and advance the cursor by 1.
    Print(char),
    /// Set a persistent style that will apply to anything printed, until a new style is applied.
    Apply(ShadedStyle),
    /// Set the cursor position.
    Goto(Pos),
}

/// States of the ScreenBufIter
enum State {
    FindDirty,
    CheckStyle,
    PrintChar,
}

/// An iterator that produces instructions for updating a screen to match changes in a ScreenBuf.
pub struct ScreenBufIter<'a> {
    buf: &'a mut ScreenBuf,
    /// The last style applied to the screen. It will persist until a new style is applied.
    current_style: Option<ShadedStyle>,
    /// On the very first iteration, we don't know where the terminal cursor is
    /// yet. We have to set it after we find the first dirty character cell,
    /// before printing the character.
    on_first_iteration: bool,
    /// Which cell the iterator is considering (NOT the position of the
    /// terminal's cursor). None means the iterator has just been constructed
    /// and is pointing to immediately before the first cell of the ScreenBuf.
    pos: Option<Pos>,
    /// Which action the iterator should do next.
    next_state: State,
}

enum FindDirtyResult {
    AtEnd,
    GotoDirty(ScreenOp),
    AtDirty,
}

impl ScreenBuf {
    pub fn new() -> Self {
        ScreenBuf {
            cells: Vec::new(),
            size: Size {
                width: 0,
                height: 0,
            },
        }
    }

    /// Get `ScreenOp` instructions that describe all changes to the screen buffer since the last time this method was called.
    pub fn drain_changes(&mut self) -> ScreenBufIter {
        ScreenBufIter {
            buf: self,
            current_style: None,
            pos: None,
            on_first_iteration: true,
            next_state: State::FindDirty,
        }
    }

    pub fn resize(&mut self, size: Size) {
        self.cells = Vec::new();
        let mut line = Vec::new();
        line.resize_with(size.width as usize, Default::default);
        self.cells.resize(size.height as usize, line);
        self.size = size;
    }

    fn set_char_with_style(
        &mut self,
        pos: Pos,
        ch: char,
        style: ShadedStyle,
    ) -> Result<(), TermError> {
        let cell = self.get_mut(pos)?;
        cell.set_char(ch);
        cell.set_style(style);
        Ok(())
    }

    fn get(&self, pos: Pos) -> Result<DoubleCharCell, TermError> {
        self.cells
            .get(pos.line as usize)
            .and_then(|line| line.get(pos.col as usize))
            .copied()
            .ok_or(TermError::OutOfBounds)
    }

    fn get_mut(&mut self, pos: Pos) -> Result<&mut DoubleCharCell, TermError> {
        self.cells
            .get_mut(pos.line as usize)
            .and_then(|line| line.get_mut(pos.col as usize))
            .ok_or(TermError::OutOfBounds)
    }

    fn next_pos(&self, old_pos: Pos) -> Option<Pos> {
        if old_pos.col >= (self.size.width - 1) {
            // At the last column of a line
            if old_pos.line >= (self.size.height - 1) {
                // At the last line too, that's the last position on the the screen!
                None
            } else {
                // Go to start of next line
                Some(Pos {
                    line: old_pos.line + 1,
                    col: 0,
                })
            }
        } else {
            // Go forward 1 column
            Some(Pos {
                line: old_pos.line,
                col: old_pos.col + 1,
            })
        }
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
            self.set_char_with_style(pos, ch, style)?;
            pos.col += 1;
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
        for _ in 0..len {
            self.set_char_with_style(pos, ch, style)?;
            pos.col += 1;
        }
        Ok(())
    }
}

impl DoubleCharCell {
    fn set_char(&mut self, ch: char) {
        self.new.ch = ch;
    }

    fn set_style(&mut self, style: ShadedStyle) {
        self.new.style = style;
    }

    fn get(&self) -> CharCell {
        self.new
    }

    fn mark(&mut self) {
        self.old = Some(self.new);
        self.new = CharCell::default();
    }

    fn is_dirty(&self) -> bool {
        if let Some(old) = self.old {
            self.new != old
        } else {
            true
        }
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
    fn advance(&mut self) -> Option<Pos> {
        if let Some(p) = self.pos {
            self.buf.get_mut(p).unwrap().mark();
            self.pos = Some(self.buf.next_pos(p)?);
        } else {
            // Start at the beginning
            self.pos = Some(Pos::zero())
        }
        self.pos
    }

    fn pos(&self) -> Pos {
        // If pos is None, the iterator is still _before_ the first cell, and not _at_ any cell.
        self.pos
            .expect("position not defined until advance() is called")
    }

    fn cell(&self) -> CharCell {
        self.buf.get(self.pos()).unwrap().get()
    }

    fn at_dirty_cell(&self) -> bool {
        self.buf.get(self.pos()).unwrap().is_dirty()
    }

    fn find_dirty(&mut self) -> FindDirtyResult {
        // Look for the next cell after this one that needs to be redisplayed.
        if self.advance().is_none() {
            return FindDirtyResult::AtEnd;
        }

        let mut jumped = false;
        while !self.at_dirty_cell() {
            if self.advance().is_none() {
                return FindDirtyResult::AtEnd;
            }
            jumped = true;
        }

        // Check if we need to explicitly jump the cursor to this position.
        if jumped || self.on_first_iteration {
            self.on_first_iteration = false;
            FindDirtyResult::GotoDirty(ScreenOp::Goto(self.pos()))
        } else {
            FindDirtyResult::AtDirty
        }
    }

    fn check_style(&mut self) -> Option<ScreenOp> {
        // Check if it has a different style than the last one we applied.
        let new_style = self.cell().style;
        let style_changed = match self.current_style {
            None => true,
            Some(s) => s != new_style,
        };

        if style_changed {
            self.current_style = Some(new_style);
            assert!(!self.on_first_iteration);
            Some(ScreenOp::Apply(new_style))
        } else {
            None
        }
    }

    fn print_char(&mut self) -> ScreenOp {
        // Finally, print the character itself
        assert!(!self.on_first_iteration);
        ScreenOp::Print(self.cell().ch)
    }
}

impl<'a> Iterator for ScreenBufIter<'a> {
    type Item = ScreenOp;
    fn next(&mut self) -> Option<ScreenOp> {
        loop {
            match self.next_state {
                State::FindDirty => {
                    self.next_state = State::CheckStyle;
                    match self.find_dirty() {
                        FindDirtyResult::GotoDirty(op) => {
                            return Some(op);
                        }
                        FindDirtyResult::AtDirty => (), // No Goto op needed, continue to next state
                        FindDirtyResult::AtEnd => {
                            // Done! Reached the end of the buffer.
                            return None;
                        }
                    }
                }
                State::CheckStyle => {
                    self.next_state = State::PrintChar;
                    let op = self.check_style();
                    if op.is_some() {
                        return op;
                    }
                }
                State::PrintChar => {
                    self.next_state = State::FindDirty;
                    return Some(self.print_char());
                }
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
            buf.set_char_with_style(pos, 'x', ShadedStyle::plain())
                .unwrap_or_else(|_| panic!("pos {} out-of-bounds of buf with size {}", pos, size));
        }
        for &pos in bad_pos {
            assert_out_of_bounds(buf.set_char_with_style(pos, 'x', ShadedStyle::plain()));
        }
    }

    #[test]
    fn test_resize() {
        let c0r0 = Pos { col: 0, line: 0 };
        let c0r1 = Pos { col: 0, line: 1 };
        let c1r0 = Pos { col: 1, line: 0 };
        let c1r1 = Pos { col: 1, line: 1 };
        let c5r8 = Pos { col: 5, line: 8 };
        let c5r7 = Pos { col: 5, line: 7 };
        let c4r7 = Pos { col: 4, line: 7 };
        let c4r8 = Pos { col: 4, line: 8 };

        let mut buf = ScreenBuf::new();
        assert_eq!(
            buf.size().unwrap(),
            Size {
                height: 0,
                width: 0,
            },
        );
        assert_out_of_bounds(buf.set_char_with_style(c0r0, 'x', ShadedStyle::plain()));

        assert_resized(
            &mut buf,
            Size {
                height: 0,
                width: 0,
            },
            &[],
            &[c0r0, c1r0, c0r1],
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
        let mut buf = ScreenBuf::new();
        buf.resize(Size {
            width: 3,
            height: 2,
        });

        let pos = Pos { col: 2, line: 0 };
        buf.print(pos, "x", style1).unwrap();
        let mut actual_ops: Vec<_> = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos::zero()),
                ScreenOp::Apply(ShadedStyle::plain()),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Apply(style1),
                ScreenOp::Print('x'),
                ScreenOp::Apply(ShadedStyle::plain()),
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
                ScreenOp::Apply(ShadedStyle::plain()),
                ScreenOp::Print(' '),
            ]
        );
    }

    #[test]
    fn test_no_change() {
        let style1 = ShadedStyle::new(Style::color(Color::Base09), Shade::background());
        let mut buf = ScreenBuf::new();
        buf.resize(Size {
            width: 3,
            height: 2,
        });

        let pos = Pos { col: 2, line: 0 };
        buf.print(pos, "x", style1).unwrap();
        let mut actual_ops: Vec<_> = buf.drain_changes().collect();

        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos::zero()),
                ScreenOp::Apply(ShadedStyle::plain()),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Apply(style1),
                ScreenOp::Print('x'),
                ScreenOp::Apply(ShadedStyle::plain()),
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

        let mut buf = ScreenBuf::new();
        buf.resize(Size {
            width: 3,
            height: 1,
        });

        buf.print(Pos::zero(), "xyz", style1).unwrap();
        let mut actual_ops: Vec<_> = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos::zero()),
                ScreenOp::Apply(style1),
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
                ScreenOp::Goto(Pos { col: 2, line: 0 }),
                ScreenOp::Apply(ShadedStyle::plain()),
                ScreenOp::Print(' '),
            ]
        );

        buf.print(Pos::zero(), "x", style1).unwrap();
        actual_ops = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos { col: 1, line: 0 }),
                ScreenOp::Apply(ShadedStyle::plain()),
                ScreenOp::Print(' '),
            ]
        );

        buf.print(Pos::zero(), "xy", style1).unwrap();
        actual_ops = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos { col: 1, line: 0 }),
                ScreenOp::Apply(style1),
                ScreenOp::Print('y'),
            ]
        );
    }

    #[test]
    fn test_complex() {
        let style1 = ShadedStyle::new(Style::color(Color::Base09), Shade::background());
        let style2 = ShadedStyle::new(Style::color(Color::Base0C), Shade::background());

        let mut buf = ScreenBuf::new();
        buf.resize(Size {
            width: 3,
            height: 4,
        });

        buf.print(Pos { col: 1, line: 0 }, "fo", style1).unwrap();
        buf.print(Pos { col: 0, line: 1 }, "oba", style1).unwrap();
        buf.print(Pos { col: 0, line: 2 }, "r", style1).unwrap();

        buf.print(Pos { col: 0, line: 1 }, "OB", style2).unwrap();

        buf.print(Pos { col: 2, line: 3 }, "$", ShadedStyle::plain())
            .unwrap();

        let mut actual_ops: Vec<_> = buf.drain_changes().collect();

        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos::zero()),
                ScreenOp::Apply(ShadedStyle::plain()),
                ScreenOp::Print(' '),
                ScreenOp::Apply(style1),
                ScreenOp::Print('f'),
                ScreenOp::Print('o'),
                ScreenOp::Apply(style2),
                ScreenOp::Print('O'),
                ScreenOp::Print('B'),
                ScreenOp::Apply(style1),
                ScreenOp::Print('a'),
                ScreenOp::Print('r'),
                ScreenOp::Apply(ShadedStyle::plain()),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Print('$'),
            ]
        );
        actual_ops = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos { col: 1, line: 0 }),
                ScreenOp::Apply(ShadedStyle::plain()),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Goto(Pos { col: 2, line: 3 }),
                ScreenOp::Print(' '),
            ]
        );

        buf.set_char_with_style(Pos { col: 2, line: 3 }, '!', ShadedStyle::plain())
            .unwrap();

        actual_ops = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos { col: 2, line: 3 }),
                ScreenOp::Apply(ShadedStyle::plain()),
                ScreenOp::Print('!'),
            ]
        );
    }
}
